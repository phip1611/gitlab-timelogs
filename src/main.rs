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

use crate::cfg::get_cfg;
use crate::cli::CliArgs;
use crate::fetch::fetch_results;
use crate::gitlab_api::types::ResponseNode;
use anyhow::{anyhow, Context};
use chrono::{Datelike, NaiveDate, Weekday};
use nu_ansi_term::{Color, Style};
use std::error::Error;
use std::time::Duration;

mod cfg;
mod cli;
mod fetch;
mod gitlab_api;
mod views;

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = get_cfg()?;
    if cfg.before() < cfg.after() {
        Err(anyhow!(
            "The `--before` date must come after the `--after` date"
        ))
        .context("Failed to validate config")?;
    }

    let response = fetch_results(
        cfg.username(),
        cfg.host(),
        cfg.token(),
        cfg.after(),
        cfg.before(),
    )?;

    println!("Host     : {}", cfg.host());
    println!("Username : {}", cfg.username());
    println!("Time Span: {} - {}", cfg.after(), cfg.before());

    // All nodes but as vector to references.
    // Simplifies the handling with other parts of the code, especially the
    // `views` module.
    let nodes = response.timelogs.nodes.iter().collect::<Vec<_>>();

    if nodes.is_empty() {
        print_warning(
            "Empty response. Is the username correct? Does the token has read permission?",
            0,
        );
    } else {
        print_all_weeks(nodes.as_slice(), &cfg);
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
        print_warning(
            "^ ERROR: You have logged this time as NEGATIVE: Update the ticket!",
            3,
        );
    }
    if duration.as_secs() / 60 < min_minutes_threshold {
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
            print_warning("^ WARN: More than 10 hours! Is this correct?", 18);
        }

        match day.weekday() {
            Weekday::Sat | Weekday::Sun => {
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

fn print_extended_summary(nodes: &[&ResponseNode]) {
    {
        let nodes_by_epic = views::to_nodes_by_epic(nodes);
        for (epic, nodes_of_epic) in nodes_by_epic {
            let duration = views::to_time_spent_sum(&nodes_of_epic);
            print!("  ");
            print_duration(duration, Color::Magenta);
            print!(
                " - {epic_key}  {epic_name}",
                epic_key = Style::new().dimmed().paint("Epic:"),
                epic_name = Style::new().bold().paint(
                    epic.as_ref()
                        .map(|e| e.title.as_str())
                        .unwrap_or("<No Epic>")
                )
            );
            println!();
        }
    }
    {
        let nodes_by_issue = views::to_nodes_by_issue(nodes);
        for (issue, nodes_of_issue) in nodes_by_issue {
            let duration = views::to_time_spent_sum(&nodes_of_issue);
            print!("  ");
            print_duration(duration, Color::Magenta);
            print!(
                " - Issue: {issue_name}",
                issue_name = Style::new().bold().fg(Color::Green).paint(issue.title)
            );
            println!();
        }
    }
}

fn print_final_summary(nodes: &[&ResponseNode], cfg: &CliArgs) {
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

    if cfg.print_extended_summary() {
        println!();
        print_extended_summary(nodes);
    }
}

fn print_all_weeks(nodes: &[&ResponseNode], cfg: &CliArgs) {
    let view = views::to_nodes_by_week(nodes);
    for (i, (week, nodes_of_week)) in view.iter().enumerate() {
        print_week((week.year(), week.week()), nodes_of_week);

        let is_last = i == view.len() - 1;
        if !is_last {
            println!();
        }
    }

    print_final_summary(nodes, cfg);
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
