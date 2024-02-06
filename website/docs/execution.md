---
sidebar_position: 20
---

# How DotSlash Works

Here is a flowchart that demonstrates what happens when a user invokes
`./scripts/node --version` on the command line when `./scripts/node` is a
DotSlash file. Note this is what happens on Mac and Linux, as the behavior on
[Windows](../windows) is slightly different.

```mermaid
flowchart TD
    USER_INVOCATION(<code>./scripts/node --version</code>) -->SHEBANG_EXPANSION
    SHEBANG_EXPANSION(<code>/usr/local/bin/dotslash ./scripts/node --version) -->|DotSlash parses <code>./scripts/node</code> to build the <code>exec</code> invocation| EXEC
    EXEC{{<code>exec $DOTSLASH_CACHE/fe/40b2ce9a.../node --version</code>}} -->|<code>exec</code> fails with <code>ENOENT</code> because<br>the artifact is not in the cache| ACQUIRE_LOCK
    EXEC -->|artifact in cache| EXEC_SUCCEEDS
    EXEC_SUCCEEDS(<code>exec</code> replaces <code>dotslash</code> process)
    ACQUIRE_LOCK(acquire file lock for artifact) -->F
    F{{check if artifact exists in cache}} -->|No| FETCH
    F -->|Yes: the artifact must have<br>been fetched by another caller<br>while we were waiting<br>to acquire the lock| RELEASE_LOCK
    FETCH(fetch artifact using <code>providers</code><br>in the DotSlash file) --> ON_FETCH
    ON_FETCH(verify artifact size and hash) --> DECOMPRESS
    DECOMPRESS(decompress artifact in temp directory) --> SANITIZE
    SANITIZE(sanitize temp directory) --> RENAME
    RENAME(<code>mv</code> temp directory to final destination) --> RELEASE_LOCK
    RELEASE_LOCK(release file lock) --> EXEC_TAKE2
    EXEC_TAKE2(<code>exec $DOTSLASH_CACHE/fe/40b2ce9a.../node --version</code>)
    EXEC_TAKE2 --> EXEC_SUCCEEDS
```
