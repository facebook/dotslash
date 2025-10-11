/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use criterion::{Criterion, criterion_group, criterion_main};
use dotslash::bench::{REQUIRED_HEADER, parse_file};
use std::fs;
use std::path::PathBuf;

fn read_fixture(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::read_to_string(p).expect("fixture missing")
}

fn large_config() -> String {
    let mut config = format!(
        r#"{}
{{
    // Large configuration with many platforms and providers
    "name": "large_tool",
    "platforms": {{
"#,
        REQUIRED_HEADER
    );

    let platforms = vec![
        "linux-x86_64",
        "linux-aarch64",
        "macos-x86_64",
        "macos-aarch64",
        "windows-x86_64",
        "windows-aarch64",
    ];

    let digests = vec![
        "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
        "bf1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
        "cf1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
        "df1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
        "ef1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
        "ff1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
    ];

    for (i, platform) in platforms.iter().enumerate() {
        config.push_str(&format!(
            r#"        // Platform: {}
        "{}": {{
            "size": {},
            "hash": "blake3",
            "digest": "{}",
            "format": "{}",
            "path": "{}",
            "providers": [
                {{
                    "type": "github-release",
                    "repo": "example/large_tool",
                    "tag": "v2.5.0",
                    "asset": "tool-{}.{}"
                }},
                {{
                    "type": "http",
                    "url": "https://cdn1.example.com/releases/v2.5.0/tool-{}.{}"
                }},
                {{
                    "type": "http",
                    "url": "https://cdn2.example.com/releases/v2.5.0/tool-{}.{}"
                }},
                {{
                    "type": "http",
                    "url": "https://mirror.example.com/releases/v2.5.0/tool-{}.{}"
                }},
                {{
                    "type": "s3",
                    "bucket": "example-releases",
                    "key": "releases/v2.5.0/tool-{}.{}"
                }}
            ]
        }}{}"#,
            platform,
            platform,
            1024 * 1024 * (i + 1),
            digests[i],
            if platform.starts_with("windows") {
                "zip"
            } else {
                "tar.gz"
            },
            if platform.starts_with("windows") {
                "bin/tool.exe"
            } else {
                "bin/tool"
            },
            platform,
            if platform.starts_with("windows") {
                "zip"
            } else {
                "tar.gz"
            },
            platform,
            if platform.starts_with("windows") {
                "zip"
            } else {
                "tar.gz"
            },
            platform,
            if platform.starts_with("windows") {
                "zip"
            } else {
                "tar.gz"
            },
            platform,
            if platform.starts_with("windows") {
                "zip"
            } else {
                "tar.gz"
            },
            platform,
            if platform.starts_with("windows") {
                "zip"
            } else {
                "tar.gz"
            },
            if i < platforms.len() - 1 { "," } else { "" }
        ));
        config.push('\n');
    }

    config.push_str(
        r#"    }
}"#,
    );

    config
}

fn benchmark_parse_file(c: &mut Criterion) {
    let medium = read_fixture("http__tar_gz__print_argv");
    let large = large_config();

    let mut group = c.benchmark_group("parse_file");
    group.bench_function("medium_config", |b| b.iter(|| parse_file(&medium).unwrap()));
    group.bench_function("large_config", |b| b.iter(|| parse_file(&large).unwrap()));

    group.finish();
}

criterion_group!(benches, benchmark_parse_file);
criterion_main!(benches);
