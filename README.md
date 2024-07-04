# Pretty YAML

Pretty YAML is a semi-tolerant and configurable YAML formatter.

![GitHub Downloads](https://img.shields.io/github/downloads/g-plane/pretty_yaml/latest/plugin.wasm?style=flat-square)

## Getting Started

### dprint

We've provided [dprint](https://dprint.dev/) integration.

Run the command below to add plugin:

```shell
dprint config add g-plane/pretty_yaml
```

After adding the dprint plugin, update your `dprint.json` and add configuration:

```jsonc
{
  // ...
  "yaml": { // <-- the key name here is "yaml", not "pretty_yaml"
    // Pretty YAML config comes here
  },
  "plugins": [
    "https://plugins.dprint.dev/g-plane/pretty_yaml-v0.5.1.wasm"
  ]
}
```

You can also read [dprint CLI documentation](https://dprint.dev/cli/) for using dprint to format files.

## Configuration

Please refer to [Configuration](./docs/config.md).

## Using in Rust

### Formatter

The formatter can be used in Rust. Please read the [documentation](https://docs.rs/pretty_yaml).

### Parser

If you want to use the underlying parser, please refer to the [documentation](https://docs.rs/yaml_parser).

## Credit

Tests come from [Prettier](https://github.com/prettier/prettier/tree/main/tests/format/yaml).

## License

MIT License

Copyright (c) 2024-present Pig Fang
