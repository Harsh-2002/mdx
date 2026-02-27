use comrak::nodes::AstNode;
use comrak::{Options, parse_document};

pub fn parse_markdown<'a>(
    arena: &'a typed_arena::Arena<AstNode<'a>>,
    input: &str,
) -> &'a AstNode<'a> {
    let mut options = Options::default();

    // Enable all GFM extensions
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.alerts = true;
    options.extension.header_ids = None;
    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.extension.math_dollars = true;
    options.extension.math_code = true;

    parse_document(arena, input, &options)
}
