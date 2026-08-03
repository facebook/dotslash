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
APIs to stay small. It is meant to be checked into source control next to every
DotSlash file that needs to run on Windows, so the release binaries are only a
few kilobytes.

## Release

The checked-in `dotslash_windows_shim-x86_64.exe` and `dotslash_windows_shim-aarch64.exe` are built from `dotslash_windows_shim.rs`, and `dotslash_windows_linker_stub.exe` is generated as their linker input. Regenerate all three on Windows with:

```shell
py release.py
```

Building both architectures requires their targets to be installed
(`rustup target add x86_64-pc-windows-msvc aarch64-pc-windows-msvc`). Pass a
single target triple to build just one architecture:

```shell
py release.py aarch64-pc-windows-msvc
```

The build is byte-for-byte reproducible. It links with the Rust-bundled
`rust-lld` — which, unlike MSVC's `link.exe`, embeds no toolchain-specific
"Rich" header — and passes `/Brepro` so timestamps are content hashes rather
than wall-clock time. The output therefore depends only on the toolchain pinned
in `rust-toolchain.toml`. The `verify windows shim` GitHub Actions workflow
rebuilds the shim whenever anything under `windows_shim/` changes and fails if
the committed binaries are stale, so regenerate and commit them in the same
change as any edit to the source or a bump of the pinned toolchain.

If you do not have a Windows machine, let CI build the artifacts for you: push your change (or trigger the workflow manually), then download `dotslash_windows_shim-x86_64` and `dotslash_windows_shim-aarch64` from the workflow run. Each contains the freshly built architecture-specific shim and the shared linker stub. Because the build is reproducible, those files are exactly what a local `py release.py` would produce; commit them into `windows_shim/` and re-run the workflow to confirm it passes.

## Testing

```shell
py run_tests.py
```

## Debugging

It may be useful to have the standard library (e.g. `dbg!`) when debugging.
Build with `--no-default-features` (avoids the default `no_std` feature) to have
access to the standard library.

```shell
cargo build --no-default-features
```
