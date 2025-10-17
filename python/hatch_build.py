# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is dual-licensed under either the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree or the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree. You may select, at your option, one of the
# above-listed licenses.
from __future__ import annotations

import os
import sys
from contextlib import contextmanager
from functools import cached_property
from platform import machine
from typing import TYPE_CHECKING, Any, BinaryIO

from hatchling.builders.hooks.plugin.interface import BuildHookInterface

if TYPE_CHECKING:
    from collections.abc import Generator


class CustomBuildHook(BuildHookInterface):
    def initialize(self, version: str, build_data: dict[str, Any]) -> None:  # noqa: ARG002
        if self.__source == "release":
            asset = self.__release_asset
        elif os.path.isdir(self.__source):
            asset = self.__local_asset
        elif not os.path.isabs(self.__source) and os.path.isfile(os.path.join(self.root, "PKG-INFO")):
            msg = (
                "The current directory has a `PKG-INFO` file, which likely means that the wheel is being "
                "built from an unpacked source distribution. You must do one of the following:\n"
                "- Set the source to an absolute rather than relative path.\n"
                "- Build the wheel separately e.g. run `python -m build --wheel` rather than just `python -m build`.\n"
                "- Set the source to `release`."
            )
            raise ValueError(msg)
        else:
            msg = "Source must be `release` or an absolute path to a directory containing release assets"
            raise ValueError(msg)

        import tarfile

        with asset() as reader, tarfile.open(fileobj=reader, mode="r:gz") as tf:
            tf.extractall(self.__temp_dir)

        if not os.path.isfile(self.__binary_path):
            msg = f"Binary not found: {self.__binary_path}"
            raise RuntimeError(msg)

        build_data["shared_scripts"][self.__binary_path] = self.__binary_name
        build_data["tag"] = f"py3-none-{self.__wheel_arch}"

    def finalize(self, version: str, build_data: dict[str, Any], artifact: str) -> None:  # noqa: ARG002
        import shutil

        shutil.rmtree(self.__temp_dir)

    @contextmanager
    def __release_asset(self) -> Generator[BinaryIO, None, None]:
        from urllib.request import HTTPError, urlopen

        tag = f"v{self.metadata.version.removeprefix('v')}"
        url = f"https://github.com/facebook/dotslash/releases/download/{tag}/{self.__asset_name}"

        try:
            with urlopen(url) as response:
                yield response
        except HTTPError as e:
            e.msg += f"\nURL: {url}"
            raise

    @contextmanager
    def __local_asset(self) -> Generator[BinaryIO, None, None]:
        asset_path = os.path.join(self.__source, self.__asset_name)
        if not os.path.isfile(asset_path):
            msg = f"Asset not found: {asset_path}"
            raise RuntimeError(msg)

        with open(asset_path, "rb") as f:
            yield f

    @cached_property
    def __platform(self) -> str:
        match sys.platform:
            case "win32":
                return "windows"
            case "darwin":
                return "macos"
            case "linux":
                return "linux"
            case _:
                msg = f"Unsupported platform: {sys.platform}"
                raise ValueError(msg)

    @cached_property
    def __arch(self) -> str:
        return machine().lower()

    @cached_property
    def __target_data(self) -> tuple[str, str]:
        match self.__platform, self.__arch:
            case "linux", "aarch64":
                return "dotslash-linux-musl.aarch64.tar.gz", self.__get_linux_wheel_arch()
            case "linux", "x86_64":
                return "dotslash-linux-musl.x86_64.tar.gz", self.__get_linux_wheel_arch()
            case "windows", "arm64":
                return "dotslash-windows-arm64.tar.gz", "win_arm64"
            case "windows", "amd64":
                return "dotslash-windows.tar.gz", "win_amd64"
            case "macos", "arm64":
                return "dotslash-macos-arm64.tar.gz", "macosx_11_0_arm64"
            case "macos", "x86_64":
                return "dotslash-macos-x86_64.tar.gz", "macosx_10_12_x86_64"
            case _:
                msg = f"Unsupported platform/arch pair: {self.__platform} {self.__arch}"
                raise ValueError(msg)

    @cached_property
    def __asset_name(self) -> str:
        return self.__target_data[0]

    @cached_property
    def __wheel_arch(self) -> str:
        return self.__target_data[1]

    @cached_property
    def __source(self) -> str:
        return os.environ.get("DOTSLASH_SOURCE") or self.config.get("source") or "release"

    @cached_property
    def __temp_dir(self) -> str:
        import tempfile

        return tempfile.mkdtemp()

    @cached_property
    def __binary_name(self) -> str:
        return "dotslash.exe" if self.__platform == "windows" else "dotslash"

    @cached_property
    def __binary_path(self) -> str:
        return os.path.join(self.__temp_dir, self.__binary_name)

    def __get_linux_wheel_arch(self) -> str:
        default_arch = f"musllinux_1_1_{self.__arch}"

        if not (hasattr(os, "confstr") and "CS_GNU_LIBC_VERSION" in os.confstr_names):
            return default_arch

        try:
            value = os.confstr("CS_GNU_LIBC_VERSION")
        except (OSError, ValueError):
            pass
        else:
            if value and value.startswith("glibc"):
                return f"manylinux_2_28_{self.__arch}"

        return default_arch
