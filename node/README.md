# DotSlash: simplified executable deployment

[DotSlash](https://dotslash-cli.com/docs/) (`dotslash`) is a command-line tool that lets you represent a set of
platform-specific, heavyweight executables with an equivalent small,
easy-to-read text file. In turn, this makes it efficient to store executables in
source control without hurting repository size. This paves the way for checking
build toolchains and other tools directly into the repo, reducing dependencies
on the host environment and thereby facilitating reproducible builds.

The `fb-dotslash` npm package allows you to use DotSlash in your Node.js projects without having to install DotSlash globally. This is particularly useful for package authors, who have traditionally needed to either include binaries for _all_ platforms or manage their own download and caching in a postinstall script.

## Using DotSlash in an npm package

First, you'll need to write a [DotSlash file](https://dotslash-cli.com/docs/dotslash-file/) that describes the binary you want to distribute.

If your npm package declares `fb-dotslash` as a dependency, any commands executed as part of `npm run` and `npm exec` will have `dotslash` available on the `PATH`. This means you can, for example, directly reference DotSlash files in your `package.json` scripts with no further setup:

```json
{
  "name": "my-package",
  "scripts": {
    "foo": "path/to/dotslash/file"
  },
  "dependencies": {
    "fb-dotslash": "^0.5.8"
  }
}
```

If you need to use `dotslash` in some other context, you can use `require('fb-dotslash')` to get the path to the DotSlash executable appropriate for the current platform:

```js
const dotslash = require('fb-dotslash');
const {spawnSync} = require('child_process');
spawnSync(dotslash, ['path/to/dotslash/file'], {stdio: 'inherit']);
```

## License

DotSlash is licensed under both the MIT license and Apache-2.0 license; the
exact terms can be found in the [LICENSE-MIT](https://github.com/facebook/dotslash/blob/main/LICENSE-MIT) and
[LICENSE-APACHE](https://github.com/facebook/dotslash/blob/main/LICENSE-APACHE) files, respectively.
