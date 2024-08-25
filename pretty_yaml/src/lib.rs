#![doc = include_str!("../README.md")]

use crate::{
    config::FormatOptions,
    printer::{Ctx, DocGen},
};
use tiny_pretty::{print, IndentKind, PrintOptions};
use yaml_parser::{
    ast::{AstNode, Root},
    SyntaxError,
};

pub mod config;
mod printer;

/// Format the given source input.
pub fn format_text(input: &str, options: &FormatOptions) -> Result<String, SyntaxError> {
    let syntax = yaml_parser::parse(input)?;
    let root = Root::cast(syntax).expect("expected root node");
    Ok(print_tree(&root, options))
}

/// Print the given concrete syntax tree.
/// You may use this when you already have the parsed CST.
pub fn print_tree(root: &Root, options: &FormatOptions) -> String {
    let ctx = Ctx {
        indent_width: options.layout.indent_width,
        options: &options.language,
    };
    print(
        &root.doc(&ctx),
        &PrintOptions {
            indent_kind: IndentKind::Space,
            line_break: options.layout.line_break.clone().into(),
            width: options.layout.print_width,
            tab_size: options.layout.indent_width,
        },
    )
}
