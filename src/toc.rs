use comrak::nodes::{AstNode, NodeValue};

use crate::cli::TocArgs;
use crate::parse::parse_markdown;

struct Heading {
    level: u8,
    text: String,
}

pub fn generate_toc(args: &TocArgs) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(&args.file)
        .map_err(|e| format!("Error reading '{}': {}", args.file, e))?;

    let arena = typed_arena::Arena::new();
    let root = parse_markdown(&arena, &content);

    let headings = collect_headings(root, args.depth);

    for heading in &headings {
        let indent = "  ".repeat((heading.level - 1) as usize);
        let slug = slugify(&heading.text);
        println!("{}- [{}](#{})", indent, heading.text, slug);
    }

    Ok(())
}

fn collect_headings<'a>(root: &'a AstNode<'a>, max_depth: u8) -> Vec<Heading> {
    let mut headings = Vec::new();

    for node in root.descendants() {
        let data = node.data.borrow();
        if let NodeValue::Heading(ref heading) = data.value
            && heading.level <= max_depth
        {
            let text = extract_text(node);
            if !text.is_empty() {
                headings.push(Heading {
                    level: heading.level,
                    text,
                });
            }
        }
    }

    headings
}

fn extract_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    for child in node.descendants() {
        let data = child.data.borrow();
        match &data.value {
            NodeValue::Text(t) => text.push_str(t),
            NodeValue::Code(c) => {
                text.push('`');
                text.push_str(&c.literal);
                text.push('`');
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => text.push(' '),
            _ => {}
        }
    }
    text
}

fn slugify(text: &str) -> String {
    // Remove backticks from code spans for the slug
    let clean = text.replace('`', "");
    clean
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c == ' ' || c == '-' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
