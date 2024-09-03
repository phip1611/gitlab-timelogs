/*
MIT License

Copyright (c) 2024 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::must_use_candidate,
    // clippy::restriction,
    // clippy::pedantic
)]
// now allow a few rules which are denied by the above statement
// --> they are ridiculous and not necessary
#![allow(
    clippy::suboptimal_flops,
    clippy::redundant_pub_crate,
    clippy::fallible_impl_from
)]
// I can't do anything about this; fault of the dependencies
#![allow(clippy::multiple_crate_versions)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]

use crate::cli::CfgFile;
use crate::fetch::fetch_results;
use crate::filtering::filter_timelogs;
use crate::gitlab_api::types::ResponseNode;
use chrono::{Datelike, NaiveDate, Weekday};
use clap::Parser;
use cli::CliArgs;
use nu_ansi_term::{Color, Style};
use serde::de::DeserializeOwned;
use std::error::Error;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;

mod cli;
mod fetch;
mod filtering;
mod gitlab_api;
mod views;

/// Returns the path of the config file with respect to the current OS.
fn config_file_path() -> Result<PathBuf, Box<dyn Error>> {
    #[cfg(target_family = "unix")]
    let config_os_dir = {
        // First look for XDG_CONFIG_HOME, then fall back to HOME
        // https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
        let home = std::env::var("XDG_CONFIG_HOME").unwrap_or(std::env::var("HOME")?);
        PathBuf::from(home).join(".config")
    };
    #[cfg(target_family = "windows")]
    let config_os_dir = PathBuf::from(std::env::var("LOCALAPPDATA")?);

    let config_dir = config_os_dir.join("gitlab-timelogs");
    Ok(config_dir.join("config.toml"))
}

/// Reads the config file and parses it from TOML.
/// On UNIX, it uses `
fn read_config_file<T: DeserializeOwned>() -> Result<T, Box<dyn Error>> {
    let config_file = config_file_path()?;
    let content = match std::fs::read_to_string(&config_file) {
        Ok(c) => c,
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {}
                _ => print_warning(
                    &format!(
                        "Failed to read config file at {}: {e}",
                        config_file.display()
                    ),
                    0,
                ),
            }

            // Treat failure to read a config file as the empty config file.
            String::new()
        }
    };

    Ok(toml::from_str(&content)?)
}

/// Parses the command line options but first, reads the config file. If certain
/// command line options are not present, they are taken from the config file.
///
/// This is a workaround that clap has no built-in support for a config file
/// that serves as source for command line options by itself. The focus is
/// also on the natural error reporting by clap.
fn get_cli_cfg() -> Result<CliArgs, Box<dyn Error>> {
    let config_content = read_config_file::<CfgFile>()?;
    let config_args: Vec<(String, String)> = config_content.to_cli_args();
    let mut all_args = std::env::args().collect::<Vec<_>>();

    // Push config options as arguments, before parsing them in clap.
    for (opt_name, opt_value) in config_args {
        if !all_args.contains(&opt_name) {
            all_args.push(opt_name);
            all_args.push(opt_value);
        }
    }

    Ok(cli::CliArgs::parse_from(all_args))
}

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = get_cli_cfg()?;
    assert!(cfg.before() >= cfg.after());
    println!("Host     : {}", cfg.host());
    println!("Username : {}", cfg.username());
    println!("Time Span: {} - {}", cfg.after(), cfg.before());

    let data_all = fetch_results(
        cfg.username(),
        cfg.host(),
        cfg.token(),
        cfg.after(),
        cfg.before(),
    );

    // All dates with timelogs.
    let data_filtered = filter_timelogs(
        &data_all, None, /* time already filtered on server */
        None, None,
    )
    .collect::<Vec<_>>();

    if data_filtered.is_empty() {
        print_warning(
            "Empty response. Is the username correct? Does the token has read permission?",
            0,
        );
    } else {
        print_all_weeks(data_filtered.as_slice());
    }

    Ok(())
}

fn print_timelog(log: &ResponseNode) {
    let (duration_is_positive, duration) = log.timeSpent();
    print!("  ");
    print_duration(duration, Color::Magenta);
    println!(
        "  {issue_name}",
        issue_name = Style::new()
            .bold()
            .fg(Color::Green)
            .paint(log.issue.title.clone()),
    );
    let min_minutes_threshold = 15;
    if !duration_is_positive {
        // msg is aligned with the suspicious data output
        print_warning(
            "^ ERROR: You have logged this time as NEGATIVE: Update the ticket!",
            3,
        );
    }
    if duration.as_secs() / 60 < min_minutes_threshold {
        // msg is aligned with the suspicious data output
        print_warning("^ WARN: Less than 15 minutes! Is this correct?", 6);
    }

    // Print issue metadata.
    let epic_name = log.epic_name().unwrap_or("<no epic>");
    let whitespace = " ".repeat(11);
    println!(
        "{whitespace}{link}",
        link = Style::new().dimmed().paint(&log.issue.webUrl)
    );
    if let Some(group) = &log.project.group {
        println!(
            "{whitespace}[{epic_key} {epic_name}, {group_key} {group_name}]",
            epic_key = Style::new().dimmed().paint("Epic:"),
            epic_name = Style::new().bold().paint(epic_name),
            group_key = Style::new().dimmed().paint("Group:"),
            group_name = Style::new().bold().paint(&group.fullName),
            whitespace = " ".repeat(11),
        );
    }

    if let Some(lines) = log.summary.as_ref().map(|t| t.lines()) {
        for line in lines {
            println!("             {line}");
        }
    }
}

fn print_warning(msg: &str, indention: usize) {
    println!(
        "{indention}{msg}",
        indention = " ".repeat(indention),
        msg = Style::new().bold().fg(Color::Yellow).paint(msg),
    );
}

fn print_date(day: &NaiveDate, nodes_of_day: &[&ResponseNode]) {
    let total = views::to_time_spent_sum(nodes_of_day);

    let day_print = format!("{day}, {}", day.weekday());

    print!("{}  (", Style::new().bold().paint(day_print));
    print_duration(total, Color::Blue);
    println!(")");

    // Sanity checks and print warnings
    {
        let max_hours_threshold = 10;
        if total.as_secs() > max_hours_threshold * 60 * 60 {
            // msg is aligned with the suspicious data output
            print_warning("^ WARN: More than 10 hours! Is this correct?", 18);
        }

        match day.weekday() {
            Weekday::Sat | Weekday::Sun => {
                // msg is aligned with the suspicious data output
                print_warning("^ WARN: You shouldn't work on the weekend, right?", 12);
            }
            _ => {}
        }
    }

    for log in nodes_of_day {
        print_timelog(log);
    }
}

fn print_week(week: (i32 /* year */, u32 /* iso week */), nodes_of_week: &[&ResponseNode]) {
    let week_style = Style::new().bold();
    let week_print = format!("WEEK {}-W{:02}", week.0, week.1);
    println!(
        "{delim} {week_print} {delim}",
        delim = week_style.paint("======================"),
        week_print = week_style.paint(week_print)
    );
    let total_week_time = views::to_time_spent_sum(nodes_of_week);
    print!(
        "{total_time_key}       ",
        total_time_key = Style::new().bold().paint("Total time:")
    );
    print_duration(total_week_time, Color::Blue);
    println!();
    println!();

    let nodes_by_day = views::to_nodes_by_day(nodes_of_week);

    for (i, (day, nodes)) in nodes_by_day.iter().enumerate() {
        print_date(day, nodes);

        let is_last = i == nodes_by_day.len() - 1;
        if !is_last {
            println!();
        }
    }
}

fn print_final_summary(nodes: &[&ResponseNode]) {
    // Print separator.
    {
        println!();
        // same length as the week separator
        println!("{}", "-".repeat(59));
        println!();
    }

    let total_time = views::to_time_spent_sum(nodes);
    let all_days = views::to_nodes_by_day(nodes);

    print!(
        "{total_time_key} ({days_amount:>2} days with records): ",
        total_time_key = Style::new().bold().paint("Total time"),
        days_amount = all_days.len(),
    );
    print_duration(total_time, Color::Blue);
    println!();

    // TODO print by epic, by issue, and by group
}

fn print_all_weeks(nodes: &[&ResponseNode]) {
    let view = views::to_nodes_by_week(nodes);
    for (i, (week, nodes_of_week)) in view.iter().enumerate() {
        print_week((week.year(), week.week()), nodes_of_week);

        let is_last = i == view.len() - 1;
        if !is_last {
            println!();
        }
    }

    print_final_summary(nodes);
}

const fn duration_to_hhmm(dur: Duration) -> (u64, u64) {
    let hours = dur.as_secs() / 60 / 60;
    let remaining_secs = dur.as_secs() - (hours * 60 * 60);
    let minutes = remaining_secs / 60;
    (hours, minutes)
}

fn print_duration(duration: Duration, color: Color) {
    let (hours, minutes) = duration_to_hhmm(duration);
    let print_str = format!("{hours:>2}h {minutes:02}m");
    print!("{}", Style::new().bold().fg(color).paint(print_str));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_to_hhmm() {
        assert_eq!(duration_to_hhmm(Duration::from_secs(0)), (0, 0));
        assert_eq!(duration_to_hhmm(Duration::from_secs(59)), (0, 0));
        assert_eq!(duration_to_hhmm(Duration::from_secs(60)), (0, 1));
        assert_eq!(duration_to_hhmm(Duration::from_secs(61)), (0, 1));
        assert_eq!(duration_to_hhmm(Duration::from_secs(119)), (0, 1));
        assert_eq!(duration_to_hhmm(Duration::from_secs(120)), (0, 2));
        let h = 3;
        let m = 7;
        assert_eq!(
            duration_to_hhmm(Duration::from_secs(h * 60 * 60 + m * 60)),
            (h, m)
        );
    }
}
