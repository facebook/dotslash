# DotSlash Windows Shim

The _DotSlash Windows Shim_ aims to workaround the fact that Windows does not
support [shebangs](<https://en.wikipedia.org/wiki/Shebang_(Unix)>) and depends
on a file's extension to determine if it is executable.

## How to use it

Place the _DotSlash Windows Shim_ executable next to a DotSlash file with the
same file name as the DotSlash file plus the `.exe` extension. For example, if
the DotSlash file is named `node`, copy the shim executable as `node.exe` into
the same directory as `node`. When `node.exe` is run, it will run `dotslash`
with the sibiling DotSlash file, and forward all arguments and IO streams.

## How it works

The _DotSlash Windows Shim_ does this:

- Gets it own executable name (e.g. `C:\dir\node.exe`) and removes the extention
  (e.g. `C:\dir\node`).
- It takes this path, plus whatever arguments were passed, and runs
  `dotslash C:\dir\node arg1 arg2 ...`.
- Waits to exit and forwards the exit code.

## Binary size

_DotSlash Windows Shim_ builds without a standard library and only uses Windows
APIs. Release binaries are around ~5KB.

## Release

A nightly toolchain is required:

```shell
cargo +nightly build --release
```

Alternatively, though not recommended:

```shell
RUSTC_BOOTSTRAP=1 cargo build --release
```

## Debugging

It may be useful to have the standard library (e.g. `dbg!`) when debugging.
Build with `--no-default-features` (avoids the default `no_std` feature) to have
access to the standard library.

```shell
cargo build --no-default-features
```
