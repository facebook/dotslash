#!/usr/bin/env python3
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is dual-licensed under either the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree or the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree. You may select, at your option, one of the
# above-listed licenses.


import os
import shutil
import subprocess
from pathlib import Path

IS_WINDOWS: bool = os.name == "nt"


def main() -> None:
    if not IS_WINDOWS:
        raise Exception("Only Windows is supported.")

    dotslash_windows_shim_root = Path(os.path.realpath(__file__)).parent

    target_dir = (
        Path(os.environ["CARGO_TARGET_DIR"])
        if "CARGO_TARGET_DIR" in os.environ
        else None
    )

    subprocess.run(
        [
            "cargo",
            "build",
            "--quiet",
            "--manifest-path",
            str(dotslash_windows_shim_root / "Cargo.toml"),
            "--bin=dotslash_windows_shim",
            "--release",
            "--target=x86_64-pc-windows-msvc",
        ],
        check=True,
        env={
            **os.environ,
            "RUSTC_BOOTSTRAP": "1",
            "RUSTFLAGS": "-Clink-arg=/DEBUG:NONE",  # Avoid embedded pdb path
        },
    )

    src = (
        (
            target_dir
            or (dotslash_windows_shim_root / "target" / "x86_64-pc-windows-msvc")
        )
        / "release"
        / "dotslash_windows_shim.exe"
    )

    dest = dotslash_windows_shim_root / "dotslash_windows_shim-x86_64.exe"

    shutil.copy(src, dest)


if __name__ == "__main__":
    main()
