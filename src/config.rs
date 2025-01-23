/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::collections::HashMap;

use anyhow::Context as _;
use serde::Deserialize;
use serde::Serialize;
use serde_jsonrc::value::Value;

use crate::artifact_path::ArtifactPath;
use crate::digest::Digest;
use crate::fetch_method::ArtifactFormat;

/// A DotSlash file must start with exactly these bytes on the first line
/// to be considered valid. Because a DotSlash file does not have a
/// standard extension, this gives us a reliable way to identify
/// all of the DotSlash files in the repo.
pub const REQUIRED_HEADER: &str = "#!/usr/bin/env dotslash";

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ConfigFile {
    #[cfg_attr(not(test), expect(dead_code))]
    pub name: String,
    pub platforms: HashMap<String, ArtifactEntry>,
}

#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ArtifactEntry<Format = ArtifactFormat> {
    pub size: u64,
    pub hash: HashAlgorithm,
    pub digest: Digest,
    #[serde(default)]
    pub format: Format,
    pub path: ArtifactPath,
    pub providers: Vec<Value>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub arg0: Arg0,
    #[serde(default = "readonly_default_as_true", skip_serializing_if = "is_true")]
    pub readonly: bool,
}

fn is_default<T>(t: &T) -> bool
where
    T: Default + PartialEq,
{
    *t == Default::default()
}

/// Determines what arg0 (`argv[0]`) gets set to.
/// Note: This has no effect on Windows, where the behavior is effectively
/// always "UnderlyingExecutable".
#[derive(Deserialize, Serialize, Copy, Clone, Default, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Arg0 {
    /// arg0 is set to the DotSlash file path as passed to `dotslash`.
    #[default]
    DotslashFile,
    /// arg0 is left unset, which defaults to the underlying executable
    /// in the cache directory.
    UnderlyingExecutable,
}

/// While having a boolean that defaults to `true` is somewhat undesirable,
/// the alternative would be to name the field "writable", which is too easy
/// to misspell as "writeable" (which would be ignored), so "readonly" it is.
fn readonly_default_as_true() -> bool {
    true
}

#[expect(clippy::trivially_copy_pass_by_ref)]
fn is_true(b: &bool) -> bool {
    *b
}

#[derive(Deserialize, Serialize, Copy, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum HashAlgorithm {
    #[serde(rename = "blake3")]
    Blake3,
    #[serde(rename = "sha256")]
    Sha256,
}

pub fn parse_file(data: &str) -> anyhow::Result<(Value, ConfigFile)> {
    // Check to see whether the DotSlash file starts with the proper shebang.
    let data = data
        .strip_prefix(REQUIRED_HEADER)
        .and_then(|rest| {
            rest.strip_prefix("\r\n")
                .or_else(|| rest.strip_prefix('\n'))
        })
        .with_context(|| {
            anyhow::format_err!("DotSlash file must start with `{REQUIRED_HEADER}`")
        })?;

    let value = serde_jsonrc::from_str::<Value>(data)?;
    let config_file = ConfigFile::deserialize(&value)?;
    Ok((value, config_file))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_file_string(json: &str) -> anyhow::Result<ConfigFile> {
        Ok(parse_file(json)?.1)
    }

    #[test]
    fn extract_config_file() {
        let dotslash = r#"#!/usr/bin/env dotslash
        {
            "name": "my_tool",
            "platforms": {
                "linux-x86_64": {
                    "size": 123,
                    "hash": "sha256",
                    "digest": "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069",
                    "format": "tar",
                    "path": "bindir/my_tool",
                    "providers": [
                        {
                            "type": "http",
                            "url": "https://example.com/my_tool.tar"
                        }
                    ]
                },
            },
        }
        "#;
        let config_file = parse_file_string(dotslash).unwrap();
        assert_eq!(
            config_file,
            ConfigFile {
                name: "my_tool".to_owned(),
                platforms: [(
                    "linux-x86_64".to_owned(),
                    ArtifactEntry {
                        size: 123,
                        hash: HashAlgorithm::Sha256,
                        digest: Digest::try_from(
                            "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069"
                                .to_owned(),
                        )
                        .unwrap(),
                        format: ArtifactFormat::Tar,
                        path: "bindir/my_tool".parse().unwrap(),
                        providers: vec![serde_jsonrc::json!({
                            "type": "http",
                            "url": "https://example.com/my_tool.tar",
                        })],
                        arg0: Arg0::DotslashFile,
                        readonly: true,
                    }
                )]
                .into(),
            },
        );
    }

    #[test]
    fn rename() {
        let dotslash = r#"#!/usr/bin/env dotslash
        {
            "name": "minesweeper",
            "platforms": {
                "linux-x86_64": {
                    "size": 123,
                    "hash": "sha256",
                    "digest": "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069",
                    "path": "minesweeper.exe",
                    "providers": [
                        {
                            "type": "http",
                            "url": "https://foo.com"
                        }
                    ],
                },
            },
        }
        "#;
        let config_file = parse_file_string(dotslash).unwrap();
        assert_eq!(
            config_file,
            ConfigFile {
                name: "minesweeper".to_owned(),
                platforms: [(
                    "linux-x86_64".to_owned(),
                    ArtifactEntry {
                        size: 123,
                        hash: HashAlgorithm::Sha256,
                        digest: Digest::try_from(
                            "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069"
                                .to_owned(),
                        )
                        .unwrap(),
                        format: ArtifactFormat::Plain,
                        path: "minesweeper.exe".parse().unwrap(),
                        providers: vec![serde_jsonrc::json!({
                            "type": "http",
                            "url": "https://foo.com",
                        })],
                        arg0: Arg0::DotslashFile,
                        readonly: true,
                    }
                )]
                .into(),
            }
        );
    }

    #[test]
    fn header_must_be_present() {
        let dotslash = r#"
        {
            "name": "made-up",
            "platforms": {
            },
        }
        "#;
        assert_eq!(
            parse_file_string(dotslash).map_err(|x| x.to_string()),
            Err("DotSlash file must start with `#!/usr/bin/env dotslash`".to_owned()),
        );
    }
}
