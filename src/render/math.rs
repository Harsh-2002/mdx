use std::io::Write;

use crate::style::write_ansi_styled;
use crate::text::repeat_char;

use super::RenderContext;
use super::inline::StyledSegment;

/// Unicode substitution table for common LaTeX commands.
const SUBSTITUTIONS: &[(&str, &str)] = &[
    ("\\alpha", "α"),
    ("\\beta", "β"),
    ("\\gamma", "γ"),
    ("\\delta", "δ"),
    ("\\epsilon", "ε"),
    ("\\zeta", "ζ"),
    ("\\eta", "η"),
    ("\\theta", "θ"),
    ("\\iota", "ι"),
    ("\\kappa", "κ"),
    ("\\lambda", "λ"),
    ("\\mu", "μ"),
    ("\\nu", "ν"),
    ("\\xi", "ξ"),
    ("\\pi", "π"),
    ("\\rho", "ρ"),
    ("\\sigma", "σ"),
    ("\\tau", "τ"),
    ("\\upsilon", "υ"),
    ("\\phi", "φ"),
    ("\\chi", "χ"),
    ("\\psi", "ψ"),
    ("\\omega", "ω"),
    ("\\Gamma", "Γ"),
    ("\\Delta", "Δ"),
    ("\\Theta", "Θ"),
    ("\\Lambda", "Λ"),
    ("\\Xi", "Ξ"),
    ("\\Pi", "Π"),
    ("\\Sigma", "Σ"),
    ("\\Phi", "Φ"),
    ("\\Psi", "Ψ"),
    ("\\Omega", "Ω"),
    ("\\sum", "Σ"),
    ("\\prod", "∏"),
    ("\\int", "∫"),
    ("\\partial", "∂"),
    ("\\nabla", "∇"),
    ("\\infty", "∞"),
    ("\\sqrt", "√"),
    ("\\pm", "±"),
    ("\\times", "×"),
    ("\\div", "÷"),
    ("\\cdot", "·"),
    ("\\leq", "≤"),
    ("\\geq", "≥"),
    ("\\neq", "≠"),
    ("\\approx", "≈"),
    ("\\equiv", "≡"),
    ("\\in", "∈"),
    ("\\notin", "∉"),
    ("\\subset", "⊂"),
    ("\\supset", "⊃"),
    ("\\subseteq", "⊆"),
    ("\\supseteq", "⊇"),
    ("\\cup", "∪"),
    ("\\cap", "∩"),
    ("\\forall", "∀"),
    ("\\exists", "∃"),
    ("\\neg", "¬"),
    ("\\wedge", "∧"),
    ("\\vee", "∨"),
    ("\\rightarrow", "→"),
    ("\\leftarrow", "←"),
    ("\\Rightarrow", "⇒"),
    ("\\Leftarrow", "⇐"),
    ("\\leftrightarrow", "↔"),
    ("\\Leftrightarrow", "⇔"),
    ("\\to", "→"),
    ("\\mapsto", "↦"),
    ("\\ldots", "…"),
    ("\\cdots", "⋯"),
    ("\\vdots", "⋮"),
    ("\\ddots", "⋱"),
    ("^0", "⁰"),
    ("^1", "¹"),
    ("^2", "²"),
    ("^3", "³"),
    ("^4", "⁴"),
    ("^5", "⁵"),
    ("^6", "⁶"),
    ("^7", "⁷"),
    ("^8", "⁸"),
    ("^9", "⁹"),
    ("^n", "ⁿ"),
    ("^i", "ⁱ"),
    ("_0", "₀"),
    ("_1", "₁"),
    ("_2", "₂"),
    ("_3", "₃"),
    ("_4", "₄"),
    ("_5", "₅"),
    ("_6", "₆"),
    ("_7", "₇"),
    ("_8", "₈"),
    ("_9", "₉"),
];

/// Apply Unicode substitutions to a LaTeX string.
fn substitute_latex(input: &str) -> String {
    let mut result = input.to_string();
    for &(pattern, replacement) in SUBSTITUTIONS {
        result = result.replace(pattern, replacement);
    }
    // Strip common LaTeX commands that have no good Unicode equivalent
    for cmd in &[
        "\\frac", "\\text", "\\mathrm", "\\mathbf", "\\mathit", "\\left", "\\right",
    ] {
        result = result.replace(cmd, "");
    }
    result
}

/// Render a math node (inline or display) to the terminal.
pub fn render_math<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    literal: &str,
    _dollar_math: bool,
    display_math: bool,
) -> std::io::Result<()> {
    let converted = substitute_latex(literal);

    if display_math {
        render_display_math(w, ctx, &converted)
    } else {
        render_inline_math(w, ctx, &converted)
    }
}

/// Render inline math like `$x^2$` — styled inline code with italic prefix.
fn render_inline_math<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    text: &str,
) -> std::io::Result<()> {
    let style = ctx.current_style().merge(&ctx.theme.inline_code);
    let style = crate::style::Style {
        italic: true,
        ..style
    };
    let formatted = format!(" {} ", text);

    if ctx.paragraph_buf.is_some() {
        ctx.paragraph_segments.push(StyledSegment {
            text: formatted,
            style,
        });
        return Ok(());
    }
    write_ansi_styled(w, &formatted, &style, ctx.color_level())
}

/// Render display math like `$$...$$` — bordered box similar to code blocks.
fn render_display_math<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    text: &str,
) -> std::io::Result<()> {
    if ctx.needs_newline {
        writeln!(w)?;
    }

    if ctx.plain {
        ctx.write_indent(w)?;
        writeln!(w, "    {}", text)?;
        ctx.needs_newline = true;
        return Ok(());
    }

    let width = ctx.available_width();
    let inner_width = width.saturating_sub(2);

    // Top border with "math" label
    ctx.write_indent(w)?;
    let label = " math ";
    let label_len = label.len();
    let remaining = inner_width.saturating_sub(label_len + 1);
    let top_line = format!(
        "{}{}{}{}{}",
        ctx.chars.code_tl,
        repeat_char(ctx.chars.code_h, 1),
        label,
        repeat_char(ctx.chars.code_h, remaining),
        ctx.chars.code_tr,
    );
    write_ansi_styled(w, &top_line, &ctx.theme.code_border, ctx.color_level())?;
    writeln!(w)?;

    // Content lines
    for line in text.lines() {
        ctx.write_indent(w)?;
        let border_v = ctx.chars.code_v.to_string();
        write_ansi_styled(w, &border_v, &ctx.theme.code_border, ctx.color_level())?;
        let line_width = crate::text::display_width(line);
        let math_style = crate::style::Style {
            italic: true,
            ..ctx.theme.inline_code.clone()
        };
        write_ansi_styled(w, line, &math_style, ctx.color_level())?;
        let pad = inner_width.saturating_sub(line_width);
        write!(w, "{}", " ".repeat(pad))?;
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
