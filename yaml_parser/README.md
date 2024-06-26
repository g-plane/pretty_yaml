# yaml_parser

[![Crates.io](https://img.shields.io/crates/v/yaml_parser?style=flat-square)](https://crates.io/crates/yaml_parser)
[![docs.rs](https://img.shields.io/docsrs/yaml_parser?style=flat-square)](https://docs.rs/yaml_parser)

Semi-tolerant YAML concrete syntax tree parser.

## Usage

```rust
match yaml_parser::parse(&code) {
    Ok(tree) => println!("{tree:#?}"),
    Err(err) => eprintln!("{err}"),
};
```

It produces rowan tree if succeeded.
For consuming the tree, see [rowan's docs](https://docs.rs/rowan).

## Tests

Tests come from [official test suite](https://github.com/yaml/yaml-test-suite).

## License

MIT License

Copyright (c) 2024-present Pig Fang
