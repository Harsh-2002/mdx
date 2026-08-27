use comrak::nodes::{AstNode, NodeValue};
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

/// How inline code spans are rendered when flattening an AST to text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeStyle {
    /// Keep the backticks, for text that will be shown as markdown again.
    Fenced,
    /// Emit the literal bare, for text that will be compared or embedded.
    Bare,
}

/// Flatten a node's inline content to plain text.
///
/// Replaces three near-identical private walkers that disagreed in ways users
/// could see: lint's dropped inline code entirely, so a heading like
/// `` `--flag` reference `` compared as " reference"; export's ignored
/// `LineBreak`, so words either side of a hard break ran together.
pub fn inline_text<'a>(node: &'a AstNode<'a>, code: CodeStyle) -> String {
    let mut out = String::new();
    for child in node.descendants() {
        let data = child.data.borrow();
        match &data.value {
            NodeValue::Text(t) => out.push_str(t),
            NodeValue::Code(c) => match code {
                CodeStyle::Fenced => {
                    out.push('`');
                    out.push_str(&c.literal);
                    out.push('`');
                }
                CodeStyle::Bare => out.push_str(&c.literal),
            },
            NodeValue::SoftBreak | NodeValue::LineBreak => out.push(' '),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flatten(md: &str, code: CodeStyle) -> String {
        let arena = typed_arena::Arena::new();
        let root = parse_markdown(&arena, md);
        inline_text(root, code)
    }

    #[test]
    fn test_plain_text() {
        assert_eq!(flatten("hello world", CodeStyle::Bare), "hello world");
    }

    #[test]
    fn test_code_style_fenced_keeps_backticks() {
        assert_eq!(flatten("a `b` c", CodeStyle::Fenced), "a `b` c");
    }

    #[test]
    fn test_code_style_bare_drops_backticks() {
        assert_eq!(flatten("a `b` c", CodeStyle::Bare), "a b c");
    }

    #[test]
    fn test_inline_code_is_never_dropped() {
        // lint's walker used to drop it, losing the whole span.
        assert!(flatten("`--flag` reference", CodeStyle::Bare).contains("--flag"));
    }

    #[test]
    fn test_hard_break_becomes_a_space() {
        // export's walker ignored LineBreak, so "one" and "two" ran together.
        let out = flatten(
            "one  
two",
            CodeStyle::Bare,
        );
        assert!(out.contains("one two"), "got: {:?}", out);
    }

    #[test]
    fn test_soft_break_becomes_a_space() {
        assert!(flatten("one\ntwo", CodeStyle::Bare).contains("one two"));
    }

    #[test]
    fn test_emphasis_is_unwrapped() {
        assert_eq!(flatten("**bold** and *it*", CodeStyle::Bare), "bold and it");
    }
}
