---
sidebar_position: 28
---

# DotSlash on Windows

DotSlash itself works great on Windows, but comes with some caveats due to
inherent Windows behaviors that might surprise Unix users.

## Shebangs

Typically on Unix, DotSlash files are run by calling them directly. This relies
on them having an exec bit (i.e. `chmod +x`) and letting the operating system
delegate to `dotslash` through the file's
[shebang](<https://en.wikipedia.org/wiki/Shebang_(Unix)>)
(i.e.`#!/usr/bin/env dotslash`). On Windows, this doesn't work because Windows
does not use shebangs, and it relies on the file extension to determine
executability.

There are different ways of dealing with this limitation.

### Explicit Interpreter

The `dotslash` interpreter can be called directly with the DotSlash file as the
first argument:

```
C:\> dotslash path\to\dotslash_file
```

This method works on both Windows and Unix. The drawback (besides being verbose)
is that if the DotSlash file switches to being anything other than a DotSlash
file, then references will have to updated.

### Sibling Batch Script

Create a file with the same name as the DotSlash file, but with an additional
`.bat` (or `.cmd`) extension in the same directory as the DotSlash file. Here is
an example `node.bat` file that would accompany the `node` DotSlash file:

```
@dotslash.exe "%~dpn0" %*
```

- The `@` suppresses echoing the command.
- The `"%~dpn0"` expression corresponds to the `node` DotSlash file.
  - `cmd.exe` expands `%~dpn0` to the script's _Drive_ with the _Path_ that
    contains it plus the file _Name_ (get it? `D` `P` `N`, see
    https://stackoverflow.com/a/5034119).
- The `%*` forwards the arguments passed to the batch script.

With this method you have two files:

```
C:\> type path\to\node
#!/usr/bin/env dotslash
{
  "name": "node-v18.16.0",
  "platforms": {
<<< snip >>>

C:\> type path\to\node.bat
@dotslash.exe "%~dpn0" %*
```

The drawbacks with this method are all the same ones associated with batch
scripts and batch script resolution. The nuances of this are outside the scope
of this documentation.

### DotSlash Windows Shim

**This is the preferred method.** The _DotSlash Windows Shim_ is a tiny `.exe`
executable that is placed next to the DotSlash file that performs the same
function as the [batch script](#sibling-batch-script) above, but is a native
executable rather than a batch script. This is the _ideal_ method that allows
for easy execution without any of the drawbacks of batch scripts. But this
method requires compiling a small executable and keeping it next to the DotSlash
file.

The _DotSlash Windows Shim_ is available under the
[`windows_shim`](https://github.com/facebook/dotslash/tree/main/windows_shim)
folder in the
[DotSlash GitHub repository](https://github.com/facebook/dotslash).

## `MAX_PATH` limits

DotSlash stores fetched artifacts in a cache directory and they're executed from
there. DotSlash tries hard to keep the cache directory path as short as
possible, but in rare cases this might not be enough.

Keeping with the `node` example from above, for the common case, the cache
location of `node.exe` would follow a pattern like:

```
C:\Users\[USERNAME]\AppData\Local\dotslash\[SHARD]\[HASH]\bin\node.exe
```

Where `[SHARD]` is two characters and `[HASH]` is 38 characters.

Again, for the common case, where you also have an 8 character username, the
`node.exe` path length is roughly 99 characters. In this case, this is below the
typical 260 character
[`MAX_PATH`](https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation)
limit. However, it's possible for something like a `node_modules` directory to
reach much larger depths, in which case, the underlying tool must be aware and
be able to handle this.

A possible workaround is to set the `DOTSLASH_CACHE` environment variable to a
shallower directory.

## `argv[0]`

On Unix, `argv[0]` is set to the DotSlash file path. On Windows that's not
possible, so `argv[0]` is the executable in the cache directory.

## Long-lived `dotslash.exe` processes

On Unix, DotSlash uses [`execv`](https://linux.die.net/man/3/execv) to replace
the `dotslash` process with the underlying tool being delegated to from the
cache. On Windows there's no equivalent API, so DotSlash executes the tool but
waits for it to exit before exiting itself. This means that while a tool is
running, there will also be a `dotslash.exe` running. The overhead is minimal
but this presents a challenge when trying to update DotSlash itself.

On Windows, you can't remove a program that is running. So to update
`dotslash.exe` you have to terminate all existing `dotslash.exe` processes. This
can often be done by running `taskkill /f /im dotslash.exe`.

## UNC and Cygwin paths

DotSlash itself is
[UNC path](https://learn.microsoft.com/en-us/dotnet/standard/io/file-path-formats#unc-paths)
aware but the underlying tool might not be. DotSlash tries hard to avoid UNC
paths to ensure maximum compatibility.

DotSlash is not
[Cygwin path](https://cygwin.com/cygwin-ug-net/using.html#cygdrive) aware.
Normally translation of a Cygwin path to a Windows path (i.e.
`/cygdrive/c/path/to/file` to `C:\path\to\file`) is handled by the Cygwin shell
executing DotSlash. But through layers of indirection it's possible to lose
this. In this case, DotSlash does not attempt to convert the path at all.
