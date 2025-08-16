# gitlab-timelogs CLI

CLI utility to assist you with your time logs in GitLab. The GitLab web UI for
time logs is very rudimentary and the UX is poor (June 2024, GitLab 16.11),
especially a summary view is missing. This is where `gitlab-timelogs` help you
by leveraging the GitLab API.

This CLI is made by developers for developers who want to check their timelogs
at the of the work day or week. `gitlab-timelogs` **is not** associated with the
official GitLab project!

![screenshot.png](screenshot.png)
(_The screenshot is slightly outdated. The latest version shows more information._)

## Features

`gitlab-timelogs` provides you with an overview of your time logs and prints
warnings for typical mistakes. It does not allow you to modify entries, but just
to inspect existing records, so you can fix them in GitLab (if necessary).

- ✅ collect time logs from issues (timelogs associated with MRs currently not
  supported)
- ✅ group them by week
- ✅ specify time range
- ✅ print warnings for common pitfalls:
    - accounted less than 15 minutes to an issue (typically a mistake)
    - accounted time to a Saturday or Sunday (not common in normal positions)
    - accounted more than 10h a day (10h is the legal maximum in Germany)
- ✅ Created for GitLab 16.11. Older and newer versions should work as well,
     but haven't been tested. Note that the free tier may not support time
     logs, but only the enterprise edition.

## Supported Platforms
_(For compilation and running.)_

`gitlab-timelogs` builds and runs at least on the following platforms:

- Linux (all architectures, I guess?)
- MacOS (all architectures, I guess?)
- Windows (all architectures, I guess?)

Note that I only tested recent versions of these OSs in Mid-2024. Older versions
of these systems should work as well.

## Consume / Install

**Via cargo:**

- `$ cargo install https://github.com/phip1611/gitlab-timelogs`

**Via Nix / on NixOS:**

- Option A: [via `nixpkgs`](https://search.nixos.org/packages?channel=unstable&from=0&size=50&sort=relevance&type=packages&query=gitlab-timelogs)
  - A1: Add `pkgs.gitlab-timelogs` to your packages
  - A2: Use `nix-shell -p gitlab-timelogs`
- Option B: consume this Flake/Repository
  - B1: Add `gitlab-timelogs.nixosModules.default` (`gitlab-timelogs` is
    referring to the flake input) to the modules of your NixOS configuration,
    which will add `gitlab-timelogs` to your system-wide packages.
  - B2: Open a shell: `$ nix shell github:phip1611/gitlab-timelogs`
  - B3: Run the tool: `$ nix run github:phip1611/gitlab-timelogs -- <args>`

**Via home-manager:**

1. import the home-manager module: `gitlab-timelogs.nixosModules.home-manager`
2. enable and configure gitlab-timelogs:

```nix
gitlab-timelogs = {
  enable = true;
  config = {
    gitlabHost = "gitlab.example.com";
    gitlabUsername = "exampleuser";
    # Either write as a string here, or read from a file that you do not push:
    gitlabToken = with builtins; readFile (toPath ./gitlab-token.txt);
  };
};
```

## Usage

- `$ gitlab-timelogs --help`
- `$ gitlab-timelogs --host gitlab.vpn.cyberus-technology.de --username pschuster --token ********** --after 2024-06-01 --before 2024-06-30`

### Configuration

1. Via CLI options. Type `--help` for guidance.
2. Via environment variables:
    - `GITLAB_HOST`
    - `GITLAB_USERNAME`
    - `GITLAB_TOKEN`
3. Via a configuration file either in
   `~/.config/gitlab-timelogs/config.toml` (UNIX) or \
   `%LOCALAPPDATA%/gitlab-timelogs/config.toml` (Windows)
   with the following content: \
    ```toml
    gitlab_host = "gitlab.example.com"
    gitlab_username = "<user>"
    gitlab_token = "<token>"
    ```

## MSRV

The MSRV is Rust stable `1.85.0`.

## Trivia

The main motivation to create this was the unbelievable poor UX of the GitLab
web UI for time logs. For example, the input mask transformed a `1h 30` to
`3d 7h` instead of `1h 30m`. This common pitfall was unbelievable annoying and
hard to spot - badly influencing a lot of our time records.

Hence, I created this as part of my work time at [Cyberus Technology GmbH](https://cyberus-technology.de)
to boost our internal productivity. We love open source! Interested in a
cool employer? Contact us!
