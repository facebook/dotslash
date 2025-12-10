/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use std::path::Path;

use anyhow::Context as _;
use serde::Deserialize;
use serde_json::Value;

use crate::config::ArtifactEntry;
use crate::provider::Provider;
use crate::util::CommandDisplay;
use crate::util::CommandStderrDisplay;
use crate::util::FileLock;

pub struct S3Provider {}

#[derive(Deserialize, Debug)]
struct S3ProviderConfig {
    bucket: String,
    key: String,
    region: Option<String>,
}

impl Provider for S3Provider {
    fn fetch_artifact(
        &self,
        provider_config: &Value,
        destination: &Path,
        _fetch_lock: &FileLock,
        _: &ArtifactEntry,
    ) -> anyhow::Result<()> {
        let S3ProviderConfig {
            bucket,
            key,
            region,
        } = <_>::deserialize(provider_config)?;
        let mut command = std::process::Command::new("aws");
        command.args(["s3", "cp"]);
        if let Some(region) = region {
            command.args(["--region", &region]);
        }
        command.arg(format!("s3://{bucket}/{key}"));
        command.arg(destination);
        let output = command
            .output()
            .with_context(|| format!("{}", CommandDisplay::new(&command)))
            .context("failed to run the AWS CLI")?;

        if !output.status.success() {
            return Err(anyhow::format_err!(
                "{}",
                CommandStderrDisplay::new(&output)
            ))
            .with_context(|| format!("{}", CommandDisplay::new(&command)))
            .context("the AWS CLI failed");
        }
        Ok(())
    }
}
