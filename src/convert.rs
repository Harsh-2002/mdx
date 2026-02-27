use comrak::nodes::{AstNode, NodeValue};

use crate::cli::ThemeName;
use crate::parse::parse_markdown;

pub struct ConvertArgs {
    pub file: Option<String>,
    pub to: String,
}

pub fn run(args: &ConvertArgs) -> Result<(), Box<dyn std::error::Error>> {
    let content = read_input(&args.file)?;
    let title = args.file.as_deref().unwrap_or("document");

    match args.to.as_str() {
        "html" => {
            let html = crate::html::render_standalone(
                &content,
                "base16-ocean.dark",
                &ThemeName::Dark,
                title,
                "",
            );
            print!("{}", html);
        }
        "json" => {
            let arena = typed_arena::Arena::new();
            let root = parse_markdown(&arena, &content);
            let json = ast_to_json(root, 0);
            println!("{}", json);
        }
        "txt" => {
            let arena = typed_arena::Arena::new();
            let root = parse_markdown(&arena, &content);
            let text = extract_plain_text(root);
            print!("{}", text);
        }
        other => {
            return Err(format!(
                "Unsupported format: '{}'. Supported: html, json, txt",
                other
            )
            .into());
        }
    }

    Ok(())
}

fn read_input(file: &Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    match file {
        Some(path) => Ok(std::fs::read_to_string(path)
            .map_err(|e| format!("Error reading '{}': {}", path, e))?),
        None => {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

fn ast_to_json<'a>(node: &'a AstNode<'a>, depth: usize) -> String {
    let data = node.data.borrow();
    let indent = "  ".repeat(depth);
    let indent1 = "  ".repeat(depth + 1);

    let node_type = match &data.value {
        NodeValue::Document => "document",
        NodeValue::Heading(_) => "heading",
        NodeValue::Paragraph => "paragraph",
        NodeValue::Text(_) => "text",
        NodeValue::Code(_) => "code",
        NodeValue::CodeBlock(_) => "code_block",
        NodeValue::Link(_) => "link",
        NodeValue::Image(_) => "image",
        NodeValue::List(_) => "list",
        NodeValue::Item(_) => "item",
        NodeValue::BlockQuote => "blockquote",
        NodeValue::ThematicBreak => "thematic_break",
        NodeValue::Table(_) => "table",
        NodeValue::TableRow(_) => "table_row",
        NodeValue::TableCell => "table_cell",
        NodeValue::Emph => "emphasis",
        NodeValue::Strong => "strong",
        NodeValue::Strikethrough => "strikethrough",
        NodeValue::SoftBreak => "softbreak",
        NodeValue::LineBreak => "linebreak",
        NodeValue::HtmlBlock(_) => "html_block",
        NodeValue::HtmlInline(_) => "html_inline",
        NodeValue::FrontMatter(_) => "front_matter",
        NodeValue::FootnoteDefinition(_) => "footnote_definition",
        NodeValue::FootnoteReference(_) => "footnote_reference",
        NodeValue::Math(_) => "math",
        NodeValue::Alert(_) => "alert",
        _ => "other",
    };

    let mut props = Vec::new();

    match &data.value {
        NodeValue::Text(t) => props.push(format!("{}\"value\": {}", indent1, json_escape(t))),
        NodeValue::Code(c) => {
            props.push(format!("{}\"value\": {}", indent1, json_escape(&c.literal)))
        }
        NodeValue::CodeBlock(cb) => {
            props.push(format!("{}\"info\": {}", indent1, json_escape(&cb.info)));
            props.push(format!(
                "{}\"literal\": {}",
                indent1,
                json_escape(&cb.literal)
            ));
        }
        NodeValue::Heading(h) => {
            props.push(format!("{}\"level\": {}", indent1, h.level));
        }
        NodeValue::Link(link) => {
            props.push(format!("{}\"url\": {}", indent1, json_escape(&link.url)));
            props.push(format!(
                "{}\"title\": {}",
                indent1,
                json_escape(&link.title)
            ));
        }
        NodeValue::Image(img) => {
            props.push(format!("{}\"url\": {}", indent1, json_escape(&img.url)));
            props.push(format!("{}\"title\": {}", indent1, json_escape(&img.title)));
        }
        NodeValue::FrontMatter(fm) => {
            props.push(format!("{}\"value\": {}", indent1, json_escape(fm)));
        }
        _ => {}
    }

    let children: Vec<String> = node
        .children()
        .map(|child| ast_to_json(child, depth + 2))
        .collect();

    let mut parts = vec![format!("{}\"type\": \"{}\"", indent1, node_type)];
    parts.extend(props);

    if !children.is_empty() {
        parts.push(format!(
            "{}\"children\": [\n{}\n{}]",
            indent1,
            children.join(",\n"),
            indent1
        ));
    }

    format!("{}{{\n{}\n{}}}", indent, parts.join(",\n"), indent)
}

fn json_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('"');
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c < '\x20' => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

fn extract_plain_text<'a>(root: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    let mut last_was_block = false;

    for node in root.descendants() {
        let data = node.data.borrow();
        match &data.value {
            NodeValue::Text(t) => {
                text.push_str(t);
                last_was_block = false;
            }
            NodeValue::Code(c) => {
                text.push_str(&c.literal);
                last_was_block = false;
            }
            NodeValue::CodeBlock(cb) => {
                if last_was_block {
                    text.push('\n');
                }
                text.push_str(&cb.literal);
                last_was_block = true;
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                text.push('\n');
                last_was_block = false;
            }
            NodeValue::Paragraph => {
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push_str("\n\n");
                }
                last_was_block = true;
            }
            NodeValue::Heading(_) => {
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push_str("\n\n");
                }
                last_was_block = true;
            }
            _ => {}
        }
    }

    // Trim trailing whitespace but keep one newline
    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{}\n", trimmed)
    }
}
