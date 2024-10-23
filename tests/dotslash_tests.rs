/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

#![allow(non_snake_case)]

mod common;

use std::ffi::OsString;

use tempfile::NamedTempFile;

use crate::common::ci;
use crate::common::DotSlashTestEnv;

#[cfg(unix)]
fn non_utf8_osstring() -> OsString {
    std::os::unix::ffi::OsStringExt::from_vec(b"a/\xc3/c".to_vec())
}

#[cfg(windows)]
fn non_utf8_osstring() -> OsString {
    // Anything between 0xD800–0xDBFF will end up producing a UTF-16
    // character that can't be converted to UTF-8.
    // https://en.wikipedia.org/wiki/UTF-16#Code_points_from_U+010000_to_U+10FFFF
    const UTF16_SURROGATE_RANGE_START: u16 = 0xD800;
    std::os::windows::ffi::OsStringExt::from_wide(&[
        u16::from(b'a'),
        u16::from(b'/'),
        UTF16_SURROGATE_RANGE_START,
        u16::from(b'/'),
        u16::from(b'c'),
    ])
}

#[test]
fn test_non_utf8_osstring() {
    assert_eq!(non_utf8_osstring().to_string_lossy(), "a/\u{FFFD}/c");
}

//
// Providers
//

#[test]
fn http__gz__valid_executable() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("tests/fixtures/http__gz__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_matches(format!(
            "\
exe: [DOTSLASHCACHEDIR]/[PACKGZHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]
0: {argv0}
",
            argv0 = if cfg!(windows) {
                "[DOTSLASHCACHEDIR]/[PACKGZHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]"
            } else {
                "tests/fixtures/http__gz__print_argv"
            }
        ));
}

#[test]
fn http__xz__valid_executable() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("tests/fixtures/http__xz__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_matches(format!(
            "\
exe: [DOTSLASHCACHEDIR]/[PACKXZHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]
0: {argv0}
",
            argv0 = if cfg!(windows) {
                "[DOTSLASHCACHEDIR]/[PACKXZHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]"
            } else {
                "tests/fixtures/http__xz__print_argv"
            }
        ));
}

#[test]
fn http__zst__valid_executable() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("tests/fixtures/http__zst__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_matches(format!(
            "\
exe: [DOTSLASHCACHEDIR]/[PACKZSTHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]
0: {argv0}
",
            argv0 = if cfg!(windows) {
                "[DOTSLASHCACHEDIR]/[PACKZSTHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]"
            } else {
                "tests/fixtures/http__zst__print_argv"
            }
        ));
}

#[test]
fn http__zip__valid_executable() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("tests/fixtures/http__zip__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_matches(format!(
            "\
exe: [DOTSLASHCACHEDIR]/[PACKZIPTHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]
0: {argv0}
",
            argv0 = if cfg!(windows) {
                "[DOTSLASHCACHEDIR]/[PACKZIPTHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]"
            } else {
                "tests/fixtures/http__zip__print_argv"
            }
        ));
}

#[test]
fn http__plain__valid_executable() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("tests/fixtures/http__plain__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_matches(format!(
            "\
exe: [DOTSLASHCACHEDIR]/[PACKPLAINHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]
0: {argv0}
",
            argv0 = if cfg!(windows) {
                "[DOTSLASHCACHEDIR]/[PACKPLAINHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]"
            } else {
                "tests/fixtures/http__plain__print_argv"
            }
        ));
}

#[test]
fn http__tar_gz__valid_executable() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("tests/fixtures/http__tar_gz__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_matches(format!(
            "\
exe: [DOTSLASHCACHEDIR]/[PACKTGZHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]
0: {argv0}
",
            argv0 = if cfg!(windows) {
                "[DOTSLASHCACHEDIR]/[PACKTGZHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]"
            } else {
                "tests/fixtures/http__tar_gz__print_argv"
            }
        ));
}

#[test]
fn http__tar_xz__valid_executable() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("tests/fixtures/http__tar_xz__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_matches(format!(
            "\
exe: [DOTSLASHCACHEDIR]/[PACKTARXZHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]
0: {argv0}
",
            argv0 = if cfg!(windows) {
                "[DOTSLASHCACHEDIR]/[PACKTARXZHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]"
            } else {
                "tests/fixtures/http__tar_xz__print_argv"
            }
        ));
}

#[test]
fn http__tar_zst__valid_executable() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("tests/fixtures/http__tar_zst__print_argv")
        .arg("abc")
        .arg("def")
        .assert()
        .code(0)
        .stderr_eq(
            "\
1: abc
2: def
",
        )
        .stdout_matches(format!(
            "\
exe: [DOTSLASHCACHEDIR]/[PACKTARZSTHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]
0: {argv0}
",
            argv0 = if cfg!(windows) {
                "[DOTSLASHCACHEDIR]/[PACKTARZSTHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]"
            } else {
                "tests/fixtures/http__tar_zst__print_argv"
            }
        ));
}

#[test]
fn http__nonexistent_url() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .substitution(
            "[PROVIDERURL]",
            "https://github.com/zertosh/dotslash_fixtures/raw/5adea95f2eac6509cad9ca87eb770596a1a21379/fake.tar.gz",
        )
        .unwrap()
        .dotslash_command()
        .arg("tests/fixtures/http__nonexistent_url")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "\
dotslash error: problem with `[CURRENTDIR]/tests/fixtures/http__nonexistent_url`
caused by: failed to download artifact into cache `[DOTSLASHCACHEDIR]` artifact location `[DOTSLASHCACHEDIR]/[PACKTGZHTTPARCHIVECACHEDIR]`
caused by: no providers succeeded. warnings:
failed to fetch artifact: failed to fetch `[PROVIDERURL]`

Caused by:
    0: `curl --location --retry 3 --fail --silent --show-error --user-agent [DOTSLASHUSERAGENT] [PROVIDERURL] --output [DOTSLASHCACHEDIR]/cf/.tmp[..]`
    1: 404 Not Found
",
        );
}

//
// Commands
//

#[test]
fn command_missing() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "dotslash error: no subcommand passed to '--'

See `dotslash --help` for more information.
",
        );
}

#[test]
fn command_no_match() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("fake")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "\
dotslash error: unknown subcommand passed to '--': `fake`

See `dotslash --help` for more information.
",
        );
}

#[test]
fn dotslash_file_arg_missing() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches("dotslash error: must specify the path to a DotSlash file\n");
}

#[test]
fn dotslash_file_not_found() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("path/to/fake/file")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "\
dotslash error: problem with `path/to/fake/file`
caused by: failed to read DotSlash file
caused by: [IOERRORNOTFOUND]
",
        );
}

#[test]
fn dotslash_file_is_a_directory() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .substitution(
            "[IOERROR]",
            if cfg!(windows) {
                "Access is denied. (os error 5)"
            } else {
                "Is a directory (os error 21)"
            },
        )
        .unwrap()
        .dotslash_command()
        .arg("tests/fixtures")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "\
dotslash error: problem with `[CURRENTDIR]/tests/fixtures`
caused by: failed to read DotSlash file
caused by: [IOERROR]
",
        );
}

#[test]
fn dotslash_file_name_is_non_utf8() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg(non_utf8_osstring())
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "\
dotslash error: problem with `a/\u{FFFD}/c`
caused by: failed to read DotSlash file
caused by: [IOERRORNOTFOUND]
",
        );
}

//
// "help" Command
//

const HELP_STDERR: &str = "\
usage: dotslash DOTSLASH_FILE [OPTIONS]

DOTSLASH_FILE must be a file that starts with `#!/usr/bin/env dotslash`
and contains a JSON body tells DotSlash how to fetch and run the executable
that DOTSLASH_FILE represents.

All OPTIONS will be forwarded directly to the executable identified by
DOTSLASH_FILE.

Supported platform: [..]

Your DotSlash cache is: [DOTSLASHCACHEDIR]

Learn more at https://dotslash-cli.com
";

#[test]
fn help_command_ok() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("help")
        .assert()
        .code(0)
        .stdout_eq("")
        .stderr_matches(HELP_STDERR);
}

#[test]
fn help_flag_ok() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--help")
        .assert()
        .code(0)
        .stdout_eq("")
        .stderr_matches(HELP_STDERR);
}

#[test]
fn help_command_extra_args() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("help")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "\
dotslash error: 'help' command failed
caused by: expected no arguments but received some
",
        );
}

#[test]
fn help_flag_extra_args() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--help")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "\
dotslash error: 'help' command failed
caused by: expected no arguments but received some
",
        );
}

//
// "version" Command
//

const VERSION_STDOUT: &str = concat!("DotSlash ", env!("CARGO_PKG_VERSION"), "\n");

#[test]
fn version_command_ok() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("version")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(VERSION_STDOUT);
}

#[test]
fn version_flag_ok() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--version")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(VERSION_STDOUT);
}

#[test]
fn version_command_extra_args() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("version")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "\
dotslash error: 'version' command failed
caused by: expected no arguments but received some
",
        );
}

#[test]
fn version_flag_extra_args() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--version")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "\
dotslash error: 'version' command failed
caused by: expected no arguments but received some
",
        );
}

//
// "b3sum" Command
//

#[test]
fn b3sum_command_ok() -> anyhow::Result<()> {
    let tempfile = NamedTempFile::new()?;
    std::fs::write(tempfile.path(), "DotSlash Rulez!\n")?;

    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("b3sum")
        .arg(tempfile.path())
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_matches("824ecff042b9ac68a33ea5ee027379a09f3da1d5a47f29fc52072809a204db87\n");
    Ok(())
}

//
// "cache-dir" Command
//

#[test]
fn cache_dir_command_ok() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("cache-dir")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_matches(
            "\
[DOTSLASHCACHEDIR]
",
        );
}

#[test]
fn cache_dir_command_extra_args() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("cache-dir")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "\
dotslash error: 'cache-dir' command failed
caused by: expected no arguments but received some
",
        );
}

//
// "clean" Command
//

#[test]
fn clean_command_ok() {
    let test_env = DotSlashTestEnv::try_new().unwrap();

    let cache_dir = test_env.dotslash_cache();

    // Cache dir exists, but should be empty.
    assert!(cache_dir.exists());
    let is_empty = std::fs::read_dir(cache_dir).unwrap().next().is_none();
    assert!(is_empty);

    test_env
        .dotslash_command()
        .arg("tests/fixtures/http__tar_gz__print_argv")
        .assert()
        .success();

    assert!(cache_dir.exists());

    test_env
        .dotslash_command()
        .arg("--")
        .arg("clean")
        .assert()
        .code(0)
        .stdout_eq("")
        .stderr_matches("Cleaning `[DOTSLASHCACHEDIR]`\n");

    assert!(!cache_dir.exists());
}

#[test]
fn clean_command_extra_args() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("clean")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "\
dotslash error: 'clean' command failed
caused by: expected no arguments but received some
",
        );
}

//
// "create-url-entry" Command
//

#[test]
fn create_url_entry_tar_gz() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("create-url-entry")
        .arg("https://github.com/zertosh/dotslash_fixtures/raw/462625c6bf2671439dce66bd5bc40b05f2ed8819/pack.tar.gz")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_matches(
            r#"{
  "size": 48689,
  "hash": "blake3",
  "digest": "068464830bd5c276e085a4eab5ef9cc57159f94273db296d6a638e49b78ca55f",
  "format": "tar.gz",
  "path": "TODO: specify the appropriate `path` for this artifact",
  "providers": [
    {
      "url": "https://github.com/zertosh/dotslash_fixtures/raw/462625c6bf2671439dce66bd5bc40b05f2ed8819/pack.tar.gz"
    }
  ]
}
"#,
        );
}

#[test]
fn create_url_entry_tar_zst() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("create-url-entry")
        .arg("https://github.com/zertosh/dotslash_fixtures/raw/462625c6bf2671439dce66bd5bc40b05f2ed8819/pack.tar.zst")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_matches(
            r#"{
  "size": 38886,
  "hash": "blake3",
  "digest": "2dedd1652985d33607d1eff62fe9f85f7f53dfce9902cb5de8a8c680cba48081",
  "format": "tar.zst",
  "path": "TODO: specify the appropriate `path` for this artifact",
  "providers": [
    {
      "url": "https://github.com/zertosh/dotslash_fixtures/raw/462625c6bf2671439dce66bd5bc40b05f2ed8819/pack.tar.zst"
    }
  ]
}
"#,
        );
}

//
// "fetch" Command
//

#[test]
fn fetch_simple() {
    let test_env = DotSlashTestEnv::try_new().unwrap();

    let assert = test_env
        .dotslash_command()
        .arg("--")
        .arg("fetch")
        .arg("tests/fixtures/http__tar_gz__print_argv")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_matches(
            "[DOTSLASHCACHEDIR]/[PACKTGZHTTPARCHIVECACHEDIR]/subdir/[PRINTARGVEXECUTABLE]\n",
        );

    let artifact = std::str::from_utf8(&assert.get_output().stdout)
        .unwrap()
        .trim_end();

    let metadata = std::fs::metadata(artifact).unwrap();
    assert!(metadata.is_file());
}

//
// "parse" Command
//

#[test]
fn parse_command_ok() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("parse")
        .arg("tests/fixtures/http__dummy_values.in")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_matches_path(ci::snapshot_path("http__dummy_values.out"));
}

#[test]
fn parse_command_extra_args() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("parse")
        .arg("path/to/dotslash_file")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "\
dotslash error: 'parse' command failed
caused by: expected exactly one argument but received more
",
        );
}

#[test]
fn parse_command_non_existent_file() {
    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("parse")
        .arg("fake/path")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_matches(
            "\
dotslash error: 'parse' command failed
caused by: failed to read file `fake/path`
caused by: [IOERRORNOTFOUND]
",
        );
}

//
// "sha256" Command
//

#[test]
fn sha256_command_ok() -> anyhow::Result<()> {
    let tempfile = NamedTempFile::new()?;
    std::fs::write(tempfile.path(), "DotSlash Rulez!\n")?;

    DotSlashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("sha256")
        .arg(tempfile.path())
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_matches("52aa28f4f276bdd9a103fcd7f74f97f2bffc52dd816b887952f791b39356b08e\n");
    Ok(())
}
