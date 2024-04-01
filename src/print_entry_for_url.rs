/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::io::IsTerminal as _;
use std::str::FromStr;

use anyhow::Context as _;
use serde_jsonrc::json;
use sha2::Digest as _;

use crate::artifact_path::ArtifactPath;
use crate::config::ArtifactEntry;
use crate::config::HashAlgorithm;
use crate::curl::CurlCommand;
use crate::curl::FetchContext;
use crate::fetch_method::ArtifactFormat;

type LooseArtifactEntry = ArtifactEntry<String>;

/// This function creates an approximate ArtifactEntry for the specified URL
/// and writes it to stdout as pretty-printed JSON.
pub(crate) fn print_entry_for_url(url: &OsStr) -> anyhow::Result<()> {
    let curl_cmd = CurlCommand::new(url);
    let url = url
        .to_str()
        .with_context(|| format!("arg is not UTF-8 `{}`", url.to_string_lossy()))?;
    let fetch_context = FetchContext {
        artifact_name: url,
        content_length: 0,
        show_progress: std::io::stderr().is_terminal(),
    };
    let tempfile = tempfile::NamedTempFile::new()?;
    curl_cmd
        .get_request(tempfile.path(), &fetch_context)
        .with_context(|| format!("failed to fetch `{}`", url))?;

    let file = File::open(tempfile.path())?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();
    let (size, hex_digest) = std::io::copy(&mut reader, &mut hasher).map(|size_in_bytes| {
        let digest = format!("{:x}", hasher.finalize());
        (size_in_bytes, digest)
    })?;

    let entry_json = serialize_entry(url, size, hex_digest)?;
    println!("{}", entry_json);
    Ok(())
}

fn serialize_entry(url: &str, size: u64, hex_digest: impl Into<String>) -> anyhow::Result<String> {
    let format = match guess_artifact_format_from_url(url.as_bytes()) {
        ArtifactFormat::Plain => {
            "TODO: specify this value; could not guess format from URL".to_string()
        }
        format => serde_jsonrc::to_value(format)?
            .as_str()
            .unwrap()
            .to_string(),
    };
    let entry = LooseArtifactEntry {
        size,
        hash: HashAlgorithm::Blake3,
        digest: hex_digest.into().try_into()?,
        format,
        path: ArtifactPath::from_str("TODO: specify the appropriate `path` for this artifact")?,
        providers: vec![json!({"url": url})],
        readonly: true,
    };
    let entry_json = serde_jsonrc::to_string_pretty(&entry)?;
    Ok(entry_json)
}

fn guess_artifact_format_from_url(url: &[u8]) -> ArtifactFormat {
    if url.ends_with(b".tar.gz") || url.ends_with(b".tgz") {
        ArtifactFormat::TarGz
    } else if url.ends_with(b".tar.zst") || url.ends_with(b".tzst") {
        ArtifactFormat::TarZstd
    } else if url.ends_with(b".tar.xz") {
        ArtifactFormat::TarXz
    } else if url.ends_with(b".tar") {
        ArtifactFormat::Tar
    } else if url.ends_with(b".zip") {
        ArtifactFormat::Zip
    } else if url.ends_with(b".gz") {
        ArtifactFormat::Gz
    } else if url.ends_with(b".xz") {
        ArtifactFormat::Xz
    } else if url.ends_with(b".zst") {
        ArtifactFormat::Zstd
    } else {
        ArtifactFormat::Plain
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::artifact_path::ArtifactPath;

    #[test]
    fn serialize_entry_recognized_format() -> anyhow::Result<()> {
        let url = "https://github.com/zertosh/dotslash_fixtures/raw/462625c6bf2671439dce66bd5bc40b05f2ed8819/pack.tar.gz";
        let size = 48689_u64;
        let hex_digest = "068464830bd5c276e085a4eab5ef9cc57159f94273db296d6a638e49b78ca55f";
        let entry_json = serialize_entry(url, size, hex_digest)?;
        let entry = serde_jsonrc::from_str::<ArtifactEntry>(&entry_json)?;
        // Ensure the output parses as a valid ArtifactEntry.
        assert_eq!(
            ArtifactEntry {
                size,
                hash: HashAlgorithm::Blake3,
                digest: hex_digest.to_string().try_into()?,
                format: ArtifactFormat::TarGz,
                path: ArtifactPath::from_str(
                    "TODO: specify the appropriate `path` for this artifact"
                )
                .unwrap(),
                providers: vec![json!({"url": url})],
                readonly: true,
            },
            entry,
        );
        Ok(())
    }

    #[test]
    fn serialize_entry_unrecognized_format() -> anyhow::Result<()> {
        let url = "http://example.com/somefile";
        let size = 48689_u64;
        let hex_digest = "068464830bd5c276e085a4eab5ef9cc57159f94273db296d6a638e49b78ca55f";
        let entry_json = serialize_entry(url, size, hex_digest)?;
        // Note that `entry_json` cannot be deserialized into an ArtifactEntry
        // due to the illegal value for `format`.
        assert_eq!(
            format!(
                r#"{{
  "size": {size},
  "hash": "blake3",
  "digest": "{hex_digest}",
  "format": "TODO: specify this value; could not guess format from URL",
  "path": "TODO: specify the appropriate `path` for this artifact",
  "providers": [
    {{
      "url": "{url}"
    }}
  ]
}}"#
            ),
            entry_json
        );
        Ok(())
    }

    #[test]
    fn guess_artifact_format() {
        #[track_caller]
        fn test(url: &str, expected_format: ArtifactFormat) {
            assert_eq!(
                expected_format,
                guess_artifact_format_from_url(url.as_bytes())
            );
        }

        test("http://example.com/foo.tar.gz", ArtifactFormat::TarGz);
        test("http://example.com/foo.tgz", ArtifactFormat::TarGz);
        test("http://example.com/foo.tar.xz", ArtifactFormat::TarXz);
        test("http://example.com/foo.tar.zst", ArtifactFormat::TarZstd);
        test("http://example.com/foo.tzst", ArtifactFormat::TarZstd);
        test("http://example.com/foo.tar", ArtifactFormat::Tar);
        test("http://example.com/foo.gz", ArtifactFormat::Gz);
        test("http://example.com/foo.zip", ArtifactFormat::Zip);
        test("http://example.com/foo.zst", ArtifactFormat::Zstd);

        // These "backwards" extensions are interpreted as Tar.
        test("http://example.com/foo.zst.tar", ArtifactFormat::Tar);
        test("http://example.com/foo.gz.tar", ArtifactFormat::Tar);

        test("http://example.com/foo", ArtifactFormat::Plain);
        test("http://example.com/foo.bar", ArtifactFormat::Plain);
        test("http://example.com/foo.zstd", ArtifactFormat::Plain);

        // The tool currently ignores query parameters.
        test("http://example.com/foo.tar.gz?dl=1", ArtifactFormat::Plain);
    }
}
