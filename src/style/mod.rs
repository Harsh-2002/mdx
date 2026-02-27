pub mod color;
pub mod theme;

use crate::terminal::ColorLevel;
use color::Color;
use crossterm::style::{self, Attribute, ContentStyle};

#[derive(Debug, Clone, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub dim: bool,
}

impl Style {
    /// Convert to crossterm ContentStyle, downgrading colors as needed.
    pub fn to_content_style(&self, level: ColorLevel) -> ContentStyle {
        let mut cs = ContentStyle::new();
        if let Some(fg) = self.fg
            && let Some(c) = fg.for_level(level)
        {
            cs.foreground_color = Some(c);
        }
        if let Some(bg) = self.bg
            && let Some(c) = bg.for_level(level)
        {
            cs.background_color = Some(c);
        }
        if self.bold {
            cs.attributes.set(Attribute::Bold);
        }
        if self.italic {
            cs.attributes.set(Attribute::Italic);
        }
        if self.underline {
            cs.attributes.set(Attribute::Underlined);
        }
        if self.strikethrough {
            cs.attributes.set(Attribute::CrossedOut);
        }
        if self.dim {
            cs.attributes.set(Attribute::Dim);
        }
        cs
    }

    /// Merge another style on top of this one (overlay).
    pub fn merge(&self, other: &Style) -> Style {
        Style {
            fg: other.fg.or(self.fg),
            bg: other.bg.or(self.bg),
            bold: self.bold || other.bold,
            italic: self.italic || other.italic,
            underline: self.underline || other.underline,
            strikethrough: self.strikethrough || other.strikethrough,
            dim: self.dim || other.dim,
        }
    }
}

/// Write styled text using crossterm's StyledContent for full attribute control.
pub fn write_ansi_styled<W: std::io::Write>(
    w: &mut W,
    text: &str,
    style: &Style,
    level: ColorLevel,
) -> std::io::Result<()> {
    if level == ColorLevel::None {
        return write!(w, "{}", text);
    }

    let cs = style.to_content_style(level);
    let styled = style::StyledContent::new(cs, text);
    write!(w, "{}", styled)
}
