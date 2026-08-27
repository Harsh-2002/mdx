use comrak::nodes::{AstNode, NodeValue};

use crate::cli::TocArgs;
use crate::parse::{CodeStyle, inline_text, parse_markdown};

struct Heading {
    level: u8,
    text: String,
    anchor: String,
}

pub fn generate_toc(args: &TocArgs) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(&args.file)
        .map_err(|e| format!("Error reading '{}': {}", args.file, e))?;

    let arena = typed_arena::Arena::new();
    let root = parse_markdown(&arena, &content);

    // Anchor every heading, then filter for display: comrak's Anchorizer
    // deduplicates in document order, so skipping a heading here would shift
    // the "-1"/"-2" suffixes away from the ones the HTML actually emits.
    let headings = collect_headings(root);

    for heading in headings.iter().filter(|h| h.level <= args.depth) {
        let indent = "  ".repeat((heading.level - 1) as usize);
        println!("{}- [{}](#{})", indent, heading.text, heading.anchor);
    }

    Ok(())
}

fn collect_headings<'a>(root: &'a AstNode<'a>) -> Vec<Heading> {
    let mut headings = Vec::new();
    // The same anchorizer comrak uses when rendering HTML, so `mdx toc` links
    // resolve against `--to html`, serve, publish and EPUB output.
    let mut anchorizer = comrak::Anchorizer::new();

    for node in root.descendants() {
        let data = node.data.borrow();
        if let NodeValue::Heading(ref heading) = data.value {
            let text = inline_text(node, CodeStyle::Fenced);
            if !text.is_empty() {
                let anchor = anchorizer.anchorize(&text);
                headings.push(Heading {
                    level: heading.level,
                    text,
                    anchor,
                });
            }
        }
    }

    headings
}
