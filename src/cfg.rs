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

//! Module for obtaining the effective configuration, based on the configuration
//! file and CLI parameters.
//!
//! [`get_cfg`] is the entry point.

use crate::cli::{CfgFile, CliArgs};
use crate::{cli, print_warning};
use clap::Parser;
use serde::de::DeserializeOwned;
use std::error::Error;
use std::io::ErrorKind;
use std::path::PathBuf;

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
pub fn get_cfg() -> Result<CliArgs, Box<dyn Error>> {
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
