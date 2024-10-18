/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::path::Path;

use serde_jsonrc::value::Value;

use crate::config::ArtifactEntry;
use crate::util::FileLock;

pub trait Provider {
    /// When called, the provider should fetch the artifact as specified by the
    /// `provider_config` and write it to `destination`.
    ///
    /// provider_config: JSON value parsed from the DotSlash file that defines
    ///     the configuration for this provider.
    ///
    /// destination: Where the artifact should be written. The caller ensures
    ///     the parent folder of `destination` exists.
    ///
    /// fetch_lock: A lock file that should be held while the artifact is being
    ///     fetched.
    ///
    /// artifact_entry: In general, the Provider should not rely on the
    ///     information in the entry to perform the fetch, as such information
    ///     should be defined in the provider_config. It is primarily provided
    ///     so the Provider can show an appropriate progess indicator based on
    ///     the expected size of the artifact.
    fn fetch_artifact(
        &self,
        provider_config: &Value,
        destination: &Path,
        fetch_lock: &FileLock,
        artifact_entry: &ArtifactEntry,
    ) -> anyhow::Result<()>;
}

pub trait ProviderFactory {
    fn get_provider(&self, provider_type: &str) -> anyhow::Result<Box<dyn Provider>>;
}
