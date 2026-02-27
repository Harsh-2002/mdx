use std::collections::HashSet;

use comrak::nodes::{AstNode, NodeValue};

use crate::parse::parse_markdown;

pub struct LintArgs {
    pub file: String,
}

struct Issue {
    line: usize,
    message: String,
}

pub fn run(args: &LintArgs) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(&args.file)
        .map_err(|e| format!("Error reading '{}': {}", args.file, e))?;

    let arena = typed_arena::Arena::new();
    let root = parse_markdown(&arena, &content);

    let file_dir = std::path::Path::new(&args.file)
        .parent()
        .unwrap_or(std::path::Path::new("."));

    let mut issues = Vec::new();

    check_broken_links(root, file_dir, &mut issues);
    check_duplicate_headings(root, &mut issues);
    check_image_alt_text(root, &mut issues);
    check_trailing_whitespace(&content, &mut issues);

    if issues.is_empty() {
        eprintln!("  No issues found");
        return Ok(());
    }

    issues.sort_by_key(|i| i.line);
    for issue in &issues {
        println!("  {}:{} {}", args.file, issue.line, issue.message);
    }
    eprintln!(
        "  {} issue{} found",
        issues.len(),
        if issues.len() == 1 { "" } else { "s" }
    );
    std::process::exit(1);
}

fn check_broken_links<'a>(
    root: &'a AstNode<'a>,
    base_dir: &std::path::Path,
    issues: &mut Vec<Issue>,
) {
    for node in root.descendants() {
        let data = node.data.borrow();
        if let NodeValue::Link(ref link) = data.value {
            let url = &link.url;
            // Only check relative paths (not URLs, not anchors)
            if !url.starts_with("http://")
                && !url.starts_with("https://")
                && !url.starts_with('#')
                && !url.starts_with("mailto:")
            {
                // Strip anchor fragment
                let path_part = url.split('#').next().unwrap_or(url);
                if !path_part.is_empty() {
                    let target = base_dir.join(path_part);
                    if !target.exists() {
                        issues.push(Issue {
                            line: data.sourcepos.start.line,
                            message: format!("broken link: {}", url),
                        });
                    }
                }
            }
        }
    }
}

fn check_duplicate_headings<'a>(root: &'a AstNode<'a>, issues: &mut Vec<Issue>) {
    let mut seen = HashSet::new();
    for node in root.descendants() {
        let data = node.data.borrow();
        if let NodeValue::Heading(_) = data.value {
            let text = extract_text(node);
            if !text.is_empty() && !seen.insert(text.clone()) {
                issues.push(Issue {
                    line: data.sourcepos.start.line,
                    message: format!("duplicate heading: \"{}\"", text),
                });
            }
        }
    }
}

fn check_image_alt_text<'a>(root: &'a AstNode<'a>, issues: &mut Vec<Issue>) {
    for node in root.descendants() {
        let data = node.data.borrow();
        if let NodeValue::Image(ref img) = data.value {
            let alt = extract_text(node);
            if alt.trim().is_empty() && img.title.is_empty() {
                issues.push(Issue {
                    line: data.sourcepos.start.line,
                    message: "image missing alt text".to_string(),
                });
            }
        }
    }
}

fn check_trailing_whitespace(content: &str, issues: &mut Vec<Issue>) {
    for (i, line) in content.lines().enumerate() {
        if line.len() > 1 && line.ends_with(' ') && !line.ends_with("  ") {
            // Single trailing space (not a deliberate line break which is two spaces)
            issues.push(Issue {
                line: i + 1,
                message: "trailing whitespace".to_string(),
            });
        }
    }
}

fn extract_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    for child in node.descendants() {
        let data = child.data.borrow();
        if let NodeValue::Text(ref t) = data.value {
            text.push_str(t);
        }
    }
    text
}
