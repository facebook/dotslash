---
sidebar_position: 16
---

# Generating DotSlash files Using GitHub Actions

As a convenience to your users, you may want to generate DotSlash files for the
executables you publish as part of a
[GitHub release](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases).
The
[dotslash-publish-release](https://github.com/facebook/dotslash-publish-release)
GitHub action is designed to help automate this.

The
[action's README](https://github.com/facebook/dotslash-publish-release#readme)
should have the most up-to-date documentation, but the general idea is that you
run the action after the artifacts for _all_ of your supported platforms are
generated, specifying the `workflow_run.workflows` as appropriate for your
release process:

```yaml
name: Generate DotSlash files

on:
  workflow_run:
    # These must match the names of the workflows that publish
    # artifacts to your GitHub release.
    workflows: [linux-release, macos-release, windows-release]
    types:
      - completed

jobs:
  generate-dotslash-files:
    name: Generating and uploading DotSlash files
    runs-on: ubuntu-latest
    if: ${{ github.event.workflow_run.conclusion == 'success' }}
    steps:
      - uses: facebook/dotslash-publish-release@v1
        # This is necessary because the action uses
        # `gh release upload` to publish the generated DotSlash file(s)
        # as part of the release.
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          # Additional file that lives in your repo that defines
          # how your DotSlash file(s) should be generated.
          config: .github/workflows/dotslash-config.json
          # Tag for the release to to target.
          tag: ${{ github.event.workflow_run.head_branch }}
```

The bulk of the configuration is not in your `.yml` file, but in a JSON file
specified by the `config` parameter to the action. For example, if this GitHub
action were defined in the [facebook/hermes](https://github.com/facebook/hermes)
repository on GitHub, and the contents of
`.github/workflows/dotslash-config.json` were as follows:

```json
{
  "outputs": {
    "hermes": {
      "platforms": {
        "macos-x86_64": {
          "regex": "^hermes-cli-darwin-",
          "path": "hermes"
        },
        "macos-aarch64": {
          "regex": "^hermes-cli-darwin-",
          "path": "hermes"
        },
        "linux-x86_64": {
          "regex": "^hermes-cli-linux-",
          "path": "hermes"
        },
        "windows-x86_64": {
          "regex": "^hermes-cli-windows-",
          "path": "hermes.exe"
        }
      }
    }
  }
}
```

Then this action would have added the following DotSlash file named `hermes` to
the [v0.12.0 release](https://github.com/facebook/hermes/releases/tag/v0.12.0):

```json
#!/usr/bin/env dotslash

{
  "name": "hermes",
  "platforms": {
    "macos-x86_64": {
      "size": 10600817,
      "hash": "blake3",
      "digest": "25f984911f199f9229ca0327c52700fa9a8db9aefe95e84f91ba6be69902436a",
      "format": "tar.gz",
      "path": "hermes",
      "providers": [
        {
          "url": "https://github.com/facebook/hermes/releases/download/v0.12.0/hermes-cli-darwin-v0.12.0.tar.gz"
        },
        {
          "type": "github-release",
          "repo": "https://github.com/facebook/hermes",
          "tag": "v0.12.0",
          "name": "hermes-cli-darwin-v0.12.0.tar.gz"
        }
      ]
    },
    "macos-aarch64": {
      "size": 10600817,
      "hash": "blake3",
      "digest": "25f984911f199f9229ca0327c52700fa9a8db9aefe95e84f91ba6be69902436a",
      "format": "tar.gz",
      "path": "hermes",
      "providers": [
        {
          "url": "https://github.com/facebook/hermes/releases/download/v0.12.0/hermes-cli-darwin-v0.12.0.tar.gz"
        },
        {
          "type": "github-release",
          "repo": "https://github.com/facebook/hermes",
          "tag": "v0.12.0",
          "name": "hermes-cli-darwin-v0.12.0.tar.gz"
        }
      ]
    },
    "linux-x86_64": {
      "size": 47099598,
      "hash": "blake3",
      "digest": "8d2c1bcefc2ce6e278167495810c2437e8050780ebb4da567811f1d754ad198c",
      "format": "tar.gz",
      "path": "hermes",
      "providers": [
        {
          "url": "https://github.com/facebook/hermes/releases/download/v0.12.0/hermes-cli-linux-v0.12.0.tar.gz"
        },
        {
          "type": "github-release",
          "repo": "https://github.com/facebook/hermes",
          "tag": "v0.12.0",
          "name": "hermes-cli-linux-v0.12.0.tar.gz"
        }
      ]
    },
    "windows-x86_64": {
      "size": 17456100,
      "hash": "blake3",
      "digest": "7efee4f92a05e34ccfa7c21c7a05f939d8b724bc802423d618db22efb83bfe1b",
      "format": "tar.gz",
      "path": "hermes.exe",
      "providers": [
        {
          "url": "https://github.com/facebook/hermes/releases/download/v0.12.0/hermes-cli-windows-v0.12.0.tar.gz"
        },
        {
          "type": "github-release",
          "repo": "https://github.com/facebook/hermes",
          "tag": "v0.12.0",
          "name": "hermes-cli-windows-v0.12.0.tar.gz"
        }
      ]
    }
  }
}
```

As you can see, the `"platforms"` in the JSON config mirrors the `"platforms"`
in the generated DotSlash file. Each key in the `"outputs"` map in the JSON file
adds a generated DotSlash file with the same name to the GitHub release. This
makes it possible to generate an arbitrary number of DotSlash files from a
single invocation of the GitHub action.
