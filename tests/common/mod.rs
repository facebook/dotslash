/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use snapbox::cmd::Command;
use snapbox::Substitutions;

#[cfg_attr(fbcode_build, path = "fb/ci.rs")]
pub mod ci;

#[path = "../../src/platform.rs"]
#[allow(dead_code)]
mod platform;

const PACK_TGZ_HTTP_ARCHIVE_CACHE_DIR: &str = "dd/c8a0044304752905aef0fc98a3d2e268afef9a";

const PACK_TAR_ZST_HTTP_ARCHIVE_CACHE_DIR: &str = "4b/6bdee10eb2b03814b531e571a243038039ab11";

const USER_AGENT: &str = concat!(
    "Mozilla/5.0 (compatible; DotSlash/",
    env!("CARGO_PKG_VERSION"),
    "; +https://dotslash-cli.com)"
);

const PRINT_ARGV_EXECUTABLE: &str = platform::if_platform! {
    linux_aarch64 = "print_argv.linux.aarch64",
    linux_x86_64 = "print_argv.linux.x86_64",
    macos_aarch64 = "print_argv.macos.aarch64",
    macos_x86_64 = "print_argv.macos.x86_64",
    windows_aarch64 = "print_argv.windows.aarch64.exe",
    windows_x86_64 = "print_argv.windows.x86_64.exe",
};

const PACK_GZ_HTTP_ARCHIVE_CACHE_DIR: &str = platform::if_platform! {
    linux_aarch64 = "58/e7b993168624e049f272fe8c9ec54398534d75",
    linux_x86_64 = "6a/b7f31e1d66c1216003e479ea9d28c86782be7c",
    macos_aarch64 = "24/e4881bc26131018e51d1efc98ce0cab149cc80",
    macos_x86_64 = "76/adaeac4a4c13b02c7fc55dda4e5d923f8d4748",
    windows_aarch64 = "ec/03a4255d20937d3b1c92a24746a116d9c302ac",
    windows_x86_64 = "a8/60d484c31d9febb3cf2a235c802534b15cc3e9",
};

const PACK_ZST_HTTP_ARCHIVE_CACHE_DIR: &str = platform::if_platform! {
    linux_aarch64 = "0b/69b2f01fa4a00fb37c5cd2ae7302994f8f2d0a",
    linux_x86_64 = "4c/9e6ee9f62c1c85199c943a2d3085f253b79751",
    macos_aarch64 = "32/58bdcd01309061223a73d29d52f72327f77e40",
    macos_x86_64 = "8d/8956ec0adf9d086d00653621f8ed385310d4eb",
    windows_aarch64 = "30/44fca3537a5e2fb7bc9f95425c7c8f7b5e0834",
    windows_x86_64 = "30/d20eeb602cb2fafafcb47a1565d163089cbb40",
};

const PACK_PLAIN_HTTP_ARCHIVE_CACHE_DIR: &str = platform::if_platform! {
    linux_aarch64 = "c5/6831ffb9352968343b4a3db3edc52c50e2a24a",
    linux_x86_64 = "b3/a8125fdb92bc86d2429ece315e535467288653",
    macos_aarch64 = "40/1f007e7e2575be3d0a218e6c23b350f78c4a53",
    macos_x86_64 = "94/7aa11cf9d77f3983d8704fe42b5af20faf604c",
    windows_aarch64 = "11/3e1afef6d6dbd9d5957686f697eedaeda4de46",
    windows_x86_64 = "f8/c90362327cce3efc60ea82a121b205235b8cc9",
};

const IO_ERROR_NOT_FOUND: &str = if cfg!(windows) {
    "The system cannot find the path specified. (os error 3)"
} else {
    "No such file or directory (os error 2)"
};

pub struct DotSlashTestEnv {
    current_dir: PathBuf,
    substitutions: Substitutions,
    tempdir_path: PathBuf,
    _tempdir: tempfile::TempDir,
}

impl DotSlashTestEnv {
    pub fn try_new() -> anyhow::Result<Self> {
        let tempdir = tempfile::Builder::new()
            .prefix("dotslash_tests-")
            .rand_bytes(5)
            .tempdir()?;

        let tempdir_path =
            dunce::canonicalize(tempdir.path()).context("failed to canonicalize tempdir")?;

        let tempdir_str = tempdir_path
            .as_path()
            .as_os_str()
            .to_str()
            .context("tempdir is not UTF-8")?
            .to_owned();

        let current_dir = ci::current_dir().context("failed to get current dir")?;

        let current_dir_str = current_dir
            .as_os_str()
            .to_str()
            .context("current_dir is not UTF-8")?
            .to_owned();

        let mut substitutions = snapbox::Substitutions::new();
        substitutions.insert("[DOTSLASHCACHEDIR]", tempdir_str)?;
        substitutions.insert("[CURRENTDIR]", current_dir_str)?;
        substitutions.insert(
            "[PACKTGZHTTPARCHIVECACHEDIR]",
            PACK_TGZ_HTTP_ARCHIVE_CACHE_DIR,
        )?;
        substitutions.insert(
            "[PACKTARZSTHTTPARCHIVECACHEDIR]",
            PACK_TAR_ZST_HTTP_ARCHIVE_CACHE_DIR,
        )?;
        substitutions.insert(
            "[PACKGZHTTPARCHIVECACHEDIR]",
            PACK_GZ_HTTP_ARCHIVE_CACHE_DIR,
        )?;
        substitutions.insert(
            "[PACKZSTHTTPARCHIVECACHEDIR]",
            PACK_ZST_HTTP_ARCHIVE_CACHE_DIR,
        )?;
        substitutions.insert(
            "[PACKPLAINHTTPARCHIVECACHEDIR]",
            PACK_PLAIN_HTTP_ARCHIVE_CACHE_DIR,
        )?;
        substitutions.insert("[PRINTARGVEXECUTABLE]", PRINT_ARGV_EXECUTABLE)?;
        substitutions.insert("[IOERRORNOTFOUND]", IO_ERROR_NOT_FOUND)?;
        substitutions.insert("[DOTSLASHUSERAGENT]", USER_AGENT)?;

        Ok(DotSlashTestEnv {
            current_dir,
            substitutions,
            tempdir_path,
            _tempdir: tempdir,
        })
    }

    pub fn substitution(
        &mut self,
        key: &'static str,
        value: impl Into<Cow<'static, str>>,
    ) -> Result<&mut Self, snapbox::Error> {
        self.substitutions.insert(key, value)?;
        Ok(self)
    }

    pub fn dotslash_cache(&self) -> &Path {
        &self.tempdir_path
    }

    pub fn dotslash_command(&self) -> Command {
        let assert = snapbox::Assert::new().substitutions(self.substitutions.clone());
        Command::new(ci::dotslash_bin())
            .current_dir(&self.current_dir)
            .env("DOTSLASH_CACHE", &self.tempdir_path)
            .envs(ci::envs())
            .with_assert(assert)
    }
}
