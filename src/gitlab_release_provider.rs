/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::path::Path;
use std::process::Command;

use serde::Deserialize;
use serde_jsonrc::value::Value;

use crate::config::ArtifactEntry;
use crate::provider::Provider;
use crate::util::file_lock::FileLock;

pub struct GitLabReleaseProvider {}

#[derive(Deserialize, Debug, PartialEq)]
struct GitLabReleaseProviderConfig {
    tag: String,
    repo: String,
    name: String,
}

impl Provider for GitLabReleaseProvider {
    fn fetch_artifact(
        &self,
        provider_config: &Value,
        destination: &Path,
        _fetch_lock: &FileLock,
        _artifact_entry: &ArtifactEntry,
    ) -> anyhow::Result<()> {
        let GitLabReleaseProviderConfig { tag, repo, name } =
            GitLabReleaseProviderConfig::deserialize(provider_config)?;

        let _output = Command::new("glab")
            .arg("release")
            .arg("download")
            .arg(tag)
            .arg("--repo")
            .arg(repo)
            .arg("--asset-name")
            .arg(name)
            .arg("--dir")
            .arg(destination)
            .output()?;

        Ok(())
    }
}
