# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is dual-licensed under either the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree or the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree. You may select, at your option, one of the
# above-listed licenses.
from __future__ import annotations


def locate() -> str:
    """
    Returns:
        The path to the DotSlash binary that was installed by this package.
    """
    import os
    import sys
    import sysconfig

    from dotslash._locate import _search_paths

    if extension := sysconfig.get_config_var("EXE"):
        binary_name = f"dotslash{extension}"
    elif sys.platform == "win32":
        binary_name = "dotslash.exe"
    else:
        binary_name = "dotslash"

    # Map of normalized paths to the actual path
    seen_paths: dict[str, str] = {}
    for search_path in _search_paths():
        normalized_path = os.path.normcase(search_path)
        if normalized_path in seen_paths:
            continue

        seen_paths[normalized_path] = search_path
        binary_path = os.path.join(search_path, binary_name)
        if os.path.isfile(binary_path):
            return binary_path

    search_paths = "\n".join(f"- {search_path}" for search_path in seen_paths.values())
    msg = f"The `{binary_name}` binary was not found in any of the following paths:\n{search_paths}"
    raise FileNotFoundError(msg)
