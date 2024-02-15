# Changelog

## Next (TBD)

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
