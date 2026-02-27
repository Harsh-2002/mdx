use std::io::Write;

use super::RenderContext;
use crate::style::{Style, write_ansi_styled};
use crate::text::wrap_text;

/// A styled text segment within a paragraph.
#[derive(Clone)]
pub struct StyledSegment {
    pub text: String,
    pub style: Style,
}

pub fn render_text<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    text: &str,
) -> std::io::Result<()> {
    // Track link text for auto-link detection (always, regardless of buffer mode)
    if ctx.link_url.is_some() {
        ctx.link_text_buf.push_str(text);
    }

    if ctx.in_heading.is_some() {
        ctx.heading_text.push_str(text);
        return Ok(());
    }
    if ctx.in_table_cell {
        if let Some(ref mut ts) = ctx.table_state {
            ts.current_cell.push_str(text);
        }
        return Ok(());
    }

    // If buffering paragraph, collect styled segments
    if ctx.paragraph_buf.is_some() {
        let style = ctx.current_style();
        ctx.paragraph_segments.push(StyledSegment {
            text: text.to_string(),
            style,
        });
        return Ok(());
    }

    let style = ctx.current_style();
    write_ansi_styled(w, text, &style, ctx.color_level())
}

pub fn render_soft_break<W: Write>(w: &mut W, ctx: &mut RenderContext<'_>) -> std::io::Result<()> {
    if ctx.in_heading.is_some() {
        ctx.heading_text.push(' ');
        return Ok(());
    }
    if ctx.in_table_cell {
        if let Some(ref mut ts) = ctx.table_state {
            ts.current_cell.push(' ');
        }
        return Ok(());
    }
    if ctx.paragraph_buf.is_some() {
        ctx.paragraph_segments.push(StyledSegment {
            text: " ".to_string(),
            style: ctx.current_style(),
        });
        return Ok(());
    }
    write!(w, " ")
}

pub fn render_line_break<W: Write>(w: &mut W, ctx: &mut RenderContext<'_>) -> std::io::Result<()> {
    if ctx.in_heading.is_some() {
        ctx.heading_text.push(' ');
        return Ok(());
    }
    if ctx.in_table_cell {
        if let Some(ref mut ts) = ctx.table_state {
            ts.current_cell.push(' ');
        }
        return Ok(());
    }
    if ctx.paragraph_buf.is_some() {
        ctx.paragraph_segments.push(StyledSegment {
            text: "\n".to_string(),
            style: ctx.current_style(),
        });
        return Ok(());
    }
    writeln!(w)?;
    ctx.write_indent(w)
}

pub fn render_inline_code<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    literal: &str,
) -> std::io::Result<()> {
    if ctx.in_heading.is_some() {
        ctx.heading_text.push_str(literal);
        return Ok(());
    }
    if ctx.in_table_cell {
        if let Some(ref mut ts) = ctx.table_state {
            ts.current_cell.push_str(literal);
        }
        return Ok(());
    }
    let style = ctx.current_style().merge(&ctx.theme.inline_code);
    let formatted = format!(" {} ", literal);
    if ctx.paragraph_buf.is_some() {
        ctx.paragraph_segments.push(StyledSegment {
            text: formatted,
            style,
        });
        return Ok(());
    }
    write_ansi_styled(w, &formatted, &style, ctx.color_level())
}

pub fn start_strong(ctx: &mut RenderContext<'_>) {
    ctx.style_stack.push(ctx.theme.bold.clone());
}

pub fn end_strong(ctx: &mut RenderContext<'_>) {
    ctx.style_stack.pop();
}

pub fn start_emph(ctx: &mut RenderContext<'_>) {
    ctx.style_stack.push(ctx.theme.italic.clone());
}

pub fn end_emph(ctx: &mut RenderContext<'_>) {
    ctx.style_stack.pop();
}

pub fn start_strikethrough(ctx: &mut RenderContext<'_>) {
    use crate::terminal::ColorLevel;
    let needs_fallback = ctx.color_level() <= ColorLevel::Basic;
    if needs_fallback {
        // Use ~~ markers as fallback for terminals that don't support CrossedOut
        let marker = StyledSegment {
            text: "~~".to_string(),
            style: ctx.current_style(),
        };
        if ctx.paragraph_buf.is_some() {
            ctx.paragraph_segments.push(marker);
        }
    }
    ctx.style_stack.push(ctx.theme.strikethrough.clone());
    ctx.strikethrough_fallback = needs_fallback;
}

pub fn end_strikethrough(ctx: &mut RenderContext<'_>) {
    ctx.style_stack.pop();
    if ctx.strikethrough_fallback {
        let marker = StyledSegment {
            text: "~~".to_string(),
            style: ctx.current_style(),
        };
        if ctx.paragraph_buf.is_some() {
            ctx.paragraph_segments.push(marker);
        }
        ctx.strikethrough_fallback = false;
    }
}

pub fn start_link(ctx: &mut RenderContext<'_>, url: &str) {
    ctx.style_stack.push(ctx.theme.link_text.clone());
    ctx.link_url = Some(url.to_string());
    ctx.link_text_buf.clear();
    // Emit OSC 8 hyperlink start sequence
    if ctx.term.supports_osc8 {
        let osc8_start = format!("\x1b]8;;{}\x07", url);
        if ctx.paragraph_buf.is_some() {
            ctx.paragraph_segments.push(StyledSegment {
                text: osc8_start,
                style: Style::default(),
            });
        }
    }
}

pub fn end_link<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    url: &str,
) -> std::io::Result<()> {
    ctx.style_stack.pop();

    // Emit OSC 8 hyperlink end sequence (before URL suffix)
    if ctx.term.supports_osc8 {
        let osc8_end = "\x1b]8;;\x07".to_string();
        if ctx.paragraph_buf.is_some() {
            ctx.paragraph_segments.push(StyledSegment {
                text: osc8_end,
                style: Style::default(),
            });
        }
    }

    // Don't show URL if the link text IS the URL (auto-links, mailto)
    let link_text = std::mem::take(&mut ctx.link_text_buf);
    ctx.link_url = None;
    let is_autolink = link_text.trim() == url
        || url
            .strip_prefix("mailto:")
            .is_some_and(|email| link_text.trim() == email);
    if is_autolink {
        return Ok(());
    }
    let style = ctx.theme.link_url.clone();
    let max_url_width = ctx.available_width() * 2 / 3;
    let display_url = crate::text::truncate_url(url, max_url_width.saturating_sub(3));
    let formatted = format!(" ({})", display_url);
    // Buffer if in table cell
    if ctx.in_table_cell {
        if let Some(ref mut ts) = ctx.table_state {
            ts.current_cell.push_str(&formatted);
        }
        return Ok(());
    }
    if ctx.paragraph_buf.is_some() {
        ctx.paragraph_segments.push(StyledSegment {
            text: formatted,
            style,
        });
        return Ok(());
    }
    write_ansi_styled(w, &formatted, &style, ctx.color_level())
}

pub fn start_image<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    _title: &str,
    url: &str,
) -> std::io::Result<()> {
    // Try inline image rendering (iTerm2/Kitty)
    ctx.skip_image_text = false;
    if !ctx.in_table_cell
        && ctx.paragraph_buf.is_none()
        && let Ok(true) = super::image::render_inline_image(w, ctx, url)
    {
        ctx.skip_image_text = true;
        return Ok(());
    }

    if ctx.in_table_cell {
        if let Some(ref mut ts) = ctx.table_state {
            ts.current_cell.push_str("[Image: ");
        }
        return Ok(());
    }
    let style = ctx.theme.image_text.clone();
    if ctx.paragraph_buf.is_some() {
        ctx.paragraph_segments.push(StyledSegment {
            text: "[Image: ".to_string(),
            style,
        });
        return Ok(());
    }
    write_ansi_styled(w, "[Image: ", &style, ctx.color_level())
}

pub fn end_image<W: Write>(w: &mut W, ctx: &mut RenderContext<'_>) -> std::io::Result<()> {
    if ctx.skip_image_text {
        ctx.skip_image_text = false;
        return Ok(());
    }
    if ctx.in_table_cell {
        if let Some(ref mut ts) = ctx.table_state {
            ts.current_cell.push(']');
        }
        return Ok(());
    }
    let style = ctx.theme.image_text.clone();
    if ctx.paragraph_buf.is_some() {
        ctx.paragraph_segments.push(StyledSegment {
            text: "]".to_string(),
            style,
        });
        return Ok(());
    }
    write_ansi_styled(w, "]", &style, ctx.color_level())
}

pub fn render_footnote_ref<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    name: &str,
) -> std::io::Result<()> {
    let style = ctx.theme.footnote_ref.clone();
    let formatted = format!("[{}]", name);
    if ctx.paragraph_buf.is_some() {
        ctx.paragraph_segments.push(StyledSegment {
            text: formatted,
            style,
        });
        return Ok(());
    }
    write_ansi_styled(w, &formatted, &style, ctx.color_level())
}

pub fn render_html_inline<W: Write>(
    w: &mut W,
    _ctx: &mut RenderContext<'_>,
    html: &str,
) -> std::io::Result<()> {
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            write!(w, "{}", ch)?;
        }
    }
    Ok(())
}

/// Flush buffered paragraph segments with word wrapping.
pub fn flush_paragraph<W: Write>(w: &mut W, ctx: &mut RenderContext<'_>) -> std::io::Result<()> {
    let segments = std::mem::take(&mut ctx.paragraph_segments);
    if segments.is_empty() {
        return Ok(());
    }

    // Join all text for wrapping calculation
    let plain: String = segments.iter().map(|s| s.text.as_str()).collect();
    let width = ctx.available_width();
    let wrapped = wrap_text(&plain, width);

    // Now output the wrapped text with styles.
    // Map characters from wrapped text back to styled segments.
    let mut seg_iter = SegmentCharIter::new(&segments);

    for (line_idx, line) in wrapped.lines().enumerate() {
        if line_idx > 0 {
            writeln!(w)?;
            ctx.write_indent(w)?;
        }

        // Output characters of this line, pulling styles from segments
        for ch in line.chars() {
            let style = seg_iter.next_style(ch);
            let s = ch.to_string();
            write_ansi_styled(w, &s, &style, ctx.color_level())?;
        }
    }

    Ok(())
}

/// Iterator that maps characters in the wrapped output back to styled segments.
struct SegmentCharIter {
    plain_chars: Vec<(char, Style)>,
    pos: usize,
}

impl SegmentCharIter {
    fn new(segments: &[StyledSegment]) -> Self {
        let mut plain_chars = Vec::new();
        for seg in segments {
            for ch in seg.text.chars() {
                plain_chars.push((ch, seg.style.clone()));
            }
        }
        SegmentCharIter {
            plain_chars,
            pos: 0,
        }
    }

    fn next_style(&mut self, wrapped_ch: char) -> Style {
        // The wrapped text may have different whitespace due to wrapping.
        // Try to match: if the wrapped char matches the next plain char, advance.
        // If it's a newline added by wrapping, use the current style.
        while self.pos < self.plain_chars.len() {
            let (plain_ch, ref style) = self.plain_chars[self.pos];
            if plain_ch == wrapped_ch {
                self.pos += 1;
                return style.clone();
            }
            // If plain char is whitespace and wrapped char is different whitespace,
            // skip the plain char (wrapping consumed it)
            if plain_ch.is_whitespace() && wrapped_ch != plain_ch {
                self.pos += 1;
                continue;
            }
            break;
        }
        // Fallback: default style
        Style::default()
    }
}
