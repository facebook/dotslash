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
import struct
import subprocess
import sys
from pathlib import Path

IS_WINDOWS: bool = os.name == "nt"

target_triplets: list[str] = ["x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"]


def write_linker_stub(path: Path) -> None:
    # A PE image starts with an MZ-compatible prefix whose field at offset
    # 0x3c points Windows to the PE signature. lld-link's default prefix makes
    # the headers spill into a second 512-byte file-alignment block. Supplying
    # this minimal valid prefix via /STUB keeps SizeOfHeaders, and therefore the
    # complete shim, 512 bytes smaller without changing how Windows starts it.
    #
    # References:
    # - Microsoft documents the PE/COFF format at
    #   https://learn.microsoft.com/en-us/windows/win32/debug/pe-format.
    # - Microsoft documents the /STUB linker option at
    #   https://learn.microsoft.com/en-us/cpp/build/reference/stub-ms-dos-stub-file-name
    #
    # The header describes one 69-byte image with a 64-byte header and no
    # relocations. Its five-byte payload makes the input a complete executable,
    # as required by /STUB; Windows does not execute it when loading the PE.
    stub = bytearray(64)
    # These fields describe the complete one-page image and its 64-byte header.
    struct.pack_into("<H", stub, 0x00, 0x5A4D)
    struct.pack_into("<H", stub, 0x02, 69)
    struct.pack_into("<H", stub, 0x04, 1)
    struct.pack_into("<H", stub, 0x08, 4)
    struct.pack_into("<H", stub, 0x0C, 0xFFFF)
    struct.pack_into("<H", stub, 0x18, 0x40)
    stub.extend(b"\xb8\x00\x4c\xcd\x21")
    if len(stub) != 69:
        raise AssertionError(f"expected a 69-byte linker stub, got {len(stub)} bytes")
    path.write_bytes(stub)


def main(targets: list[str] | None = None) -> None:
    if not IS_WINDOWS:
        raise Exception("Only Windows is supported.")

    # Default to all targets; a caller (e.g. CI) may pass a subset to build just
    # the target that matches the current runner's architecture.
    if targets:
        unknown = [t for t in targets if t not in target_triplets]
        if unknown:
            raise SystemExit(f"Unknown target(s): {', '.join(unknown)}")
    else:
        targets = target_triplets

    dotslash_windows_shim_root = Path(os.path.realpath(__file__)).parent

    target_dir = (
        Path(os.environ["CARGO_TARGET_DIR"])
        if "CARGO_TARGET_DIR" in os.environ
        else None
    )

    # Link with the linker bundled in the active Rust toolchain (rust-lld)
    # instead of the MSVC link.exe. lld-link emits no "Rich" header, so the
    # output depends only on the pinned toolchain (see rust-toolchain.toml) and
    # not on whichever Visual Studio version happens to be installed. Together
    # with /Brepro below this keeps the checked-in binaries byte-for-byte
    # reproducible, which the verify-windows-shim CI job relies on.
    target_libdir = Path(
        subprocess.check_output(
            ["rustc", "--print", "target-libdir"], text=True
        ).strip()
    )
    rust_lld = target_libdir.parent / "bin" / "rust-lld.exe"
    if not rust_lld.is_file():
        raise FileNotFoundError(f"Rust's bundled linker was not found: {rust_lld}")

    # Regenerate the checked-in linker input before any selected release binary.
    linker_stub = dotslash_windows_shim_root / "dotslash_windows_linker_stub.exe"
    write_linker_stub(linker_stub)

    rustflags = [
        f"-Clinker={rust_lld}",
        "-Clinker-flavor=lld-link",
        "-Clink-arg=/DEBUG:NONE",  # Avoid an embedded PDB path.
        "-Clink-arg=/NODEFAULTLIB:msvcrt",  # The shim does not use the CRT.
        "-Clink-arg=/Brepro",  # Hash-based timestamps instead of wall-clock time.
        "-Clink-arg=/MERGE:.pdata=.rdata",  # Both sections are read-only.
        f"-Clink-arg=/STUB:{linker_stub}",
    ]

    # Ambient RUSTFLAGS could change the measured release layout and break
    # reproducibility. Encoded flags also preserve the linker and stub paths as
    # single arguments when the workspace path contains spaces.
    build_env = {**os.environ}
    build_env.pop("RUSTFLAGS", None)
    build_env["RUSTC_BOOTSTRAP"] = "1"  # Required by no_std language items.
    build_env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(rustflags)

    for triplet in targets:
        subprocess.run(
            [
                "cargo",
                "build",
                "--quiet",
                "--manifest-path",
                str(dotslash_windows_shim_root / "Cargo.toml"),
                "--bin=dotslash_windows_shim",
                "--release",
                f"--target={triplet}",
            ],
            check=True,
            env=build_env,
        )

        src = (
            (target_dir or (dotslash_windows_shim_root / "target" / triplet))
            / "release"
            / "dotslash_windows_shim.exe"
        )

        arch = triplet.partition("-")[0]

        dest = dotslash_windows_shim_root / f"dotslash_windows_shim-{arch}.exe"

        shutil.copy(src, dest)


if __name__ == "__main__":
    main(sys.argv[1:])
