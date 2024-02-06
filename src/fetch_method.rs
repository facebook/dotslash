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

#[derive(Default, Deserialize, Serialize, Debug, PartialEq, Eq)]
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

    #[serde(rename = "zst")]
    Zstd,
}

pub enum DecompressStep {
    Gzip,
    Zstd,
}

/// Currently, only .tar is supported, though we may support .zip in the future.
pub enum ArchiveFormat {
    Tar,
}

impl ArtifactFormat {
    pub fn extraction_policy(&self) -> (Option<DecompressStep>, Option<ArchiveFormat>) {
        match self {
            Self::Plain => (None, None),
            Self::Gz => (Some(DecompressStep::Gzip), None),
            Self::Tar => (None, Some(ArchiveFormat::Tar)),
            Self::TarGz => (Some(DecompressStep::Gzip), Some(ArchiveFormat::Tar)),
            Self::TarZstd => (Some(DecompressStep::Zstd), Some(ArchiveFormat::Tar)),
            Self::Zstd => (Some(DecompressStep::Zstd), None),
        }
    }
}
