# Changelog

## v0.5.8 (2025-08-19)

- DotSlash is now available as an
  [npm package](https://www.npmjs.com/package/fb-dotslash):
  <https://github.com/facebook/dotslash/pull/74>
- Optimized the size of the
  [DotSlash Windows shim](https://dotslash-cli.com/docs/windows/#dotslash-windows-shim):
  <https://github.com/facebook/dotslash/pull/46>

## v0.5.7 (2025-07-09)

- Fix release pipeline for ARM64 Windows:
  <https://github.com/facebook/dotslash/commit/96964d54609611fba87764e3697ad8a9ff3268f3>
  - v0.5.6 was never published because of this.

## v0.5.6 (2025-07-08)

- [Fixed a bug](https://github.com/facebook/dotslash/pull/75) where DotSlash
  would sometimes write corrupted zip files to its cache
- One-liner installations are now possible again, see
  [the new installation instructions](https://dotslash-cli.com/docs/installation/#prebuilt-binaries)
- [ARM64 Windows binaries](https://github.com/facebook/dotslash/pull/76) for
  DotSlash are available

## v0.5.5 (2025-06-25)

- Added support for
  [provider order randomization](https://github.com/facebook/dotslash/pull/49)
- Added support for
  [.bz2 and .tar.bz2](https://github.com/facebook/dotslash/pull/53)

Additionally, as of this release we are now attaching binaries to releases that
don't have the release version in the filename. These files are in addition to
the files that mention the version number for backwards compatibility with the
`install-dotslash` action. WARNING: We will be removing the legacy versioned
filenames in a future release, follow
[this issue](https://github.com/facebook/dotslash/issues/68).

## v0.5.4 (2025-05-19)

- Reverted "One-liner installations are now possible, see
  [the new installation instructions](https://dotslash-cli.com/docs/installation/#prebuilt-binaries)
  <https://github.com/facebook/dotslash/pull/17>"

## v0.5.3 (2025-05-19)

- One-liner installations are now possible, see
  [the new installation instructions](https://dotslash-cli.com/docs/installation/#prebuilt-binaries)
  <https://github.com/facebook/dotslash/pull/17>
- Precompiled arch-specific binaries are now available for macOS:
  <https://github.com/facebook/dotslash/pull/48>

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
