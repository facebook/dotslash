# DotSlash Dev Container Feature

A feature to install DotSlash into a Dev Container.

It is installed at `/usr/local/bin/dotslash`.

## Example Usage

```json
"features": {
  "ghcr.io/meta/devcontainers/features/dotslash:1": {
    "version": "latest"
  }
}
```

## Options

| Options Id | Description                         | Type   | Default Value |
| ---------- | ----------------------------------- | ------ | ------------- |
| version    | The version of DotSlash to install. | string | latest        |
