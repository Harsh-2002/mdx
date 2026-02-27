use std::io::Write;

use comrak::nodes::{AlertType, ListType, NodeList};

use crate::style::write_ansi_styled;
use crate::text::{repeat_char, wrap_text};

use super::{AlertState, ListInfo, RenderContext};

pub fn start_heading<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    level: u8,
) -> std::io::Result<()> {
    ctx.in_heading = Some(level);
    ctx.heading_text.clear();

    if ctx.needs_newline {
        writeln!(w)?;
    }

    // H1 gets a rule above (skip in plain mode)
    if level == 1 && !ctx.plain {
        ctx.write_indent(w)?;
        let rule = repeat_char(ctx.chars.h1_rule, ctx.available_width());
        write_ansi_styled(w, &rule, &ctx.theme.heading_rule, ctx.color_level())?;
        writeln!(w)?;
    }

    Ok(())
}

pub fn end_heading<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    level: u8,
) -> std::io::Result<()> {
    let text = std::mem::take(&mut ctx.heading_text);
    ctx.in_heading = None;

    let style = match level {
        1 => &ctx.theme.h1,
        2 => &ctx.theme.h2,
        3 => &ctx.theme.h3,
        4 => &ctx.theme.h4,
        5 => &ctx.theme.h5,
        _ => &ctx.theme.h6,
    };

    let display_text = if level == 1 && !ctx.plain {
        text.to_uppercase()
    } else {
        text
    };

    // Plain mode: prefix with # markers
    let prefix = if ctx.plain {
        format!("{} ", "#".repeat(level as usize))
    } else {
        String::new()
    };

    // Wrap text
    let full_text = format!("{}{}", prefix, display_text);
    let wrapped = wrap_text(&full_text, ctx.available_width());

    for line in wrapped.lines() {
        ctx.write_indent(w)?;
        write_ansi_styled(w, line, style, ctx.color_level())?;
        writeln!(w)?;
    }

    // H1 gets rule below, H2 gets thinner rule below (skip in plain mode)
    if !ctx.plain {
        match level {
            1 => {
                ctx.write_indent(w)?;
                let rule = repeat_char(ctx.chars.h1_rule, ctx.available_width());
                write_ansi_styled(w, &rule, &ctx.theme.heading_rule, ctx.color_level())?;
                writeln!(w)?;
            }
            2 => {
                ctx.write_indent(w)?;
                let rule = repeat_char(ctx.chars.h2_rule, ctx.available_width());
                write_ansi_styled(w, &rule, &ctx.theme.heading_rule, ctx.color_level())?;
                writeln!(w)?;
            }
            _ => {}
        }
    }

    ctx.needs_newline = true;
    Ok(())
}

pub fn start_paragraph<W: Write>(w: &mut W, ctx: &mut RenderContext<'_>) -> std::io::Result<()> {
    if ctx.in_table_cell {
        return Ok(());
    }
    // After a list marker, the first paragraph continues on the same line
    if ctx.after_list_marker {
        ctx.after_list_marker = false;
    } else {
        if ctx.needs_newline && !ctx.in_tight_list {
            writeln!(w)?;
        }
        ctx.write_indent(w)?;
    }
    // Enable paragraph buffering for word wrapping
    ctx.paragraph_buf = Some(());
    ctx.paragraph_segments.clear();
    Ok(())
}

pub fn end_paragraph<W: Write>(w: &mut W, ctx: &mut RenderContext<'_>) -> std::io::Result<()> {
    if ctx.in_table_cell {
        return Ok(());
    }
    // Flush buffered paragraph with word wrapping
    if ctx.paragraph_buf.is_some() {
        super::inline::flush_paragraph(w, ctx)?;
        ctx.paragraph_buf = None;
    }
    writeln!(w)?;
    ctx.needs_newline = true;
    Ok(())
}

pub fn start_blockquote(ctx: &mut RenderContext<'_>) {
    ctx.blockquote_depth += 1;
    ctx.rebuild_indent();
    ctx.style_stack.push(ctx.theme.blockquote_text.clone());
}

pub fn end_blockquote(ctx: &mut RenderContext<'_>) {
    ctx.blockquote_depth = ctx.blockquote_depth.saturating_sub(1);
    ctx.rebuild_indent();
    ctx.style_stack.pop();
}

pub fn start_list(ctx: &mut RenderContext<'_>, list_type: ListType, start: usize, tight: bool) {
    ctx.list_stack.push(ListInfo {
        list_type,
        start,
        current: start,
        tight,
    });
    if tight {
        ctx.in_tight_list = true;
    }
}

pub fn end_list<W: Write>(w: &mut W, ctx: &mut RenderContext<'_>) -> std::io::Result<()> {
    let was_tight = ctx.list_stack.last().is_some_and(|l| l.tight);
    ctx.list_stack.pop();
    // Check if we're still in a tight list
    ctx.in_tight_list = ctx.list_stack.last().is_some_and(|l| l.tight);

    if ctx.list_stack.is_empty() && !ctx.needs_newline {
        writeln!(w)?;
        ctx.needs_newline = true;
    }
    let _ = was_tight;
    Ok(())
}

pub fn start_list_item<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    _item: &NodeList,
) -> std::io::Result<()> {
    if ctx.needs_newline && !ctx.in_tight_list {
        writeln!(w)?;
    }

    let depth = ctx.list_stack.len().saturating_sub(1);
    let indent = "  ".repeat(depth);

    // Build blockquote prefix
    let mut bq_prefix = String::new();
    for i in 0..ctx.blockquote_depth {
        if i > 0 {
            bq_prefix.push(' ');
        }
        bq_prefix.push(ctx.chars.bq_bar);
        bq_prefix.push(' ');
    }

    if let Some(list_info) = ctx.list_stack.last_mut() {
        match list_info.list_type {
            ListType::Bullet => {
                let bullet = match depth % 3 {
                    0 => ctx.chars.bullet_l0,
                    1 => ctx.chars.bullet_l1,
                    _ => ctx.chars.bullet_l2,
                };
                write!(w, "{}{}", bq_prefix, indent)?;
                write_ansi_styled(w, bullet, &ctx.theme.list_bullet, ctx.color_level())?;
                write!(w, " ")?;
            }
            ListType::Ordered => {
                let num = list_info.current;
                list_info.current += 1;
                write!(w, "{}{}", bq_prefix, indent)?;
                let num_str = format!("{}. ", num);
                write_ansi_styled(w, &num_str, &ctx.theme.list_bullet, ctx.color_level())?;
            }
        }
    }

    // Set indent for subsequent lines within this item
    let total_indent = depth + 1;
    ctx.indent_level = total_indent;
    ctx.rebuild_indent();
    ctx.after_list_marker = true;

    Ok(())
}

pub fn end_list_item(ctx: &mut RenderContext<'_>) {
    let depth = ctx.list_stack.len().saturating_sub(1);
    ctx.indent_level = depth;
    ctx.rebuild_indent();
    ctx.needs_newline = false; // Item already ended with newline from paragraph
}

pub fn render_task_marker<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    checked: bool,
) -> std::io::Result<()> {
    if checked {
        write_ansi_styled(
            w,
            ctx.chars.task_done,
            &ctx.theme.task_done,
            ctx.color_level(),
        )?;
    } else {
        write_ansi_styled(
            w,
            ctx.chars.task_undone,
            &ctx.theme.task_undone,
            ctx.color_level(),
        )?;
    }
    write!(w, " ")
}

pub fn start_footnote_def<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    name: &str,
) -> std::io::Result<()> {
    if ctx.needs_newline {
        writeln!(w)?;
    }
    ctx.write_indent(w)?;
    let label = format!("[{}]: ", name);
    write_ansi_styled(w, &label, &ctx.theme.footnote_ref, ctx.color_level())?;
    ctx.indent_level += 1;
    ctx.rebuild_indent();
    Ok(())
}

pub fn end_footnote_def(ctx: &mut RenderContext<'_>) {
    ctx.indent_level = ctx.indent_level.saturating_sub(1);
    ctx.rebuild_indent();
}

pub fn start_alert<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    alert_type: &AlertType,
) -> std::io::Result<()> {
    if ctx.needs_newline {
        writeln!(w)?;
    }

    let (label, style) = match alert_type {
        AlertType::Note => ("Note", &ctx.theme.alert_note),
        AlertType::Tip => ("Tip", &ctx.theme.alert_tip),
        AlertType::Important => ("Important", &ctx.theme.alert_important),
        AlertType::Warning => ("Warning", &ctx.theme.alert_warning),
        AlertType::Caution => ("Caution", &ctx.theme.alert_caution),
    };

    // Render alert header with colored bar
    ctx.write_indent(w)?;
    let bar = ctx.chars.bq_bar;
    write_ansi_styled(w, &format!("{} ", bar), style, ctx.color_level())?;
    write_ansi_styled(w, label, style, ctx.color_level())?;
    writeln!(w)?;

    ctx.alert_state = Some(AlertState {
        alert_type: *alert_type,
    });
    ctx.blockquote_depth += 1;
    ctx.rebuild_indent();

    Ok(())
}

pub fn end_alert(ctx: &mut RenderContext<'_>) {
    ctx.alert_state = None;
    ctx.blockquote_depth = ctx.blockquote_depth.saturating_sub(1);
    ctx.rebuild_indent();
}
