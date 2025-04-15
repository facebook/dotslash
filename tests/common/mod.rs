/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context as _;
use snapbox::Assert;
use snapbox::RedactedValue;
use snapbox::Redactions;
use snapbox::cmd::Command;
use tempfile::TempDir;

#[cfg_attr(fbcode_build, path = "fb/ci.rs")]
pub mod ci;

#[path = "../../src/platform.rs"]
#[expect(dead_code)]
mod platform;

macro_rules! if_win_else {
    ($windows:expr, $not_windows:expr $(,)?) => {
        if cfg!(windows) {
            $windows
        } else {
            $not_windows
        }
    };
}

pub(crate) use if_win_else;

const PACK_TGZ_HTTP_ARCHIVE_CACHE_DIR: &str = "cf/df86e55cbbd2455fd1e36468c2b1ff7f8998d4";

const PACK_TAR_XZ_HTTP_ARCHIVE_CACHE_DIR: &str = "a2/7e95fba3d48794eb41dd0f63634e508ca72621";

const PACK_TAR_ZST_HTTP_ARCHIVE_CACHE_DIR: &str = "ef/ca1937daf58b9c65c54cc9a360450fe5d43835";

const PACK_ZIP_HTTP_ARCHIVE_CACHE_DIR: &str = "04/bcb0761a2e4d35c9c9c15b14e8e5d1f2a29d80";

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
    linux_aarch64 = "bb/1a78fe5c9fb5a4efd2665d2385853c86733822",
    linux_x86_64 = "15/8f0652f1ef9276f3c1aa647a2f1ac9f28dde32",
    macos_aarch64 = "13/676cc4457e6bcc0344c0a823701d4ec2337432",
    macos_x86_64 = "f4/b9233113f64ded6bf42f0e5c4236ef31552733",
    windows_aarch64 = "f1/a292555df0a08062bc4632c3d363362491aec2",
    windows_x86_64 = "b3/c130f6ab98388b2ef7b14dd8f833e96cc4515f",
};

const PACK_XZ_HTTP_ARCHIVE_CACHE_DIR: &str = platform::if_platform! {
    linux_aarch64 = "f8/379be6815479ec5466d1d8d0b064e2344b22d2",
    linux_x86_64 = "26/5d340b2c556a7098aaa82732991bb3f820aa36",
    macos_aarch64 = "8a/60f78296c051ed9f020cef2df4a41ed76f5cd9",
    macos_x86_64 = "d6/e288c5470cd5bcba63f1734a3780a1dca53d84",
    windows_aarch64 = "3b/85c4c3eae3a023918fe1f2aec2926191e6b7d1",
    windows_x86_64 = "b8/607feb9b08082efd12b02d075ba3ae671cd396",
};

const PACK_ZST_HTTP_ARCHIVE_CACHE_DIR: &str = platform::if_platform! {
    linux_aarch64 = "88/04463f5ce3ef56616faeddbc9936ae0ed1c2a1",
    linux_x86_64 = "0d/dcd334203082641b987ebd46a758bb032d8961",
    macos_aarch64 = "68/2ea5eacaf1881b15876f63b5aba2437ddf4b0d",
    macos_x86_64 = "26/13225b385a72a4656eac271304b14b9dbf689c",
    windows_aarch64 = "d1/8ebd85d8164c4adc44073d3e07843c874a74dd",
    windows_x86_64 = "b3/6c9ca1cfc1c38f8300761ca8b893f7911e053b",
};

const PACK_PLAIN_HTTP_ARCHIVE_CACHE_DIR: &str = platform::if_platform! {
    linux_aarch64 = "07/2c58ff3a1560e08b300834964fc8ee60af5aab",
    linux_x86_64 = "e9/6c95f9ab97c0175a4a86b0da31c98f8e7b1d6f",
    macos_aarch64 = "c5/4434b1de7f5718a3acd3e2e04924c9abc2fccf",
    macos_x86_64 = "2b/7c3edc2287dfd5cf2f0b772e92d89dd226ba7e",
    windows_aarch64 = "8a/26506e13829c5263dac097d73013921bc8301e",
    windows_x86_64 = "19/0d26175e50e486f87d90ca9d10ac50e3c248fd",
};

const IO_ERROR_NOT_FOUND: &str = if_win_else!(
    "The system cannot find the path specified. (os error 3)",
    "No such file or directory (os error 2)",
);

#[derive(Debug)]
struct SuperRedactions {
    redactions: Redactions,
    literal: HashMap<&'static str, String>,
}

impl SuperRedactions {
    fn new() -> Self {
        Self {
            redactions: Redactions::new(),
            literal: HashMap::new(),
        }
    }

    fn redaction(
        &mut self,
        key: &'static str,
        value: impl Into<RedactedValue>,
    ) -> snapbox::assert::Result<&mut Self> {
        self.redactions.insert(key, value)?;
        Ok(self)
    }

    /// Normalizes path separators to the platform native separator,
    /// and expands nested path redactions.
    fn path_redaction(
        &mut self,
        key: &'static str,
        value: impl AsRef<Path>,
    ) -> snapbox::assert::Result<&mut Self> {
        let value = value
            .as_ref()
            .components()
            .map(|comp| {
                (|| {
                    let k = comp.as_os_str().to_str()?;
                    let v = self.literal.get(k)?.as_ref();
                    Some(v)
                })()
                .unwrap_or(comp.as_os_str())
            })
            .collect::<PathBuf>()
            .to_string_lossy()
            .into_owned();

        self.redactions.insert(key, value.clone())?;
        self.literal.insert(key, value);

        Ok(self)
    }
}

pub struct DotslashTestEnv {
    current_dir: PathBuf,
    redactions: SuperRedactions,
    tempdir_path: PathBuf,
    _tempdir: TempDir,
}

impl DotslashTestEnv {
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

        let mut redactions = SuperRedactions::new();
        redactions.path_redaction("[DOTSLASH_CACHE_DIR]", tempdir_str)?;
        redactions.path_redaction("[CURRENT_DIR]", current_dir_str)?;
        redactions.path_redaction(
            "[PACK_TGZ_HTTP_ARCHIVE_CACHE_DIR]",
            PACK_TGZ_HTTP_ARCHIVE_CACHE_DIR,
        )?;
        redactions.path_redaction(
            "[PACK_TAR_XZ_HTTP_ARCHIVE_CACHE_DIR]",
            PACK_TAR_XZ_HTTP_ARCHIVE_CACHE_DIR,
        )?;
        redactions.path_redaction(
            "[PACK_TAR_ZST_HTTP_ARCHIVE_CACHE_DIR]",
            PACK_TAR_ZST_HTTP_ARCHIVE_CACHE_DIR,
        )?;
        redactions.path_redaction(
            "[PACK_ZIP_HTTP_ARCHIVE_CACHE_DIR]",
            PACK_ZIP_HTTP_ARCHIVE_CACHE_DIR,
        )?;
        redactions.path_redaction(
            "[PACK_GZ_HTTP_ARCHIVE_CACHE_DIR]",
            PACK_GZ_HTTP_ARCHIVE_CACHE_DIR,
        )?;
        redactions.path_redaction(
            "[PACK_XZ_HTTP_ARCHIVE_CACHE_DIR]",
            PACK_XZ_HTTP_ARCHIVE_CACHE_DIR,
        )?;
        redactions.path_redaction(
            "[PACK_ZST_HTTP_ARCHIVE_CACHE_DIR]",
            PACK_ZST_HTTP_ARCHIVE_CACHE_DIR,
        )?;
        redactions.path_redaction(
            "[PACK_PLAIN_HTTP_ARCHIVE_CACHE_DIR]",
            PACK_PLAIN_HTTP_ARCHIVE_CACHE_DIR,
        )?;
        redactions.path_redaction("[PRINT_ARGV_EXECUTABLE]", PRINT_ARGV_EXECUTABLE)?;
        redactions.redaction("[IO_ERROR_NOT_FOUND]", IO_ERROR_NOT_FOUND)?;
        redactions.redaction("[DOTSLASH_USER_AGENT]", USER_AGENT)?;

        Ok(DotslashTestEnv {
            current_dir,
            redactions,
            tempdir_path,
            _tempdir: tempdir,
        })
    }

    pub fn redaction(&mut self, key: &'static str, value: impl Into<RedactedValue>) -> &mut Self {
        self.redactions.redaction(key, value).unwrap();
        self
    }

    pub fn path_redaction(&mut self, key: &'static str, value: impl AsRef<Path>) -> &mut Self {
        self.redactions.path_redaction(key, value.as_ref()).unwrap();
        self
    }

    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }

    pub fn dotslash_cache(&self) -> &Path {
        &self.tempdir_path
    }

    pub fn dotslash_command(&self) -> Command {
        let assert = Assert::new()
            .normalize_paths(false)
            .redact_with(self.redactions.redactions.clone());
        Command::new(ci::dotslash_bin())
            .current_dir(&self.current_dir)
            .env("DOTSLASH_CACHE", &self.tempdir_path)
            .envs(ci::envs())
            .with_assert(assert)
    }
}

impl Drop for DotslashTestEnv {
    fn drop(&mut self) {
        // Make it possible to delete the temp cache.
        let _ = make_tree_entries_writable(&self.tempdir_path);
    }
}

fn make_tree_entries_writable(folder: &Path) -> io::Result<()> {
    for entry in fs::read_dir(folder)? {
        let path = entry?.path();
        let metadata = fs::symlink_metadata(&path)?;

        if metadata.is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            make_tree_entries_writable(&path)?;
        }

        let mut perms = metadata.permissions();
        if perms.readonly() {
            #[expect(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            fs::set_permissions(path, perms)?;
        }
    }

    Ok(())
}
