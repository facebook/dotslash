#!/usr/bin/env python3
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

# pyre-strict

import json
import os
import shutil
import subprocess
import tempfile
import time
import unittest
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path
from typing import Final

EMPTY_STR_LIST: Final[list[str]] = []

try:
    from .fb.ci import set_ci_envs

    set_ci_envs()
except ImportError:
    pass


def get_path_env(name: str) -> Path:
    try:
        value = os.environ[name]
    except KeyError as err:
        raise unittest.SkipTest(f"required env var not set: `{name}`") from err

    try:
        return Path(value).resolve(strict=True)
    except FileNotFoundError as err:
        raise unittest.SkipTest(f"required path does not exist: `{value}`") from err


class TestPathEnv(unittest.TestCase):
    def test_require_dotslash_bin_env(self) -> None:
        self.assertIn("DOTSLASH_BIN", os.environ)
        self.assertTrue(os.path.exists(os.environ["DOTSLASH_BIN"]))

    def test_require_dotslash_windows_shim_env(self) -> None:
        self.assertIn("DOTSLASH_WINDOWS_SHIM", os.environ)
        self.assertTrue(os.path.exists(os.environ["DOTSLASH_WINDOWS_SHIM"]))


def generate_dotslash_file(name: str) -> str:
    spec = {
        "name": name,
        "platforms": {
            platform: {
                "size": 85730,
                "hash": "blake3",
                "digest": "823b76bb8c632029aa8ee01830e31292934f1e4f01975e1caeab7b9e6b113438",
                "format": "tar.zst",
                "path": f"{name}.{suffix}",
                "providers": [
                    {
                        "url": "https://github.com/zertosh/dotslash_fixtures/raw/dc46c6298a20312c261da9a3c44a312b6a383611/dist/pack.tar.zst",
                    }
                ],
            }
            for suffix, platform in [
                ("linux.aarch64", "linux-aarch64"),
                ("linux.x86_64", "linux-x86_64"),
                ("macos.aarch64", "macos-aarch64"),
                ("macos.x86_64", "macos-x86_64"),
                ("windows.aarch64.exe", "windows-aarch64"),
                ("windows.x86_64.exe", "windows-x86_64"),
            ]
        },
    }
    json_str = json.dumps(spec, indent=2)
    return f"#!/usr/bin/env dotslash\n\n{json_str}\n"


# Windows CreateProcess seems to ignore changing the cwd and PATH on the
# `subprocess.run` call, so we change it on the parent process instead.
@contextmanager
def prepend_path(path: Path) -> Iterator[None]:
    curr = os.environ["PATH"]
    os.environ["PATH"] = os.pathsep.join([str(path), curr])
    try:
        yield
    finally:
        os.environ["PATH"] = curr


@contextmanager
def move_cwd(path: Path) -> Iterator[None]:
    curr = os.getcwd()
    os.chdir(path)
    try:
        yield
    finally:
        os.chdir(curr)


PRINT_ARGS_ARG0 = r"^0:.+\\print_args\.windows\.(aarch64|x86_64)\.exe$"


class DotslashWindowsShimTest(unittest.TestCase):
    def setUp(self) -> None:
        self.maxDiff = None

        dotslash_bin = get_path_env("DOTSLASH_BIN")
        dotslash_windows_shim = get_path_env("DOTSLASH_WINDOWS_SHIM")

        self._tempdir = tempfile.TemporaryDirectory()
        self._fixtures: Path = Path(self._tempdir.name)

        bin_dir = self._fixtures / "bin"
        bin_dir.mkdir()
        shutil.copy(dotslash_bin, bin_dir)

        for name in (
            "exit_code",
            "print_args",
            "print_argv",
            "stdin_to_stderr",
            "stdin_to_stdout",
        ):
            file_source = generate_dotslash_file(name)
            (self._fixtures / name).write_text(file_source)
            shutil.copy(dotslash_windows_shim, self._fixtures / f"{name}.exe")

        self._original_path = os.environ["PATH"]
        os.environ["PATH"] = os.pathsep.join([str(bin_dir), self._original_path])

    def tearDown(self) -> None:
        os.environ["PATH"] = self._original_path
        for _ in range(3):
            try:
                self._tempdir.cleanup()
                return
            except PermissionError:
                time.sleep(1)
        self._tempdir.cleanup()

    def test_args_none(self) -> None:
        ret = subprocess.run(
            [str(self._fixtures / "print_args.exe")],
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(ret.stderr, "")
        self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
        self.assertEqual(ret.returncode, 0)

    def test_args_none_symlink(self) -> None:
        # GetModuleFileNameW does not dereference symlinks!
        print_args_path = self._fixtures / "print_args.exe"
        print_args_path.unlink()
        print_args_path.symlink_to("stdin_to_stdout.exe")
        self.assertTrue(print_args_path.is_symlink())
        ret = subprocess.run(
            [str(print_args_path)],
            input="this should not be seen on Windows",
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(ret.stderr, "")
        self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
        self.assertEqual(ret.returncode, 0)

    def test_args_none_with_period_in_name(self) -> None:
        shutil.move(
            self._fixtures / "print_args",
            self._fixtures / "print_args.dotslash",
        )
        shutil.move(
            self._fixtures / "print_args.exe",
            self._fixtures / "print_args.dotslash.exe",
        )
        ret = subprocess.run(
            [str(self._fixtures / "print_args.dotslash.exe")],
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(ret.stderr, "")
        self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
        self.assertEqual(ret.returncode, 0)

    def test_args_none_with_no_extension(self) -> None:
        with move_cwd(self._fixtures):
            shutil.move("print_args.exe", "print_args")

        # This executes because CreateProcessW adds an implicit `.exe`.
        # PathCchRemoveExtension won't have an extension to remove,
        # so we'll pass the exetuable to `dotslash`, which will then
        # fail because it's not an actual DotSlash file.
        print_args_path = self._fixtures / "print_args"
        ret = subprocess.run(
            [print_args_path],
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(
            ret.stderr,
            f"dotslash error: problem with `{print_args_path}`\n"
            "caused by: failed to read DotSlash file\n"
            "caused by: stream did not contain valid UTF-8\n",
        )
        self.assertEqual(ret.stdout, "")
        self.assertEqual(ret.returncode, 1)

    def test_args_none_with_unc_path(self) -> None:
        ret = subprocess.run(
            ["\\\\?\\" + str(self._fixtures / "print_args.exe")],
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(ret.stderr, "")
        self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
        self.assertEqual(ret.returncode, 0)

    def test_args_none_max_path_name(self) -> None:
        # We need to overcome Python's own MAX_PATH limitations by
        # running through Cygwin/GitBash. PowerShell doesn't work for our
        # purposes because somehow it makes GetModuleFileNameW return
        # a short name (e.g. `C:\ABCDEF~1`) so we don't actually end up
        # testing that we handle a growing GetModuleFileNameW correctly.
        MAX_PATH = 260
        # No idea why MAGIC_NUMBER is what it is. The math doesn't add up.
        # But this is the number at which Cygwin/GitBash fail to execute
        # the program. The number varies on the filename but not the cwd.
        # But it's quite a bit higher than filename length itself for
        # some reason.
        MAGIC_NUMBER = 29
        SUFFIX = "x" * (MAX_PATH - MAGIC_NUMBER)
        BASH_CMD = shutil.which("bash")
        assert BASH_CMD is not None, "bash not found"
        with move_cwd(self._fixtures):
            ret = subprocess.run(
                [
                    BASH_CMD,
                    "-c",
                    "cp print_args print_args-{suffix} && "
                    "cp print_args.exe print_args-{suffix}.exe && "
                    "./print_args-{suffix}.exe".format(suffix=SUFFIX),
                ],
                capture_output=True,
                encoding="utf8",
            )
            self.assertEqual(ret.stderr, "")
            self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
            self.assertEqual(ret.returncode, 0)

    def test_args_empty(self) -> None:
        ret = subprocess.run(
            [str(self._fixtures / "print_args.exe"), ""],
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(ret.stderr, "1:\n")
        self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
        self.assertEqual(ret.returncode, 0)

    def test_args_simple(self) -> None:
        ret = subprocess.run(
            [str(self._fixtures / "print_args.exe"), "a", "b", "c"],
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(ret.stderr, "1:a\n2:b\n3:c\n")
        self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
        self.assertEqual(ret.returncode, 0)

    def test_args_simple_with_unicode_in_name(self) -> None:
        shutil.move(
            self._fixtures / "print_args",
            self._fixtures / "print🍎args",
        )
        shutil.move(
            self._fixtures / "print_args.exe",
            self._fixtures / "print🍎args.exe",
        )
        ret = subprocess.run(
            [str(self._fixtures / "print🍎args.exe"), "a", "b", "c"],
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(ret.stderr, "1:a\n2:b\n3:c\n")
        self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
        self.assertEqual(ret.returncode, 0)

    def test_args_with_space_in_name(self) -> None:
        with move_cwd(self._fixtures):
            shutil.move("print_args", "print args")
            shutil.move("print_args.exe", "print args.exe")

        for args, stderr in [
            (EMPTY_STR_LIST, ""),
            (["a", "b", "c"], "1:a\n2:b\n3:c\n"),
        ]:
            with self.subTest(args=args):
                ret = subprocess.run(
                    [str(self._fixtures / "print args.exe"), *args],
                    capture_output=True,
                    encoding="utf8",
                )
                self.assertEqual(ret.stderr, stderr)
                self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
                self.assertEqual(ret.returncode, 0)

    def test_args_simple_relative_path(self) -> None:
        with move_cwd(self._fixtures):
            ret = subprocess.run(
                ["./print_args.exe", "a", "b", "c"],
                capture_output=True,
                encoding="utf8",
            )
            self.assertEqual(ret.stderr, "1:a\n2:b\n3:c\n")
            self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
            self.assertEqual(ret.returncode, 0)

    def test_args_simple_in_path(self) -> None:
        with prepend_path(self._fixtures):
            ret = subprocess.run(
                ["print_args.exe", "a", "b", "c"],
                capture_output=True,
                encoding="utf8",
            )
            self.assertEqual(ret.stderr, "1:a\n2:b\n3:c\n")
            self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
            self.assertEqual(ret.returncode, 0)

    def test_args_quotes(self) -> None:
        ret = subprocess.run(
            [
                str(self._fixtures / "print_args.exe"),
                '""',
                '"""',
                'a="\\"\'b\'\\""',
            ],
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(ret.stderr, '1:""\n2:"""\n3:a="\\"\'b\'\\""\n')
        self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
        self.assertEqual(ret.returncode, 0)

    def test_args_unicode(self) -> None:
        ret = subprocess.run(
            [
                str(self._fixtures / "print_args.exe"),
                "a",  # 1 byte UTF-8 character https://www.compart.com/en/unicode/U+0061
                "Ф",  # 2 byte UTF-8 character https://www.compart.com/en/unicode/U+0424
                "ꎆ",  # 3 byte UTF-8 character https://www.compart.com/en/unicode/U+A386
                "🍎",  # 4 byte UTF-8 character https://www.compart.com/en/unicode/U+1F34E
                "👩‍❤️‍💋‍👨",  # 11 bytes https://emojipedia.org/kiss-woman-man/
            ],
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(ret.stderr, "1:a\n2:Ф\n3:ꎆ\n4:🍎\n5:👩‍❤️‍💋‍👨\n")
        self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
        self.assertEqual(ret.returncode, 0)

    def test_args_long_args(self) -> None:
        # Windows CreateProcess API has a length limit of 32,768.
        # The shim will actually use a bit more than the original call:
        #   Original: foo.exe a b c
        #   Shim:     dotslash foo a b c
        # Actually more than the above because "foo" is resolved to an
        # absolute path. So here we test staying a bit below this limit.
        # https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessa#parameters
        MAX_COMMAND_LINE = 32768
        long_arg = "x" * (MAX_COMMAND_LINE - 512)
        ret = subprocess.run(
            [str(self._fixtures / "print_args.exe"), long_arg],
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(ret.stderr, f"1:{long_arg}\n")
        self.assertRegex(ret.stdout, PRINT_ARGS_ARG0)
        self.assertEqual(ret.returncode, 0)

    def test_stdin_to_stdout(self) -> None:
        ret = subprocess.run(
            [str(self._fixtures / "stdin_to_stdout.exe")],
            input="abc",
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(ret.stderr, "")
        self.assertEqual(ret.stdout, "abc")
        self.assertEqual(ret.returncode, 0)

    def test_stdin_to_stderr(self) -> None:
        ret = subprocess.run(
            [str(self._fixtures / "stdin_to_stderr.exe")],
            input="abc",
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(ret.stderr, "abc")
        self.assertEqual(ret.stdout, "")
        self.assertEqual(ret.returncode, 0)

    def test_exit_code(self) -> None:
        ret = subprocess.run(
            [str(self._fixtures / "exit_code.exe"), "64"],
            capture_output=True,
            encoding="utf8",
        )
        self.assertEqual(ret.stderr, "")
        self.assertEqual(ret.stdout, "")
        self.assertEqual(ret.returncode, 64)

    def test_missing_dotslash(self) -> None:
        ret = subprocess.run(
            [str(self._fixtures / "exit_code.exe"), "0"],
            capture_output=True,
            encoding="utf8",
            env=dict(os.environ, PATH="/"),
        )
        self.assertEqual(
            ret.stderr,
            "dotslash-windows-shim: dotslash executable not found.\n",
        )
        self.assertEqual(ret.stdout, "")
        self.assertEqual(ret.returncode, 1)


if __name__ == "__main__":
    unittest.main(verbosity=2)
