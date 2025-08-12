#!/usr/bin/env python3
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is dual-licensed under either the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree or the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree. You may select, at your option, one of the
# above-listed licenses.


import os
import subprocess
import sys
from pathlib import Path

IS_WINDOWS: bool = os.name == "nt"


def main() -> None:
    if not IS_WINDOWS:
        raise Exception("Only Windows is supported.")

    dotslash_windows_shim_root = Path(os.path.realpath(__file__)).parent
    dotslash_root = dotslash_windows_shim_root.parent

    target_dir = (
        Path(os.environ["CARGO_TARGET_DIR"])
        if "CARGO_TARGET_DIR" in os.environ
        else None
    )

    if "DOTSLASH_BIN" not in os.environ:
        subprocess.run(
            [
                "cargo",
                "build",
                "--quiet",
                "--manifest-path",
                str(dotslash_root / "Cargo.toml"),
                "--bin=dotslash",
                "--release",
            ],
            check=True,
        )
        os.environ["DOTSLASH_BIN"] = str(
            (target_dir or (dotslash_root / "target")) / "release" / "dotslash.exe"
        )

    if "DOTSLASH_WINDOWS_SHIM" not in os.environ:
        subprocess.run(
            [
                "cargo",
                "build",
                "--quiet",
                "--manifest-path",
                str(dotslash_windows_shim_root / "Cargo.toml"),
                "--bin=dotslash_windows_shim",
                "--release",
                # UNCOMMENT to compile allowing std use - useful for debugging.
                # "--no-default-features",
            ],
            check=True,
            env={**os.environ, "RUSTC_BOOTSTRAP": "1"},
        )
        os.environ["DOTSLASH_WINDOWS_SHIM"] = str(
            (target_dir or (dotslash_windows_shim_root / "target"))
            / "release"
            / "dotslash_windows_shim.exe"
        )

    subprocess.run(
        [
            sys.executable,
            str(dotslash_windows_shim_root / "tests" / "test.py"),
        ],
        check=True,
    )


if __name__ == "__main__":
    main()
