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
use std::fs;
use std::str;

use tempfile::NamedTempFile;

use crate::common::DotslashTestEnv;
use crate::common::ci;
use crate::common::if_win_else;

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
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_GZ_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
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
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__gz__print_argv
",
        ));
}

#[test]
fn http__xz__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_XZ_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
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
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__xz__print_argv
",
        ));
}

#[test]
fn http__zst__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_ZST_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
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
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__zst__print_argv
",
        ));
}

#[test]
fn http__zip__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_ZIP_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
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
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__zip__print_argv
",
        ));
}

#[test]
fn http__plain__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_PLAIN_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
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
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__plain__print_argv
",
        ));
}

#[test]
fn http__tar_gz__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_TGZ_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
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
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__tar_gz__print_argv
",
        ));
}

#[test]
fn http__tar_xz__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_TAR_XZ_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
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
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__tar_xz__print_argv
",
        ));
}

#[test]
fn http__tar_zst__valid_executable() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction(
            "[ARTIFACT_EXE]",
            "[DOTSLASH_CACHE_DIR]/[PACK_TAR_ZST_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
        )
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
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__tar_zst__print_argv
",
        ));
}

#[test]
fn http__nonexistent_url() {
    DotslashTestEnv::try_new()
        .unwrap()
        .path_redaction("[DOTSLASH_FILE]", "[CURRENT_DIR]/tests/fixtures/http__nonexistent_url")
        .path_redaction("[ARTIFACT_LOCATION]", "[DOTSLASH_CACHE_DIR]/[PACK_TGZ_HTTP_ARCHIVE_CACHE_DIR]")
        .path_redaction("[OUTPUT_FILE]", "[DOTSLASH_CACHE_DIR]/cf/.tmp")
        .redaction(
            "[PROVIDER_URL]",
            "https://github.com/zertosh/dotslash_fixtures/raw/5adea95f2eac6509cad9ca87eb770596a1a21379/fake.tar.gz",
        )
        .dotslash_command()
        .arg("tests/fixtures/http__nonexistent_url")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: problem with `[DOTSLASH_FILE]`
caused by: failed to download artifact into cache `[DOTSLASH_CACHE_DIR]` artifact location `[ARTIFACT_LOCATION]`
caused by: no providers succeeded. warnings:
failed to fetch artifact: failed to fetch `[PROVIDER_URL]`

Caused by:
    0: `curl --location --retry 3 --fail --silent --show-error --user-agent [DOTSLASH_USER_AGENT] [PROVIDER_URL] --output [OUTPUT_FILE][..]`
    1: 404 Not Found
",
        );
}

#[test]
fn http__arg0() -> anyhow::Result<()> {
    let mut test_env = DotslashTestEnv::try_new()?;

    let dotslash_file = "tests/fixtures/http__tar_zst__print_argv";

    let contents = fs::read_to_string(test_env.current_dir().join(dotslash_file))?;

    let tempfile = NamedTempFile::new()?;
    fs::write(
        tempfile.path(),
        contents.replace(
            r#""format": "tar.zst","#,
            r#""format": "tar.zst", "arg0": "underlying-executable","#,
        ),
    )?;

    test_env.path_redaction(
        "[ARTIFACT_EXE]",
        "[DOTSLASH_CACHE_DIR]/[PACK_TAR_ZST_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
    );

    // Default behavior is "dotslash-file".
    test_env
        .dotslash_command()
        .arg(dotslash_file)
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(if_win_else!(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
            "\
exe: [ARTIFACT_EXE]
0: tests/fixtures/http__tar_zst__print_argv
",
        ));

    // Modified "underlying-executable" behavior.
    test_env
        .dotslash_command()
        .arg(tempfile.path())
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(
            "\
exe: [ARTIFACT_EXE]
0: [ARTIFACT_EXE]
",
        );

    Ok(())
}

//
// Commands
//

#[test]
fn command_missing() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "dotslash error: no subcommand passed to '--'

See `dotslash --help` for more information.
",
        );
}

#[test]
fn command_no_match() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("fake")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: unknown subcommand passed to '--': `fake`

See `dotslash --help` for more information.
",
        );
}

#[test]
fn dotslash_file_arg_missing() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq("dotslash error: must specify the path to a DotSlash file\n");
}

#[test]
fn dotslash_file_not_found() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("path/to/fake/file")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: problem with `path/to/fake/file`
caused by: failed to read DotSlash file
caused by: [IO_ERROR_NOT_FOUND]
",
        );
}

#[test]
fn dotslash_file_is_a_directory() {
    DotslashTestEnv::try_new()
        .unwrap()
        .redaction(
            "[IO_ERROR_IS_A_DIRECTORY]",
            if_win_else!(
                "Access is denied. (os error 5)",
                "Is a directory (os error 21)",
            ),
        )
        .path_redaction("[DOTSLASH_FILE]", "[CURRENT_DIR]/tests/fixtures")
        .dotslash_command()
        .arg("tests/fixtures")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: problem with `[DOTSLASH_FILE]`
caused by: failed to read DotSlash file
caused by: [IO_ERROR_IS_A_DIRECTORY]
",
        );
}

#[test]
fn dotslash_file_name_is_non_utf8() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg(non_utf8_osstring())
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: problem with `a/\u{FFFD}/c`
caused by: failed to read DotSlash file
caused by: [IO_ERROR_NOT_FOUND]
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

Your DotSlash cache is: [DOTSLASH_CACHE_DIR]

dotslash also has these special experimental commands:
  dotslash --help                   Print this message
  dotslash --version                Print the version of dotslash
  dotslash -- b3sum FILE            Compute blake3 hash
  dotslash -- clean                 Clean dotslash cache
  dotslash -- create-url-entry URL  Generate \"http\" provider entry
  dotslash -- cache-dir             Print path to the cache directory
  dotslash -- fetch DOTSLASH_FILE   Prepare for execution, but print exe path
                                    instead of executing
  dotslash -- parse DOTSLASH_FILE   Parse the dotslash file
  dotslash -- sha256 FILE           Compute sha256 sum of the file

Learn more at https://dotslash-cli.com
";

#[test]
fn help_command_ok() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("help")
        .assert()
        .code(0)
        .stdout_eq("")
        .stderr_eq(HELP_STDERR);
}

#[test]
fn help_flag_ok() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--help")
        .assert()
        .code(0)
        .stdout_eq("")
        .stderr_eq(HELP_STDERR);
}

#[test]
fn help_command_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("help")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'help' command failed
caused by: expected no arguments but received some
",
        );
}

#[test]
fn help_flag_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--help")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
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
    DotslashTestEnv::try_new()
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
    DotslashTestEnv::try_new()
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
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("version")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'version' command failed
caused by: expected no arguments but received some
",
        );
}

#[test]
fn version_flag_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--version")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
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
    fs::write(tempfile.path(), "DotSlash Rulez!\n")?;

    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("b3sum")
        .arg(tempfile.path())
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq("824ecff042b9ac68a33ea5ee027379a09f3da1d5a47f29fc52072809a204db87\n");

    Ok(())
}

//
// "cache-dir" Command
//

#[test]
fn cache_dir_command_ok() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("cache-dir")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq("[DOTSLASH_CACHE_DIR]\n");
}

#[test]
fn cache_dir_command_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("cache-dir")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
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
fn clean_command_ok() -> anyhow::Result<()> {
    let test_env = DotslashTestEnv::try_new()?;

    let cache_dir = test_env.dotslash_cache();

    // Cache dir exists, but should be empty.
    assert!(cache_dir.exists());
    let is_empty = fs::read_dir(cache_dir)?.next().is_none();
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
        .stderr_eq("Cleaning `[DOTSLASH_CACHE_DIR]`\n");

    assert!(!cache_dir.exists());

    Ok(())
}

#[test]
fn clean_command_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("clean")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
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
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("create-url-entry")
        .arg("https://github.com/zertosh/dotslash_fixtures/raw/462625c6bf2671439dce66bd5bc40b05f2ed8819/pack.tar.gz")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(
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
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("create-url-entry")
        .arg("https://github.com/zertosh/dotslash_fixtures/raw/462625c6bf2671439dce66bd5bc40b05f2ed8819/pack.tar.zst")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(
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
fn fetch_simple() -> anyhow::Result<()> {
    let mut test_env = DotslashTestEnv::try_new()?;
    test_env.path_redaction(
        "[ARTIFACT_EXE]",
        "[DOTSLASH_CACHE_DIR]/[PACK_TGZ_HTTP_ARCHIVE_CACHE_DIR]/subdir/[PRINT_ARGV_EXECUTABLE]",
    );

    let assert = test_env
        .dotslash_command()
        .arg("--")
        .arg("fetch")
        .arg("tests/fixtures/http__tar_gz__print_argv")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq("[ARTIFACT_EXE]\n");

    let artifact = str::from_utf8(&assert.get_output().stdout)?.trim_end();

    let metadata = fs::metadata(artifact)?;
    assert!(metadata.is_file());

    Ok(())
}

//
// "parse" Command
//

#[test]
fn parse_command_ok() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("parse")
        .arg("tests/fixtures/http__dummy_values.in")
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq(ci::snapshot_file("http__dummy_values.out"));
}

#[test]
fn parse_command_extra_args() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("parse")
        .arg("path/to/dotslash_file")
        .arg("foo")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'parse' command failed
caused by: expected exactly one argument but received more
",
        );
}

#[test]
fn parse_command_non_existent_file() {
    DotslashTestEnv::try_new()
        .unwrap()
        .dotslash_command()
        .arg("--")
        .arg("parse")
        .arg("fake/path")
        .assert()
        .code(1)
        .stdout_eq("")
        .stderr_eq(
            "\
dotslash error: 'parse' command failed
caused by: failed to read file `fake/path`
caused by: [IO_ERROR_NOT_FOUND]
",
        );
}

//
// "sha256" Command
//

#[test]
fn sha256_command_ok() -> anyhow::Result<()> {
    let tempfile = NamedTempFile::new()?;
    fs::write(tempfile.path(), "DotSlash Rulez!\n")?;

    DotslashTestEnv::try_new()?
        .dotslash_command()
        .arg("--")
        .arg("sha256")
        .arg(tempfile.path())
        .assert()
        .code(0)
        .stderr_eq("")
        .stdout_eq("52aa28f4f276bdd9a103fcd7f74f97f2bffc52dd816b887952f791b39356b08e\n");

    Ok(())
}
