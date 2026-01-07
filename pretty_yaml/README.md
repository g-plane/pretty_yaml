`pretty_yaml` is a semi-tolerant and configurable YAML formatter.

## Basic Usage

You can format source code string by using [`format_text`] function.

```rust
use pretty_yaml::{config::FormatOptions, format_text};

let options = FormatOptions::default();
assert_eq!("- a\n- b\n", &format_text("-  a\n-     b", &options).unwrap());
```

For detailed documentation of configuration,
please refer to [Configuration](https://pretty-yaml.netlify.app/).

If there're syntax errors in source code, it will return `Err`:

```rust
use pretty_yaml::{config::FormatOptions, format_text};

let options = FormatOptions::default();
assert!(format_text("{", &options).is_err());
```

## Print Syntax Tree

If you have already parsed the syntax tree with [`yaml_parser`](https://docs.rs/yaml_parser),
you can use [`print_tree`] to print it.

```rust
use pretty_yaml::{config::FormatOptions, print_tree};
use rowan::ast::AstNode;
use yaml_parser::{ast::Root, parse};

let input = "-  a\n-     b";
let tree = parse(input).unwrap();
let root = Root::cast(tree).unwrap();

let options = FormatOptions::default();
assert_eq!("- a\n- b\n", &print_tree(&root, &options));
```
