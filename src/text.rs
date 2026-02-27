use unicode_width::UnicodeWidthStr;

/// Wrap text to fit within the given width, preserving existing line breaks.
pub fn wrap_text(text: &str, width: usize) -> String {
    if width == 0 {
        return text.to_string();
    }
    textwrap::fill(text, width)
}

/// Get the display width of a string, accounting for Unicode characters.
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Repeat a character to fill a given display width.
pub fn repeat_char(ch: char, count: usize) -> String {
    std::iter::repeat_n(ch, count).collect()
}

/// Truncate a URL to fit within `max_width` display columns.
/// If the URL fits, it is returned as-is.
/// Otherwise it is cut and an `…` (U+2026) is appended.
pub fn truncate_url(url: &str, max_width: usize) -> String {
    let width = display_width(url);
    if width <= max_width || max_width < 4 {
        return url.to_string();
    }
    let mut result = String::new();
    let mut w = 0;
    let limit = max_width - 1; // reserve 1 column for …
    for ch in url.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw > limit {
            break;
        }
        result.push(ch);
        w += cw;
    }
    result.push('\u{2026}');
    result
}

/// Pad a string to a given display width with spaces.
pub fn pad_to_width(s: &str, width: usize, align: Alignment) -> String {
    let current = display_width(s);
    if current >= width {
        return s.to_string();
    }
    let padding = width - current;
    match align {
        Alignment::Left => format!("{}{}", s, " ".repeat(padding)),
        Alignment::Right => format!("{}{}", " ".repeat(padding), s),
        Alignment::Center => {
            let left = padding / 2;
            let right = padding - left;
            format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
}
