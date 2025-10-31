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

import dotslash


def test_locate() -> None:
    path = dotslash.locate()
    assert isinstance(path, str)
    assert os.path.isabs(path)
    assert os.path.isfile(path)

    stem, extension = os.path.splitext(os.path.basename(path))
    assert stem == "dotslash"

    expected_extension = ".exe" if sys.platform == "win32" else ""
    assert extension == expected_extension
