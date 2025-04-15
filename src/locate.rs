/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use anyhow::Context as _;

use crate::artifact_location::ArtifactLocation;
use crate::artifact_location::determine_location;
use crate::config;
use crate::config::ArtifactEntry;
use crate::dotslash_cache::DotslashCache;
use crate::platform::SUPPORTED_PLATFORM;
use crate::util;
use crate::util::ListOf;

pub fn locate_artifact(
    dotslash_data: &str,
    dotslash_cache: &DotslashCache,
) -> anyhow::Result<(ArtifactEntry, ArtifactLocation)> {
    let (_original_json, mut config_file) =
        config::parse_file(dotslash_data).context("failed to parse DotSlash file")?;

    let (_platform, artifact_entry) = config_file
        .platforms
        .remove_entry(SUPPORTED_PLATFORM)
        .ok_or_else(|| {
            anyhow::format_err!(
                "expected platform `{}` - but found {}",
                SUPPORTED_PLATFORM,
                ListOf::new(config_file.platforms.keys()),
            )
        })
        .context("platform not supported")?;

    let artifact_location = determine_location(&artifact_entry, dotslash_cache);

    // Update the mtime to work around tmpwatch and tmpreaper behavior
    // with old artifacts.
    //
    // Not on macOS because something (macOS security?) adds a 50-100ms
    // delay after modifying the file.
    //
    // Not on Windows because of "file used by another process" errors.
    if cfg!(target_os = "linux") {
        let _ = util::update_mtime(&artifact_location.executable);
    }

    Ok((artifact_entry, artifact_location))
}
