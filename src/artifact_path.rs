/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::fmt;
use std::path::Component;
use std::path::Path;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

/// `ArtifactPath` is a newtype type for `String` rather than `PathBuf` because
/// we want it to be unambiguously represented with forward slashes on all
/// platforms.
#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(try_from = "String")]
pub struct ArtifactPath(String);

impl fmt::Display for ArtifactPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for ArtifactPath {
    type Error = ArtifactInvalidPathError;

    fn try_from(path: String) -> Result<Self, Self::Error> {
        Self::from_str(&path)
    }
}

impl FromStr for ArtifactPath {
    type Err = ArtifactInvalidPathError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _ = normalize_as_relative_unix_path(s)?;
        Ok(ArtifactPath(s.to_owned()))
    }
}

impl ArtifactPath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Verifies that `s` is a normalized, relative UNIX path that is accepted by
/// DotSlash (i.e., does not contain backslashes or null bytes). If `s`
/// satisfies the requirements, returns `Ok(n)`, where `n` is the number of
/// components in `s`; otherwise, returns an error.
fn normalize_as_relative_unix_path(s: &str) -> anyhow::Result<usize, ArtifactInvalidPathError> {
    if s.is_empty() {
        return Err(ArtifactInvalidPathError::EmptyPath);
    }

    let mut bytes = s.as_bytes();
    match bytes.iter().find(|c| matches!(c, b'\\' | b'\0')) {
        Some(b'\\') => return Err(ArtifactInvalidPathError::ContainsBackslash(s.to_owned())),
        Some(b'\0') => return Err(ArtifactInvalidPathError::ContainsNull(s.to_owned())),
        Some(byte) => panic!("unexpected match {}", byte),
        None => {}
    }

    let mut it = Path::new(s).components().peekable();
    let mut num_components = 0;

    while let Some(comp) = it.next() {
        match comp {
            Component::Normal(c) => {
                if let Some(rest) = bytes.strip_prefix(c.as_encoded_bytes()) {
                    bytes = rest;
                    num_components += 1;
                } else {
                    return Err(ArtifactInvalidPathError::NormalizedPath(s.to_owned()));
                }
                if it.peek().is_some() {
                    if let Some(rest) = bytes.strip_prefix(b"/") {
                        bytes = rest;
                    } else {
                        return Err(ArtifactInvalidPathError::NormalizedPath(s.to_owned()));
                    }
                }
            }
            Component::Prefix(_) => {
                return Err(ArtifactInvalidPathError::PrefixComponent(s.to_owned()));
            }
            Component::RootDir => {
                return Err(ArtifactInvalidPathError::RootDirComponent(s.to_owned()));
            }
            Component::CurDir => {
                return Err(ArtifactInvalidPathError::CurDirComponent(s.to_owned()));
            }
            Component::ParentDir => {
                return Err(ArtifactInvalidPathError::ParentDirComponent(s.to_owned()));
            }
        }
    }

    if bytes.is_empty() {
        Ok(num_components)
    } else {
        Err(ArtifactInvalidPathError::NormalizedPath(s.to_owned()))
    }
}

#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ArtifactInvalidPathError {
    #[error("path cannot be the empty string")]
    EmptyPath,

    #[error("path cannot contain backslash: `{0}`")]
    ContainsBackslash(String),

    #[error("path cannot contain NUL: `{0}`")]
    ContainsNull(String),

    #[error("path must be relative and normalized using '/' as separator: `{0}`")]
    NormalizedPath(String),

    #[error("path cannot contain prefix component: `{0}`")]
    PrefixComponent(String),

    #[error("path cannot contain root directory component: `{0}`")]
    RootDirComponent(String),

    #[error("path cannot contain current directory component: `{0}`")]
    CurDirComponent(String),

    #[error("path cannot contain parent directory component: `{0}`")]
    ParentDirComponent(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_as_relative_path() {
        assert_eq!(normalize_as_relative_unix_path("foo"), Ok(1));
    }

    #[test]
    fn normalized_relative_path() {
        assert_eq!(normalize_as_relative_unix_path("foo/bar"), Ok(2));
        assert_eq!(normalize_as_relative_unix_path("foo/bar/baz"), Ok(3));
    }

    #[test]
    fn filename_with_null_byte() {
        assert_eq!(
            normalize_as_relative_unix_path("foo\0"),
            Err(ArtifactInvalidPathError::ContainsNull("foo\0".to_owned())),
        );
    }

    #[test]
    fn path_with_superfluous_slash() {
        assert_eq!(
            normalize_as_relative_unix_path("foo//bar"),
            Err(ArtifactInvalidPathError::NormalizedPath(
                "foo//bar".to_owned()
            )),
        );
    }

    #[test]
    fn path_with_trailing_slash() {
        assert_eq!(
            normalize_as_relative_unix_path("foo/bar/"),
            Err(ArtifactInvalidPathError::NormalizedPath(
                "foo/bar/".to_owned()
            )),
        );
    }

    #[test]
    fn path_starting_with_cur_dir_component() {
        assert_eq!(
            normalize_as_relative_unix_path("./foo/bar"),
            Err(ArtifactInvalidPathError::CurDirComponent(
                "./foo/bar".to_owned()
            )),
        );
    }

    #[test]
    fn path_with_cur_dir_component_in_middle() {
        assert_eq!(
            normalize_as_relative_unix_path("foo/./bar"),
            Err(ArtifactInvalidPathError::NormalizedPath(
                "foo/./bar".to_owned()
            )),
        );
    }

    #[test]
    fn path_with_cur_dir_component_at_end() {
        assert_eq!(
            normalize_as_relative_unix_path("foo/."),
            Err(ArtifactInvalidPathError::NormalizedPath("foo/.".to_owned())),
        );
    }

    #[test]
    fn path_starting_with_parent_dir_component() {
        assert_eq!(
            normalize_as_relative_unix_path("../foo"),
            Err(ArtifactInvalidPathError::ParentDirComponent(
                "../foo".to_owned()
            )),
        );
    }

    #[test]
    fn path_with_parent_dir_component_in_middle() {
        assert_eq!(
            normalize_as_relative_unix_path("foo/../bar"),
            Err(ArtifactInvalidPathError::ParentDirComponent(
                "foo/../bar".to_owned()
            )),
        );
    }

    #[test]
    fn path_with_parent_dir_component_at_end() {
        assert_eq!(
            normalize_as_relative_unix_path("foo/.."),
            Err(ArtifactInvalidPathError::ParentDirComponent(
                "foo/..".to_owned()
            )),
        );
    }

    #[test]
    fn reject_empty_string() {
        assert_eq!(
            normalize_as_relative_unix_path(""),
            Err(ArtifactInvalidPathError::EmptyPath),
        );
    }

    #[test]
    fn absolute_unix_path() {
        assert_eq!(
            normalize_as_relative_unix_path("/usr/local/bin/dotslash"),
            Err(ArtifactInvalidPathError::RootDirComponent(
                "/usr/local/bin/dotslash".to_owned()
            )),
        );
    }

    #[test]
    fn absolute_windows_path() {
        assert_eq!(
            normalize_as_relative_unix_path("C:\\Tools\\dotslash.exe"),
            Err(ArtifactInvalidPathError::ContainsBackslash(
                "C:\\Tools\\dotslash.exe".to_owned()
            )),
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn absolute_windows_path_with_forward_slashes() {
        assert_eq!(
            normalize_as_relative_unix_path("C:/Tools/dotslash.exe"),
            // Note how on UNIX, what would be an absolute file path on Windows
            // is treated as a relative path with three components.
            Ok(3)
        );
    }

    #[test]
    #[cfg(windows)]
    fn absolute_windows_path_with_forward_slashes() {
        assert_eq!(
            normalize_as_relative_unix_path("C:/Tools/dotslash.exe"),
            Err(ArtifactInvalidPathError::PrefixComponent(
                "C:/Tools/dotslash.exe".to_owned(),
            ))
        );
    }

    #[test]
    fn relative_path_with_backslashes() {
        assert_eq!(
            normalize_as_relative_unix_path("foo\\bar"),
            Err(ArtifactInvalidPathError::ContainsBackslash(
                "foo\\bar".to_owned()
            )),
        );
    }
}
