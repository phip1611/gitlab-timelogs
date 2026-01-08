# gitlab-timelogs CLI

A lightweight CLI to fetch, summarize, and validate your GitLab issue
time logs. Group entries by week, filter by date ranges, and spot anomalies
like weekend work or >10h days. Read-only, fast, and cross-platform - built
to make time tracking in GitLab finally usable.

Made by developers for developers. `gitlab-timelogs` **is not** associated
with the official GitLab project!

![screenshot.png](screenshot.png)
(_The screenshot is slightly outdated. The latest version shows more information._)

## Features

`gitlab-timelogs` provides you with an overview of your time logs and prints
warnings for typical mistakes. It does not allow you to modify entries, but just
to inspect existing records, so you can fix them in GitLab (if necessary).

- ✅ collect time logs from issues (timelogs associated with MRs currently not
  supported)
- ✅ group them by week
- ✅ specify time range and apply filters (such as group filter)
- ✅ print warnings for common pitfalls:
    - accounted less than 15 minutes to an issue (typically a mistake)
    - accounted time to a Saturday or Sunday (not common in normal positions)
      (at least in Europe 😀)
    - accounted more than 10h a day (10h is the legal maximum in Germany)

## GitLab Server Support

Development of this CLI began with GitLab 16.11. Since then, it has been
regularly tested against the latest stable release (currently 18.2). Because it
only relies on basic parts of the GitLab API, it should work across a wide
range of GitLab versions.

Note: Certain features may be unavailable on the GitLab Free tier.

## Supported Platforms
_(For compilation and running.)_

`gitlab-timelogs` builds and runs at least on the following platforms:

- ✅ Linux
- ✅ MacOS
- ✅ Windows

including different versions and architectures that Rust supports (x86, ARM).

## Consume / Install

**Via cargo:**

- `$ cargo install gitlab-timelogs`

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
    # Token with global `read_api` permission.
    #
    # CAUTION! Do not push your token to a public git repository!
    gitlabToken = with builtins; readFile (toPath ./gitlab-token.txt);
  };
};
```

## Usage

- `$ gitlab-timelogs --help`

If you created `config.toml` (see below):

- `$ gitlab-timelogs`
- `$ gitlab-timelogs --after 2024-06-01 --before 2024-06-30`

otherwise, a direct invocation works as follows:

- `$ GITLAB_TOKEN="your-token" gitlab-timelogs --host gitlab.example.com --username your-username`

_**Hint**: You need a GitLab token with `read_api` permission. \
<https://gitlab.example.com/-/user_settings/personal_access_tokens>_

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
    # Token with global `read_api` permission.
    gitlab_token = "<token>"
    ```

## MSRV

The MSRV is Rust stable `1.85.0`.

## Trivia

The main motivation to create this was the unbelievable poor UX of the GitLab
web UI for time logs at that given time. For example, the input mask transformed a `1h 30` to
`3d 7h` instead of `1h 30m`. This common pitfall was unbelievably annoying and
hard to spot - badly influencing a lot of our time records.

Hence, I created this as part of my work time at [Cyberus Technology GmbH](https://cyberus-technology.de)
to boost our internal productivity. We love open source! Interested in a
cool employer? Contact us!
