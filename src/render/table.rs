use std::io::Write;

use comrak::nodes::TableAlignment;

use crate::style::write_ansi_styled;
use crate::text::{Alignment, display_width, pad_to_width, repeat_char};

use super::{RenderContext, TableState};

pub fn start_table(ctx: &mut RenderContext<'_>, alignments: &[TableAlignment]) {
    ctx.table_state = Some(TableState {
        rows: Vec::new(),
        alignments: alignments.to_vec(),
        in_header: false,
        header_rows: 0,
        current_cell: String::new(),
    });
}

pub fn start_table_row(ctx: &mut RenderContext<'_>, header: bool) {
    if let Some(ref mut ts) = ctx.table_state {
        ts.in_header = header;
        ts.rows.push(Vec::new());
    }
}

pub fn end_table_row(ctx: &mut RenderContext<'_>) {
    if let Some(ref mut ts) = ctx.table_state
        && ts.in_header
    {
        ts.header_rows = ts.rows.len();
    }
}

pub fn start_table_cell(ctx: &mut RenderContext<'_>) {
    ctx.in_table_cell = true;
    if let Some(ref mut ts) = ctx.table_state {
        ts.current_cell.clear();
    }
}

pub fn end_table_cell(ctx: &mut RenderContext<'_>) {
    ctx.in_table_cell = false;
    if let Some(ref mut ts) = ctx.table_state {
        let cell = std::mem::take(&mut ts.current_cell);
        if let Some(row) = ts.rows.last_mut() {
            row.push(cell);
        }
    }
}

pub fn end_table<W: Write>(w: &mut W, ctx: &mut RenderContext<'_>) -> std::io::Result<()> {
    let ts = match ctx.table_state.take() {
        Some(ts) => ts,
        None => return Ok(()),
    };

    if ts.rows.is_empty() {
        return Ok(());
    }

    if ctx.needs_newline {
        writeln!(w)?;
    }

    // Plain mode: simple pipe-separated output
    if ctx.plain {
        return render_plain_table(w, ctx, &ts);
    }

    // Calculate column widths
    let num_cols = ts.rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if num_cols == 0 {
        return Ok(());
    }

    let mut col_widths: Vec<usize> = vec![0; num_cols];
    for row in &ts.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                col_widths[i] = col_widths[i].max(display_width(cell));
            }
        }
    }

    // Ensure minimum column width of 3
    for w_val in &mut col_widths {
        *w_val = (*w_val).max(3);
    }

    // Check total width and shrink if needed
    let border_overhead = num_cols + 1; // vertical bars
    let padding_overhead = num_cols * 2; // 1 space padding each side
    let total_content = col_widths.iter().sum::<usize>();
    let total_width = total_content + border_overhead + padding_overhead;
    let max_width = ctx.available_width();

    if total_width > max_width && total_content > 0 {
        let available_content = max_width.saturating_sub(border_overhead + padding_overhead);
        if available_content > 0 {
            let ratio = available_content as f64 / total_content as f64;
            for w_val in &mut col_widths {
                *w_val = ((*w_val as f64 * ratio).floor() as usize).max(3);
            }
        }
    }

    let alignments = &ts.alignments;

    // Top border: ┌───┬───┐
    ctx.write_indent(w)?;
    render_horizontal_border(
        w,
        ctx,
        &col_widths,
        ctx.chars.table_tl,
        ctx.chars.table_t_down,
        ctx.chars.table_tr,
        ctx.chars.table_h,
    )?;
    writeln!(w)?;

    for (row_idx, row) in ts.rows.iter().enumerate() {
        let is_header = row_idx < ts.header_rows;
        let cell_style = if is_header {
            &ctx.theme.table_header
        } else {
            &ctx.theme.table_cell
        };

        // Wrap each cell's text and compute row height
        let mut wrapped_cells: Vec<Vec<String>> = Vec::new();
        for (col_idx, cell) in row.iter().enumerate() {
            let col_w = col_widths.get(col_idx).copied().unwrap_or(3);
            let lines = wrap_cell(cell, col_w);
            wrapped_cells.push(lines);
        }
        // Fill missing columns
        for _ in row.len()..num_cols {
            wrapped_cells.push(vec![String::new()]);
        }

        let row_height = wrapped_cells
            .iter()
            .map(|c| c.len())
            .max()
            .unwrap_or(1)
            .max(1);

        // Render each visual line of the row
        for line_idx in 0..row_height {
            ctx.write_indent(w)?;
            for (col_idx, cell_lines) in wrapped_cells.iter().enumerate() {
                let col_w = col_widths.get(col_idx).copied().unwrap_or(3);
                let align = alignments
                    .get(col_idx)
                    .copied()
                    .unwrap_or(TableAlignment::None);
                let alignment = match align {
                    TableAlignment::Left | TableAlignment::None => Alignment::Left,
                    TableAlignment::Center => Alignment::Center,
                    TableAlignment::Right => Alignment::Right,
                };

                let line_text = cell_lines.get(line_idx).map(|s| s.as_str()).unwrap_or("");
                let padded = pad_to_width(line_text, col_w, alignment);

                write_ansi_styled(
                    w,
                    ctx.chars.table_v,
                    &ctx.theme.table_border,
                    ctx.color_level(),
                )?;
                write!(w, " ")?;
                write_ansi_styled(w, &padded, cell_style, ctx.color_level())?;
                write!(w, " ")?;
            }
            write_ansi_styled(
                w,
                ctx.chars.table_v,
                &ctx.theme.table_border,
                ctx.color_level(),
            )?;
            writeln!(w)?;
        }

        // Header separator: ╞═══╪═══╡
        if row_idx + 1 == ts.header_rows {
            ctx.write_indent(w)?;
            render_horizontal_border(
                w,
                ctx,
                &col_widths,
                ctx.chars.table_header_left,
                ctx.chars.table_header_cross,
                ctx.chars.table_header_right,
                ctx.chars.table_header_h,
            )?;
            writeln!(w)?;
        }
    }

    // Bottom border: └───┴───┘
    ctx.write_indent(w)?;
    render_horizontal_border(
        w,
        ctx,
        &col_widths,
        ctx.chars.table_bl,
        ctx.chars.table_t_up,
        ctx.chars.table_br,
        ctx.chars.table_h,
    )?;
    writeln!(w)?;

    ctx.needs_newline = true;
    Ok(())
}

fn render_horizontal_border<W: Write>(
    w: &mut W,
    ctx: &RenderContext<'_>,
    col_widths: &[usize],
    left: &str,
    cross: &str,
    right: &str,
    h_char: char,
) -> std::io::Result<()> {
    let style = &ctx.theme.table_border;
    write_ansi_styled(w, left, style, ctx.color_level())?;
    for (i, &col_w) in col_widths.iter().enumerate() {
        let seg = repeat_char(h_char, col_w + 2); // +2 for padding
        write_ansi_styled(w, &seg, style, ctx.color_level())?;
        if i < col_widths.len() - 1 {
            write_ansi_styled(w, cross, style, ctx.color_level())?;
        }
    }
    write_ansi_styled(w, right, style, ctx.color_level())?;
    Ok(())
}

fn render_plain_table<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    ts: &TableState,
) -> std::io::Result<()> {
    let num_cols = ts.rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if num_cols == 0 {
        return Ok(());
    }

    // Calculate column widths
    let mut col_widths: Vec<usize> = vec![0; num_cols];
    for row in &ts.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                col_widths[i] = col_widths[i].max(display_width(cell));
            }
        }
    }
    for w_val in &mut col_widths {
        *w_val = (*w_val).max(3);
    }

    for (row_idx, row) in ts.rows.iter().enumerate() {
        ctx.write_indent(w)?;
        write!(w, "|")?;
        for (col_idx, cell) in row.iter().enumerate() {
            let col_w = col_widths.get(col_idx).copied().unwrap_or(3);
            let padded = pad_to_width(cell, col_w, Alignment::Left);
            write!(w, " {} |", padded)?;
        }
        // Fill missing columns
        for col_idx in row.len()..num_cols {
            let col_w = col_widths.get(col_idx).copied().unwrap_or(3);
            write!(w, " {} |", " ".repeat(col_w))?;
        }
        writeln!(w)?;

        // Header separator
        if row_idx + 1 == ts.header_rows {
            ctx.write_indent(w)?;
            write!(w, "|")?;
            for &col_w in &col_widths {
                write!(w, "-{}-|", "-".repeat(col_w))?;
            }
            writeln!(w)?;
        }
    }

    ctx.needs_newline = true;
    Ok(())
}

/// Wrap cell text into lines that fit within the given column width.
fn wrap_cell(text: &str, col_width: usize) -> Vec<String> {
    if col_width == 0 {
        return vec![text.to_string()];
    }
    let wrapped = textwrap::wrap(text, col_width);
    if wrapped.is_empty() {
        vec![String::new()]
    } else {
        wrapped.into_iter().map(|cow| cow.into_owned()).collect()
    }
}
