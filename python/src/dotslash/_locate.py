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
import sysconfig


def _search_paths():
    # The scripts directory for the current Python installation
    yield sysconfig.get_path("scripts")

    # The scripts directory for the base prefix if in a virtual environment
    yield sysconfig.get_path("scripts", vars={"base": sys.base_prefix, "platbase": sys.base_exec_prefix})

    module_dir = os.path.dirname(os.path.abspath(__file__))
    package_parent, package_name = os.path.split(module_dir)
    if package_name == "dotslash":
        # Above the package root e.g. when running `pip install --prefix` or `uv run --with`
        # Windows: <prefix>\Lib\site-packages\dotslash
        # macOS: <prefix>/lib/pythonX.Y/site-packages/dotslash
        # Linux:
        #   <prefix>/lib/pythonX.Y/site-packages/dotslash
        #   <prefix>/lib/pythonX.Y/dist-packages/dotslash (Debian-based distributions)
        head, tail = os.path.split(package_parent)
        if tail.endswith("-packages"):
            head, tail = os.path.split(head)
            if sys.platform == "win32":
                if tail == "Lib":
                    yield os.path.join(head, "Scripts")
            elif tail.startswith("python"):
                head, tail = os.path.split(head)
                if tail == sys.platlibdir:
                    yield os.path.join(head, "bin")
        else:
            # Adjacent to the package root e.g. when using the `--target` option of pip-like installers
            yield os.path.join(package_parent, "bin")

    # The scripts directory for user installations
    yield sysconfig.get_path("scripts", scheme=sysconfig.get_preferred_scheme("user"))
