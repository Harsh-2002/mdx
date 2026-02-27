use comrak::nodes::{AstNode, NodeValue};

use crate::parse::parse_markdown;

pub struct StatsArgs {
    pub file: Option<String>,
}

struct DocStats {
    words: usize,
    lines: usize,
    chars: usize,
    headings: usize,
    links: usize,
    code_blocks: usize,
    images: usize,
}

pub fn run(args: &StatsArgs) -> Result<(), Box<dyn std::error::Error>> {
    let content = read_input(&args.file)?;

    let arena = typed_arena::Arena::new();
    let root = parse_markdown(&arena, &content);
    let stats = collect_stats(root, &content);

    let reading_time = stats.words.div_ceil(238);

    println!("     Words: {}", format_number(stats.words));
    println!("     Lines: {}", format_number(stats.lines));
    println!("     Chars: {}", format_number(stats.chars));
    println!("  Headings: {}", stats.headings);
    println!("     Links: {}", stats.links);
    println!("    Images: {}", stats.images);
    println!("Code blocks: {}", stats.code_blocks);
    println!("  Reading time: ~{} min", reading_time.max(1));

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

fn collect_stats<'a>(root: &'a AstNode<'a>, raw: &str) -> DocStats {
    let mut stats = DocStats {
        words: 0,
        lines: raw.lines().count(),
        chars: raw.len(),
        headings: 0,
        links: 0,
        code_blocks: 0,
        images: 0,
    };

    for node in root.descendants() {
        let data = node.data.borrow();
        match &data.value {
            NodeValue::Heading(_) => stats.headings += 1,
            NodeValue::Link(_) => stats.links += 1,
            NodeValue::Image(_) => stats.images += 1,
            NodeValue::CodeBlock(_) => stats.code_blocks += 1,
            NodeValue::Text(t) => {
                stats.words += t.split_whitespace().count();
            }
            _ => {}
        }
    }

    stats
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
