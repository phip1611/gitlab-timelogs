# gitlab-timelogs CLI

This CLI collects all timelogs to all tickets for the past days in the
provided GitLab instance and prints them out in a helpful way. The tool works
around various limitations of the very bad (as of Mid-2024) GitLab UI support
for timelogs.


## Target Audience

Developers who track their times in GitLab issues. Use it to check if you:
- accounted less than 15 minutes to an issue (typically a mistake)
- accounted your full 8 hours of time per day split across multiple
  tickets
- didn't account time to a Saturday or Sunday (which we normally don't do), and
- to check if you accounted more than 10h for a day (normally not legally
  possible in Germany)

![screenshot.png](screenshot.png)

## Install

Via cargo:
- `cargo install --git https://gitlab.vpn.cyberus-technology.de/pschuster/gitlab-timelogs`

Via Nix:
- Option A: Add `gitlab-overview.nixosModules.default` (referring to the flake
  input) to the modules of your NixOS configuration, which will add
  `gitlab-timelogs` to your system-wide packages.
- Option B: `$ nix shell git+ssh://git@gitlab.vpn.cyberus-technology.de/pschuster/gitlab-timelogs.git\?ref=main`
- Option C: add this flake as input and add the package into your system config


## Usage

- `$ gitlab-timelogs --help`
- `$ gitlab-timelogs --host gitlab.vpn.cyberus-technology.de --username pschuster --token ********** --after 2024-06-01 --before 2024-06-30`


### Configuration

1. Via CLI options. Type `--help` for guidance.
2. Via environment variables:
   - `GITLAB_HOST`
   - `GITLAB_USERNAME`
   - `GITLAB_TOKEN`
3. Via a `~/.config/gitlab-timelogs/config.toml` file:
    ```toml
    gitlab_host = "gitlab.example.com"
    gitlab_username = "<user>"
    gitlab_token = "<token>"
    ```
