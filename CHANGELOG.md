# Unreleased (Yet)

## v0.6.0 (2025-08-16)

- GraphQL Error Responses from GitLab are now handled and displayer properly
  ([#38](https://github.com/phip1611/gitlab-timelogs/issues/38))
- Updated dependencies
- Use Rust edition 2024
- MSRV is now 1.85

## v0.5.0 (2024-12-17)

- Added basic error reporting. For example, the CLI might now tell you:
  - ```
    Error:
    Failed to parse response body as JSON

    Caused by:
        0: error decoding response body
        1: missing field `webUrl2` at line 1 column 308
    ```
  - ```
    Error:
    Failed to receive proper response

    Caused by:
        HTTP status client error (401 Unauthorized) for url (https://gitlab.example.com/api/graphql)
    ```

## v0.4.1 (2024-12-17)

- Better error reporting: it is now clearer if the request failed due to a
  wrong or expired token, for example.

## v0.4.0 (2024-09-03)

- Added `-x/--extended-summary` flag to show an extended summary at the end
  of the output, where the summarized time per epic and per issue is listed.
- internal code improvements

## v0.3.0 (2024-08-26)

- time span filtering already happens on the server-side, which accelerates
  requests by a notable amount.
- updated dependencies

## v0.2.2 (2024-07-04)

- improve handling of default xdg config dir (unix only)
- fix typos

## v0.2.1 (2024-07-04)

- fix documentation bugs

## v0.2.0 (2024-07-04)

- tool now also builds and runs on Windows
- the nix build for Darwin/macOS was fixed

## v0.1.1 (2024-07-04)

- doc & README updates

## v0.1.0 (2024-06-27)

- initial release
