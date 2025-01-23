/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use serde::Deserialize;
use serde::Serialize;

use crate::util::unarchive::ArchiveType;

#[derive(Deserialize, Serialize, Copy, Clone, Default, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ArtifactFormat {
    /// Artifact is a single file with no compression applied.
    #[default]
    #[serde(skip)]
    Plain,

    #[serde(rename = "gz")]
    Gz,

    #[serde(rename = "tar")]
    Tar,

    #[serde(rename = "tar.gz")]
    TarGz,

    #[serde(rename = "tar.zst")]
    TarZstd,

    #[serde(rename = "tar.xz")]
    TarXz,

    #[serde(rename = "xz")]
    Xz,

    #[serde(rename = "zst")]
    Zstd,

    #[serde(rename = "zip")]
    Zip,
}

impl ArtifactFormat {
    #[must_use]
    pub fn as_archive_type(self) -> Option<ArchiveType> {
        match self {
            Self::Plain => None,
            Self::Gz => Some(ArchiveType::Gz),
            Self::Xz => Some(ArchiveType::Xz),
            Self::Zstd => Some(ArchiveType::Zstd),
            Self::Tar => Some(ArchiveType::Tar),
            Self::TarGz => Some(ArchiveType::TarGz),
            Self::TarXz => Some(ArchiveType::TarXz),
            Self::TarZstd => Some(ArchiveType::TarZstd),
            Self::Zip => Some(ArchiveType::Zip),
        }
    }

    #[must_use]
    pub fn is_container(self) -> bool {
        match self {
            Self::Plain | Self::Gz | Self::Xz | Self::Zstd => false,
            Self::Tar | Self::TarGz | Self::TarXz | Self::TarZstd | Self::Zip => true,
        }
    }
}
