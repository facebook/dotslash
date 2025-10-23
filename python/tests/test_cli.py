# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is dual-licensed under either the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree or the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree. You may select, at your option, one of the
# above-listed licenses.
from __future__ import annotations

import subprocess
import sys
from importlib import metadata


def test_cli() -> None:
    result = subprocess.run([sys.executable, "-m", "dotslash", "--version"], capture_output=True, encoding="utf-8")
    output = result.stdout.strip()
    assert result.returncode == 0, output

    name, version = output.split()
    assert name == "DotSlash"
    assert version == metadata.version("dotslash")
