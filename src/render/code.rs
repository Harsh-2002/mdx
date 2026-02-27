use std::io::Write;

use crate::style::write_ansi_styled;
use crate::terminal::ColorLevel;
use crate::text::repeat_char;

use super::RenderContext;

pub fn start_code_block<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    info: &str,
    literal: &str,
) -> std::io::Result<()> {
    let lang = info.split_whitespace().next().unwrap_or("");

    render_plain_code_block(w, ctx, lang, literal)
}

/// Render a code block with borders and syntax highlighting.
pub fn render_plain_code_block<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    lang: &str,
    literal: &str,
) -> std::io::Result<()> {
    if ctx.needs_newline {
        writeln!(w)?;
    }

    // Plain mode: no borders, just 4-space indent
    if ctx.plain {
        for line in literal.lines() {
            ctx.write_indent(w)?;
            writeln!(w, "    {}", line)?;
        }
        ctx.needs_newline = true;
        return Ok(());
    }

    let width = ctx.available_width();
    let inner_width = width.saturating_sub(2); // account for side borders

    // Top border with language label
    ctx.write_indent(w)?;
    let label = if lang.is_empty() {
        String::new()
    } else {
        format!(" {} ", lang)
    };

    let label_len = crate::text::display_width(&label);
    let remaining = inner_width.saturating_sub(label_len);

    let top_line = format!(
        "{}{}{}{}",
        ctx.chars.code_tl,
        repeat_char(ctx.chars.code_h, 1),
        label,
        repeat_char(ctx.chars.code_h, remaining.saturating_sub(1)),
    );
    // Trim to width and add corner
    let top_trimmed = if crate::text::display_width(&top_line) >= width {
        let truncated = truncate_ansi_str(&top_line, width.saturating_sub(1));
        format!("{}{}", truncated, ctx.chars.code_tr)
    } else {
        let pad = width.saturating_sub(crate::text::display_width(&top_line) + 1);
        format!(
            "{}{}{}",
            top_line,
            repeat_char(ctx.chars.code_h, pad),
            ctx.chars.code_tr
        )
    };
    write_ansi_styled(w, &top_trimmed, &ctx.theme.code_border, ctx.color_level())?;
    writeln!(w)?;

    // Render code lines with syntax highlighting
    let highlighted_lines = highlight_code(ctx, lang, literal);

    for line in &highlighted_lines {
        ctx.write_indent(w)?;
        let border_v: String = ctx.chars.code_v.to_string();
        write_ansi_styled(w, &border_v, &ctx.theme.code_border, ctx.color_level())?;

        // Pad line or truncate to fit within box
        let line_display_width = strip_ansi_width(line);
        if line_display_width <= inner_width {
            write!(w, "{}", line)?;
            let pad = inner_width - line_display_width;
            write!(w, "{}", " ".repeat(pad))?;
        } else {
            // Truncate to inner_width
            let truncated = truncate_ansi_str(line, inner_width.saturating_sub(1));
            let trunc_width = strip_ansi_width(&truncated);
            write!(w, "{}", truncated)?;
            write!(w, "\u{2026}")?; // … ellipsis
            let pad = inner_width.saturating_sub(trunc_width + 1);
            write!(w, "{}", " ".repeat(pad))?;
        }

        write_ansi_styled(w, &border_v, &ctx.theme.code_border, ctx.color_level())?;
        writeln!(w)?;
    }

    // Bottom border
    ctx.write_indent(w)?;
    let bottom = format!(
        "{}{}{}",
        ctx.chars.code_bl,
        repeat_char(ctx.chars.code_h, width.saturating_sub(2)),
        ctx.chars.code_br,
    );
    write_ansi_styled(w, &bottom, &ctx.theme.code_border, ctx.color_level())?;
    writeln!(w)?;

    ctx.needs_newline = true;
    Ok(())
}

fn highlight_code(ctx: &mut RenderContext<'_>, lang: &str, code: &str) -> Vec<String> {
    if ctx.color_level() == ColorLevel::None || lang.is_empty() {
        // No highlighting, just return lines
        return code.lines().map(|l| l.to_string()).collect();
    }

    // Try syntax highlighting with syntect
    ctx.ensure_highlighter();

    let highlighter = ctx.highlighter.as_ref().unwrap();

    let syntax = highlighter
        .syntax_set
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| highlighter.syntax_set.find_syntax_plain_text());

    let theme = highlighter
        .theme_set
        .themes
        .get(ctx.syntax_theme.as_str())
        .unwrap_or_else(|| {
            highlighter
                .theme_set
                .themes
                .get("base16-ocean.dark")
                .unwrap_or_else(|| highlighter.theme_set.themes.values().next().unwrap())
        });

    let mut h = syntect::easy::HighlightLines::new(syntax, theme);
    let mut result = Vec::new();

    for line in syntect::util::LinesWithEndings::from(code) {
        match h.highlight_line(line, &highlighter.syntax_set) {
            Ok(ranges) => {
                let mut styled_line = String::new();
                for (style, text) in ranges {
                    let text = text.trim_end_matches('\n').trim_end_matches('\r');
                    if text.is_empty() {
                        continue;
                    }
                    let fg = style.foreground;
                    match ctx.color_level() {
                        ColorLevel::TrueColor => {
                            styled_line.push_str(&format!(
                                "\x1b[38;2;{};{};{}m{}\x1b[0m",
                                fg.r, fg.g, fg.b, text
                            ));
                        }
                        ColorLevel::Ansi256 => {
                            let idx = rgb_to_ansi256(fg.r, fg.g, fg.b);
                            styled_line.push_str(&format!("\x1b[38;5;{}m{}\x1b[0m", idx, text));
                        }
                        ColorLevel::Basic => {
                            let code = rgb_to_basic_code(fg.r, fg.g, fg.b);
                            styled_line.push_str(&format!("\x1b[{}m{}\x1b[0m", code, text));
                        }
                        ColorLevel::None => {
                            styled_line.push_str(text);
                        }
                    }
                }
                result.push(styled_line);
            }
            Err(_) => {
                result.push(line.trim_end_matches('\n').to_string());
            }
        }
    }

    result
}

/// Truncate a string (which may contain ANSI codes) to a display width.
fn truncate_ansi_str(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut width = 0;
    let mut has_ansi = false;
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            has_ansi = true;
            // Copy ANSI escape sequence as-is
            result.push(ch);
            while let Some(&c) = chars.peek() {
                result.push(c);
                chars.next();
                if c == 'm' {
                    break;
                }
            }
        } else {
            let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if width + ch_width > max_width {
                break;
            }
            result.push(ch);
            width += ch_width;
        }
    }
    // Reset styling after truncation only if we had ANSI codes
    if has_ansi {
        result.push_str("\x1b[0m");
    }
    result
}

fn strip_ansi_width(s: &str) -> usize {
    let stripped = strip_ansi(s);
    crate::text::display_width(&stripped)
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Skip until 'm' or end
            while let Some(&c) = chars.peek() {
                chars.next();
                if c == 'm' {
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    if r == g && g == b {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return ((r as f64 - 8.0) / 247.0 * 24.0).round() as u8 + 232;
    }
    let ri = (r as f64 / 255.0 * 5.0).round() as u8;
    let gi = (g as f64 / 255.0 * 5.0).round() as u8;
    let bi = (b as f64 / 255.0 * 5.0).round() as u8;
    16 + 36 * ri + 6 * gi + bi
}

fn rgb_to_basic_code(r: u8, g: u8, b: u8) -> u8 {
    let brightness = (r as u16 + g as u16 + b as u16) / 3;
    let is_bright = brightness > 128;
    let max = r.max(g).max(b);
    let threshold = max / 2;
    let has_r = r > threshold;
    let has_g = g > threshold;
    let has_b = b > threshold;

    match (has_r, has_g, has_b, is_bright) {
        (false, false, false, _) => 30,    // black
        (true, false, false, false) => 31, // dark red
        (true, false, false, true) => 91,  // red
        (false, true, false, false) => 32, // dark green
        (false, true, false, true) => 92,  // green
        (false, false, true, false) => 34, // dark blue
        (false, false, true, true) => 94,  // blue
        (true, true, false, false) => 33,  // dark yellow
        (true, true, false, true) => 93,   // yellow
        (true, false, true, false) => 35,  // dark magenta
        (true, false, true, true) => 95,   // magenta
        (false, true, true, false) => 36,  // dark cyan
        (false, true, true, true) => 96,   // cyan
        (true, true, true, false) => 37,   // grey
        (true, true, true, true) => 97,    // white
    }
}
