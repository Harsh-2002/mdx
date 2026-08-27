use comrak::nodes::AstNode;
use comrak::parse_document;

use crate::options::markdown_options;

pub fn parse_markdown<'a>(
    arena: &'a typed_arena::Arena<AstNode<'a>>,
    input: &str,
) -> &'a AstNode<'a> {
    // Single source of truth for which markdown constructs mdx understands.
    // The fmt and HTML paths build on the same set; see src/options.rs.
    let options = markdown_options();
    parse_document(arena, input, &options)
}
