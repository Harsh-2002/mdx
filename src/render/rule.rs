use std::io::Write;

use crate::style::write_ansi_styled;
use crate::text::repeat_char;

use super::RenderContext;

pub fn render_hr<W: Write>(w: &mut W, ctx: &mut RenderContext<'_>) -> std::io::Result<()> {
    if ctx.needs_newline {
        writeln!(w)?;
    }
    ctx.write_indent(w)?;
    if ctx.plain {
        writeln!(w, "---")?;
    } else {
        let rule = repeat_char(ctx.chars.hr, ctx.available_width());
        write_ansi_styled(w, &rule, &ctx.theme.hr, ctx.color_level())?;
        writeln!(w)?;
    }
    ctx.needs_newline = true;
    Ok(())
}
