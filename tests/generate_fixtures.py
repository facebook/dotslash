#!/usr/bin/env python3
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

import argparse
import json
import os
import subprocess

"""Run this script to regenerate the runnable print_argv DotSlash files in the
fixtures/ directory.
"""


platform_configs = [
    ("linux.aarch64", "linux-aarch64"),
    ("linux.x86_64", "linux-x86_64"),
    ("macos.aarch64", "macos-aarch64"),
    ("macos.x86_64", "macos-x86_64"),
    ("windows.aarch64.exe", "windows-aarch64"),
    ("windows.x86_64.exe", "windows-x86_64"),
]

GITHUB_REPO = "https://github.com/zertosh/dotslash_fixtures"
DOTSLASH_EXE_NAME = "print_argv"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--commit-hash", required=True)
    args = parser.parse_args()

    commit_hash: str = args.commit_hash

    # TODO(asuarez): Make the artifacts for [".tar", ".tar.gz", ".tar.zst"]
    # available as well?
    file_extensions = ["", ".gz", ".xz", ".zst"]
    for file_extension in file_extensions:
        generate_dotslash_file_for_file_extension(
            commit_hash=commit_hash,
            file_extension=file_extension,
        )


def generate_dotslash_file_for_file_extension(
    commit_hash: str,
    file_extension: str,
) -> None:
    platforms = {}
    for platform_name, platform_id in platform_configs:
        url = f"{GITHUB_REPO}/raw/{commit_hash}/print_argv.{platform_name}{file_extension}"
        entry_json = subprocess.check_output(
            [
                "cargo",
                "run",
                "--release",
                "--quiet",
                "--",
                "--",
                "create-url-entry",
                url,
            ]
        )
        entry = json.loads(entry_json)
        if file_extension == "":
            del entry["format"]
        entry["path"] = f"subdir/print_argv.{platform_name}"
        platforms[platform_id] = entry

    dotslash_json = {
        "name": DOTSLASH_EXE_NAME,
        "platforms": platforms,
    }

    if file_extension == "":
        format_id = "plain"
    else:
        format_id = file_extension[1:].replace(".", "_")
    dotslash_file = os.path.join(
        os.path.dirname(__file__), "fixtures", f"http__{format_id}__print_argv"
    )

    with open(dotslash_file, "w") as f:
        f.write("#!/usr/bin/env dotslash\n\n")
        f.write(json.dumps(dotslash_json, indent=2))
        f.write("\n")


if __name__ == "__main__":
    main()
