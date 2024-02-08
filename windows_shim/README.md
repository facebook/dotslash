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

The current "reference implementation" is rather large. A no_std version that
uses only Windows APIs will be added soon. Stay tuned!
