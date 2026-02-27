pub mod block;
pub mod code;
pub mod image;
pub mod inline;
pub mod math;
pub mod rule;
pub mod table;

use std::io::Write;

use comrak::nodes::{AlertType, AstNode, ListType, NodeValue, TableAlignment};

use std::path::PathBuf;

use crate::style::Style;
use crate::style::theme::Theme;
use crate::terminal::{ColorLevel, TerminalInfo};

/// Characters used for rendering, with Unicode and ASCII variants.
pub struct Chars {
    pub h1_rule: char,
    pub h2_rule: char,
    pub hr: char,
    pub bq_bar: char,
    pub bullet_l0: &'static str,
    pub bullet_l1: &'static str,
    pub bullet_l2: &'static str,
    pub task_done: &'static str,
    pub task_undone: &'static str,
    pub code_tl: &'static str,
    pub code_tr: &'static str,
    pub code_bl: &'static str,
    pub code_br: &'static str,
    pub code_h: char,
    pub code_v: char,
    pub table_tl: &'static str,
    pub table_tr: &'static str,
    pub table_bl: &'static str,
    pub table_br: &'static str,
    pub table_h: char,
    pub table_v: &'static str,
    pub table_t_down: &'static str,
    pub table_t_up: &'static str,
    pub table_t_right: &'static str,
    pub table_t_left: &'static str,
    pub table_cross: &'static str,
    pub table_header_h: char,
    pub table_header_cross: &'static str,
    pub table_header_left: &'static str,
    pub table_header_right: &'static str,
}

impl Chars {
    pub fn unicode() -> Self {
        Chars {
            h1_rule: '\u{2550}',            // ═
            h2_rule: '\u{2500}',            // ─
            hr: '\u{2500}',                 // ─
            bq_bar: '\u{2502}',             // │
            bullet_l0: "\u{25CF}",          // ●
            bullet_l1: "\u{25CB}",          // ○
            bullet_l2: "\u{25A0}",          // ■
            task_done: "\u{2713}",          // ✓
            task_undone: "\u{2610}",        // ☐
            code_tl: "\u{250C}",            // ┌
            code_tr: "\u{2510}",            // ┐
            code_bl: "\u{2514}",            // └
            code_br: "\u{2518}",            // ┘
            code_h: '\u{2500}',             // ─
            code_v: '\u{2502}',             // │
            table_tl: "\u{250C}",           // ┌
            table_tr: "\u{2510}",           // ┐
            table_bl: "\u{2514}",           // └
            table_br: "\u{2518}",           // ┘
            table_h: '\u{2500}',            // ─
            table_v: "\u{2502}",            // │
            table_t_down: "\u{252C}",       // ┬
            table_t_up: "\u{2534}",         // ┴
            table_t_right: "\u{251C}",      // ├
            table_t_left: "\u{2524}",       // ┤
            table_cross: "\u{253C}",        // ┼
            table_header_h: '\u{2550}',     // ═
            table_header_cross: "\u{256A}", // ╪
            table_header_left: "\u{255E}",  // ╞
            table_header_right: "\u{2561}", // ╡
        }
    }

    pub fn ascii() -> Self {
        Chars {
            h1_rule: '=',
            h2_rule: '-',
            hr: '-',
            bq_bar: '|',
            bullet_l0: "*",
            bullet_l1: "-",
            bullet_l2: "+",
            task_done: "[x]",
            task_undone: "[ ]",
            code_tl: "+",
            code_tr: "+",
            code_bl: "+",
            code_br: "+",
            code_h: '-',
            code_v: '|',
            table_tl: "+",
            table_tr: "+",
            table_bl: "+",
            table_br: "+",
            table_h: '-',
            table_v: "|",
            table_t_down: "+",
            table_t_up: "+",
            table_t_right: "+",
            table_t_left: "+",
            table_cross: "+",
            table_header_h: '=',
            table_header_cross: "+",
            table_header_left: "+",
            table_header_right: "+",
        }
    }
}

/// State for table buffering during AST walk.
pub struct TableState {
    pub rows: Vec<Vec<String>>,
    pub alignments: Vec<TableAlignment>,
    pub in_header: bool,
    pub header_rows: usize,
    pub current_cell: String,
}

impl Default for TableState {
    fn default() -> Self {
        Self::new()
    }
}

impl TableState {
    pub fn new() -> Self {
        TableState {
            rows: Vec::new(),
            alignments: Vec::new(),
            in_header: false,
            header_rows: 0,
            current_cell: String::new(),
        }
    }
}

/// Alert state during rendering.
pub struct AlertState {
    pub alert_type: AlertType,
}

/// Context passed through the rendering process.
pub struct RenderContext<'a> {
    pub term: &'a TerminalInfo,
    pub theme: &'a Theme,
    pub chars: Chars,
    pub style_stack: Vec<Style>,
    pub indent_prefix: String,
    pub indent_level: usize,
    pub list_stack: Vec<ListInfo>,
    pub blockquote_depth: usize,
    pub in_heading: Option<u8>,
    pub heading_text: String,
    pub table_state: Option<TableState>,
    pub in_table_cell: bool,
    pub alert_state: Option<AlertState>,
    pub highlighter: Option<SyntaxHighlighter>,
    pub needs_newline: bool,
    pub in_tight_list: bool,
    pub paragraph_buf: Option<()>,
    pub paragraph_segments: Vec<inline::StyledSegment>,
    pub link_url: Option<String>,
    pub link_text_buf: String,
    pub after_list_marker: bool,
    pub syntax_theme: String,
    pub plain: bool,
    pub strikethrough_fallback: bool,
    pub image_base_dir: Option<PathBuf>,
    pub skip_image_text: bool,
}

pub struct ListInfo {
    pub list_type: ListType,
    pub start: usize,
    pub current: usize,
    pub tight: bool,
}

pub struct SyntaxHighlighter {
    pub syntax_set: syntect::parsing::SyntaxSet,
    pub theme_set: syntect::highlighting::ThemeSet,
}

impl SyntaxHighlighter {
    fn load() -> Self {
        SyntaxHighlighter {
            syntax_set: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            theme_set: syntect::highlighting::ThemeSet::load_defaults(),
        }
    }
}

impl<'a> RenderContext<'a> {
    pub fn new(
        term: &'a TerminalInfo,
        theme: &'a Theme,
        syntax_theme: String,
        plain: bool,
    ) -> Self {
        let chars = if term.unicode {
            Chars::unicode()
        } else {
            Chars::ascii()
        };
        RenderContext {
            term,
            theme,
            chars,
            style_stack: Vec::new(),
            indent_prefix: String::new(),
            indent_level: 0,
            list_stack: Vec::new(),
            blockquote_depth: 0,
            in_heading: None,
            heading_text: String::new(),
            table_state: None,
            in_table_cell: false,
            alert_state: None,
            highlighter: None,
            needs_newline: false,
            in_tight_list: false,
            paragraph_buf: None,
            paragraph_segments: Vec::new(),
            link_url: None,
            link_text_buf: String::new(),
            after_list_marker: false,
            syntax_theme,
            plain,
            strikethrough_fallback: false,
            image_base_dir: None,
            skip_image_text: false,
        }
    }

    pub fn color_level(&self) -> ColorLevel {
        self.term.color_level
    }

    pub fn available_width(&self) -> usize {
        let base = self.term.width as usize;
        let indent = self.current_indent_width();
        base.saturating_sub(indent)
    }

    pub fn current_indent_width(&self) -> usize {
        crate::text::display_width(&self.indent_prefix)
    }

    pub fn current_style(&self) -> Style {
        let mut result = Style::default();
        for s in &self.style_stack {
            result = result.merge(s);
        }
        result
    }

    pub fn ensure_highlighter(&mut self) -> &SyntaxHighlighter {
        if self.highlighter.is_none() {
            self.highlighter = Some(SyntaxHighlighter::load());
        }
        self.highlighter.as_ref().unwrap()
    }

    pub fn write_indent<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        if !self.indent_prefix.is_empty() {
            write!(w, "{}", self.indent_prefix)?;
        }
        Ok(())
    }

    /// Build indent prefix from current blockquote depth and list indentation.
    pub fn rebuild_indent(&mut self) {
        let mut prefix = String::new();
        for i in 0..self.blockquote_depth {
            let bar = self.chars.bq_bar;
            if i > 0 {
                prefix.push(' ');
            }
            prefix.push(bar);
            prefix.push(' ');
        }
        let list_indent = self.indent_level * 2;
        for _ in 0..list_indent {
            prefix.push(' ');
        }
        self.indent_prefix = prefix;
    }
}

/// Render a parsed markdown document to a writer.
pub fn render<'a, W: Write>(
    w: &mut W,
    root: &'a AstNode<'a>,
    ctx: &mut RenderContext<'_>,
) -> std::io::Result<()> {
    use comrak::arena_tree::NodeEdge;

    for edge in root.traverse() {
        match edge {
            NodeEdge::Start(node) => {
                handle_node_start(w, node, ctx)?;
            }
            NodeEdge::End(node) => {
                handle_node_end(w, node, ctx)?;
            }
        }
    }
    Ok(())
}

fn handle_node_start<W: Write>(
    w: &mut W,
    node: &AstNode<'_>,
    ctx: &mut RenderContext<'_>,
) -> std::io::Result<()> {
    let val = &node.data.borrow().value;
    match val {
        NodeValue::Document => {}
        NodeValue::Heading(heading) => {
            block::start_heading(w, ctx, heading.level)?;
        }
        NodeValue::Paragraph => {
            block::start_paragraph(w, ctx)?;
        }
        NodeValue::Text(text) => {
            inline::render_text(w, ctx, text)?;
        }
        NodeValue::SoftBreak => {
            inline::render_soft_break(w, ctx)?;
        }
        NodeValue::LineBreak => {
            inline::render_line_break(w, ctx)?;
        }
        NodeValue::Code(code) => {
            inline::render_inline_code(w, ctx, &code.literal)?;
        }
        NodeValue::Strong => {
            inline::start_strong(ctx);
        }
        NodeValue::Emph => {
            inline::start_emph(ctx);
        }
        NodeValue::Strikethrough => {
            inline::start_strikethrough(ctx);
        }
        NodeValue::Link(link) => {
            inline::start_link(ctx, &link.url);
        }
        NodeValue::Image(link) => {
            inline::start_image(w, ctx, &link.title, &link.url)?;
        }
        NodeValue::ThematicBreak => {
            rule::render_hr(w, ctx)?;
        }
        NodeValue::BlockQuote => {
            block::start_blockquote(ctx);
        }
        NodeValue::List(nl) => {
            block::start_list(ctx, nl.list_type, nl.start, nl.tight);
        }
        NodeValue::Item(item_list) => {
            block::start_list_item(w, ctx, item_list)?;
        }
        NodeValue::TaskItem(task) => {
            let checked = task.symbol.is_some();
            block::render_task_marker(w, ctx, checked)?;
        }
        NodeValue::CodeBlock(cb) => {
            code::start_code_block(w, ctx, &cb.info, &cb.literal)?;
        }
        NodeValue::Table(table) => {
            table::start_table(ctx, &table.alignments);
        }
        NodeValue::TableRow(header) => {
            table::start_table_row(ctx, *header);
        }
        NodeValue::TableCell => {
            table::start_table_cell(ctx);
        }
        NodeValue::FootnoteReference(fnref) => {
            inline::render_footnote_ref(w, ctx, &fnref.name)?;
        }
        NodeValue::FootnoteDefinition(fndef) => {
            block::start_footnote_def(w, ctx, &fndef.name)?;
        }
        NodeValue::Alert(alert) => {
            block::start_alert(w, ctx, &alert.alert_type)?;
        }
        NodeValue::HtmlInline(html) => {
            inline::render_html_inline(w, ctx, html)?;
        }
        NodeValue::HtmlBlock(hb) => {
            write!(w, "{}", hb.literal)?;
        }
        NodeValue::FrontMatter(_) => {
            // Strip frontmatter — don't render
        }
        NodeValue::Math(math) => {
            math::render_math(w, ctx, &math.literal, math.dollar_math, math.display_math)?;
        }
        _ => {}
    }
    Ok(())
}

fn handle_node_end<W: Write>(
    w: &mut W,
    node: &AstNode<'_>,
    ctx: &mut RenderContext<'_>,
) -> std::io::Result<()> {
    let val = &node.data.borrow().value;
    match val {
        NodeValue::Document => {}
        NodeValue::Heading(heading) => {
            block::end_heading(w, ctx, heading.level)?;
        }
        NodeValue::Paragraph => {
            block::end_paragraph(w, ctx)?;
        }
        NodeValue::Strong => {
            inline::end_strong(ctx);
        }
        NodeValue::Emph => {
            inline::end_emph(ctx);
        }
        NodeValue::Strikethrough => {
            inline::end_strikethrough(ctx);
        }
        NodeValue::Link(link) => {
            inline::end_link(w, ctx, &link.url)?;
        }
        NodeValue::Image(_) => {
            inline::end_image(w, ctx)?;
        }
        NodeValue::BlockQuote => {
            block::end_blockquote(ctx);
        }
        NodeValue::List(_) => {
            block::end_list(w, ctx)?;
        }
        NodeValue::Item(_) => {
            block::end_list_item(ctx);
        }
        NodeValue::Table(_) => {
            table::end_table(w, ctx)?;
        }
        NodeValue::TableRow(_) => {
            table::end_table_row(ctx);
        }
        NodeValue::TableCell => {
            table::end_table_cell(ctx);
        }
        NodeValue::FootnoteDefinition(_) => {
            block::end_footnote_def(ctx);
        }
        NodeValue::Alert(_) => {
            block::end_alert(ctx);
        }
        NodeValue::FrontMatter(_) => {}
        NodeValue::Math(_) => {}
        _ => {}
    }
    Ok(())
}
