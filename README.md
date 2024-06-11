# gitlab-timelogs CLI

This CLI collects all timelogs to all tickets in the provided GitLab instance
and prints them out in a helpful way. This works around various limitations of
the very bad GitLab UI for timelogs (as of Mid-2024). It helps to check if you:
- accounted your 8 hours of time per day split across multiple
  tickets
- accounted time to a Saturday or Sunday (which we normally don't do), and
- to check if you accounted more than 10h for a day (normally not legally
  possible in Germany)

![screenshot.png](screenshot.png)

## Install

Via cargo:
- `cargo install --git https://gitlab.vpn.cyberus-technology.de/pschuster/gitlab-timelogs`

Via Nix:
- Option A: add this flake as input and add the package into your system config
- Option B: `$ nix shell git+ssh://git@gitlab.vpn.cyberus-technology.de/pschuster/gitlab-timelogs.git\?ref=main`

## Usage

- `$ gitlab-timelogs --help`
- `$ gitlab-timelogs --host gitlab.vpn.cyberus-technology.de --username pschuster --token ********** --days 3`

You can provide all parameters also as environment variable.
