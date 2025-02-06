# Changelog

## v0.5.2 (2025-02-05)

- Include experimental commands in --help:
  <https://github.com/facebook/dotslash/pull/44>

## v0.5.1 (2025-02-03)

- Improved the error message for GitHub provider auth failures:
  <https://github.com/facebook/dotslash/pull/43>

## v0.5.0 (2025-01-13)

- Added `arg0` artifact entry config field:
  <https://github.com/facebook/dotslash/pull/37>
- MSRV 1.83.

## v0.4.3 (2024-10-13)

- Fix MUSL aarch64 linux releases:
  <https://github.com/facebook/dotslash/pull/35>
  - v0.4.2 was never actually published because of this.

## v0.4.2 (2024-10-11)

- Added MUSL Linux releases: <https://github.com/facebook/dotslash/pull/34>
- Many dependency updates, but key among them is
  [`tar` 0.4.40 to 0.4.42](https://github.com/facebook/dotslash/commit/4ee240e788eaaa7ddad15a835819fb624d1f11f6).

## v0.4.1 (2024-04-10)

- Fixed macos builds
  <https://github.com/facebook/dotslash/commit/25cbf80242d8d51d40ae7284738b376e98cbcc1d>

## v0.4.0 (2024-04-10)

- Added support for `.zip` archives:
  <https://github.com/facebook/dotslash/pull/13>
- Added --fetch subcommand <https://github.com/facebook/dotslash/pull/20>
- Fixed new clippy lints from Rust 1.77
  <https://github.com/facebook/dotslash/commit/018ee4cc189a6e7b05b9e53273f5be3cc7a81fd6>
- Updated various dependencies

## v0.3.0 (2024-03-25)

- Added support for `.tar.xz` archives:
  <https://github.com/facebook/dotslash/pull/12>
- Ensure the root of the artifact directory is read-only on UNIX:
  <https://github.com/facebook/dotslash/commit/10faac39bfaad87d293394c58b777bbbc50224c8>
- `aarch64` Linux added to the set of prebuilt releases (though this did not
  require code changes to DotSlash itself):
  <https://github.com/facebook/dotslash/commit/18f8518b7372f7ab61edcda3b95d434f2cd77247>

## v0.2.0 (2024-02-05)

[Release](https://github.com/facebook/dotslash/releases/tag/v0.2.0) |
[Tag](https://github.com/facebook/dotslash/tree/v0.2.0)

- Apparently the v0.1.0 create published to crates.io inadvertently contained
  the `website/` folder.
  [Added `package.include` in `Cargo.toml` to fix this](https://github.com/facebook/dotslash/commit/10faac39bfaad87d293394c58b777bbbc50224c8)
  and republished as v0.2.0. No other code changes.

## v0.1.0 (2024-02-05)

[Release](https://github.com/facebook/dotslash/releases/tag/v0.1.0) |
[Tag](https://github.com/facebook/dotslash/tree/v0.1.0)

- Initial version built from the first commit in the repo, following the
  [project announcement](https://engineering.fb.com/2024/02/06/developer-tools/dotslash-simplified-executable-deployment/).
