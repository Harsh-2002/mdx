use crate::terminal::ColorLevel;
use crossterm::style::Color as CtColor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Rgb(u8, u8, u8),
    Ansi256(u8),
    Black,
    DarkRed,
    DarkGreen,
    DarkYellow,
    DarkBlue,
    DarkMagenta,
    DarkCyan,
    Grey,
    DarkGrey,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Color {
    /// Downgrade a color to the appropriate level for the terminal.
    pub fn for_level(self, level: ColorLevel) -> Option<CtColor> {
        match level {
            ColorLevel::None => None,
            ColorLevel::Basic => Some(self.to_basic()),
            ColorLevel::Ansi256 => Some(self.to_ansi256()),
            ColorLevel::TrueColor => Some(self.to_crossterm()),
        }
    }

    fn to_crossterm(self) -> CtColor {
        match self {
            Color::Rgb(r, g, b) => CtColor::Rgb { r, g, b },
            Color::Ansi256(n) => CtColor::AnsiValue(n),
            Color::Black => CtColor::Black,
            Color::DarkRed => CtColor::DarkRed,
            Color::DarkGreen => CtColor::DarkGreen,
            Color::DarkYellow => CtColor::DarkYellow,
            Color::DarkBlue => CtColor::DarkBlue,
            Color::DarkMagenta => CtColor::DarkMagenta,
            Color::DarkCyan => CtColor::DarkCyan,
            Color::Grey => CtColor::Grey,
            Color::DarkGrey => CtColor::DarkGrey,
            Color::Red => CtColor::Red,
            Color::Green => CtColor::Green,
            Color::Yellow => CtColor::Yellow,
            Color::Blue => CtColor::Blue,
            Color::Magenta => CtColor::Magenta,
            Color::Cyan => CtColor::Cyan,
            Color::White => CtColor::White,
        }
    }

    fn to_ansi256(self) -> CtColor {
        match self {
            Color::Rgb(r, g, b) => CtColor::AnsiValue(rgb_to_ansi256(r, g, b)),
            Color::Ansi256(n) => CtColor::AnsiValue(n),
            _ => self.to_crossterm(),
        }
    }

    fn to_basic(self) -> CtColor {
        match self {
            Color::Rgb(r, g, b) => rgb_to_basic(r, g, b),
            Color::Ansi256(n) => ansi256_to_basic(n),
            _ => self.to_crossterm(),
        }
    }
}

fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // Check if it's a grayscale color
    if r == g && g == b {
        if r < 8 {
            return 16;
        }
        if r > 248 {
            return 231;
        }
        return ((r as f64 - 8.0) / 247.0 * 24.0).round() as u8 + 232;
    }

    // Map to 6x6x6 color cube
    let ri = (r as f64 / 255.0 * 5.0).round() as u8;
    let gi = (g as f64 / 255.0 * 5.0).round() as u8;
    let bi = (b as f64 / 255.0 * 5.0).round() as u8;
    16 + 36 * ri + 6 * gi + bi
}

fn rgb_to_basic(r: u8, g: u8, b: u8) -> CtColor {
    // Simple approach: map to nearest basic color
    let brightness = (r as u16 + g as u16 + b as u16) / 3;
    let is_bright = brightness > 128;

    // Determine dominant channel(s)
    let max = r.max(g).max(b);
    let threshold = max / 2;

    let has_r = r > threshold;
    let has_g = g > threshold;
    let has_b = b > threshold;

    match (has_r, has_g, has_b, is_bright) {
        (false, false, false, _) => CtColor::Black,
        (true, false, false, false) => CtColor::DarkRed,
        (true, false, false, true) => CtColor::Red,
        (false, true, false, false) => CtColor::DarkGreen,
        (false, true, false, true) => CtColor::Green,
        (false, false, true, false) => CtColor::DarkBlue,
        (false, false, true, true) => CtColor::Blue,
        (true, true, false, false) => CtColor::DarkYellow,
        (true, true, false, true) => CtColor::Yellow,
        (true, false, true, false) => CtColor::DarkMagenta,
        (true, false, true, true) => CtColor::Magenta,
        (false, true, true, false) => CtColor::DarkCyan,
        (false, true, true, true) => CtColor::Cyan,
        (true, true, true, false) => CtColor::Grey,
        (true, true, true, true) => CtColor::White,
    }
}

fn ansi256_to_basic(n: u8) -> CtColor {
    match n {
        0 => CtColor::Black,
        1 => CtColor::DarkRed,
        2 => CtColor::DarkGreen,
        3 => CtColor::DarkYellow,
        4 => CtColor::DarkBlue,
        5 => CtColor::DarkMagenta,
        6 => CtColor::DarkCyan,
        7 => CtColor::Grey,
        8 => CtColor::DarkGrey,
        9 => CtColor::Red,
        10 => CtColor::Green,
        11 => CtColor::Yellow,
        12 => CtColor::Blue,
        13 => CtColor::Magenta,
        14 => CtColor::Cyan,
        15 => CtColor::White,
        16..=231 => {
            // Color cube: convert index to RGB, then to basic
            let n = n - 16;
            let b = n % 6;
            let g = (n / 6) % 6;
            let r = n / 36;
            let r8 = if r > 0 { r * 40 + 55 } else { 0 };
            let g8 = if g > 0 { g * 40 + 55 } else { 0 };
            let b8 = if b > 0 { b * 40 + 55 } else { 0 };
            rgb_to_basic(r8, g8, b8)
        }
        232..=255 => {
            // Grayscale ramp
            let level = (n - 232) * 10 + 8;
            if level < 64 {
                CtColor::Black
            } else if level < 128 {
                CtColor::DarkGrey
            } else if level < 192 {
                CtColor::Grey
            } else {
                CtColor::White
            }
        }
    }
}
