use crate::{ctx::Ctx, printer::DocGen};
use tiny_pretty::print;
use yaml_parser::ast::{AstNode, Root};

mod ctx;
mod printer;

/// Format the given source input.
pub fn format_text(input: &str) -> String {
    let syntax = yaml_parser::parse(input).unwrap();
    let root = Root::cast(syntax).unwrap();
    print_tree(&root)
}

/// Print the given concrete syntax tree.
/// You may use this when you already have the parsed CST.
pub fn print_tree(root: &Root) -> String {
    let ctx = Ctx { indent_width: 2 };
    let doc = root.doc(&ctx);
    print(&doc, &Default::default())
}
