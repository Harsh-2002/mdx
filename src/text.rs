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

/// Truncate to fit within `max_width` display columns, appending `…` (U+2026)
/// if anything was cut. Counts display columns, not bytes or chars, so a line
/// of CJK or emoji lands in the right column.
pub fn truncate_to_width(s: &str, max_width: usize) -> String {
    let width = display_width(s);
    if width <= max_width || max_width < 4 {
        return s.to_string();
    }
    let mut result = String::new();
    let mut w = 0;
    let limit = max_width - 1; // reserve 1 column for …
    for ch in s.chars() {
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

/// Truncate a URL to fit within `max_width` display columns.
pub fn truncate_url(url: &str, max_width: usize) -> String {
    truncate_to_width(url, max_width)
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

/// Tracks fenced-code-block state while scanning markdown line by line.
///
/// Markdown structure cannot be recovered by looking at a line in isolation: a
/// `---` or a `# heading` inside a fenced block is content, not structure.
/// Feed every line in order and consult the return value before treating a line
/// as structural.
#[derive(Debug, Default)]
pub struct FenceTracker {
    /// Fence character and its run length, when open.
    fence: Option<(char, usize)>,
}

impl FenceTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// True while the scanner is inside a fenced code block.
    pub fn in_fence(&self) -> bool {
        self.fence.is_some()
    }

    /// Feed the next line; returns [`in_fence`](Self::in_fence) afterwards.
    ///
    /// A closing fence must use the same character as the opener and be at
    /// least as long, per CommonMark; anything else is content.
    pub fn feed(&mut self, line: &str) -> bool {
        let trimmed = line.trim_start();
        if let Some(fc) = trimmed.chars().next().filter(|c| *c == '`' || *c == '~') {
            let run = trimmed.chars().take_while(|c| *c == fc).count();
            if run >= 3 {
                match self.fence {
                    Some((open_c, open_len)) if open_c == fc && run >= open_len => {
                        self.fence = None
                    }
                    None => self.fence = Some((fc, run)),
                    _ => {}
                }
            }
        }
        self.in_fence()
    }
}

#[cfg(test)]
mod fence_tests {
    use super::*;

    #[test]
    fn test_backtick_fence_open_and_close() {
        let mut f = FenceTracker::new();
        assert!(f.feed("```rust"));
        assert!(f.feed("code"));
        assert!(!f.feed("```"));
        assert!(!f.feed("after"));
    }

    #[test]
    fn test_tilde_fence_is_tracked_separately() {
        let mut f = FenceTracker::new();
        assert!(f.feed("~~~"));
        // A backtick run does not close a tilde fence.
        assert!(f.feed("```"));
        assert!(!f.feed("~~~"));
    }

    #[test]
    fn test_closing_fence_must_be_at_least_as_long() {
        let mut f = FenceTracker::new();
        assert!(f.feed("````"));
        assert!(f.feed("```"), "shorter run must not close the fence");
        assert!(!f.feed("````"));
    }

    #[test]
    fn test_runs_shorter_than_three_are_content() {
        let mut f = FenceTracker::new();
        assert!(!f.feed("`` inline ``"));
        assert!(!f.feed("text"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_to_width_leaves_short_strings_alone() {
        assert_eq!(truncate_to_width("abc", 10), "abc");
        assert_eq!(truncate_to_width("", 10), "");
    }

    /// Byte slicing here used to abort the process: `mdx diff` on any document
    /// containing a wide character panicked on a non-char-boundary index.
    #[test]
    fn test_truncate_to_width_never_splits_a_character() {
        for width in 0..12 {
            let out = truncate_to_width("✅✅✅✅✅✅", width);
            assert!(
                out.chars().count() * 3 >= out.len(),
                "produced invalid UTF-8"
            );
        }
        assert_eq!(truncate_to_width("日本語テキスト", 6), "日本\u{2026}");
    }

    /// Wide characters occupy two columns, so counting chars or bytes puts the
    /// next column in the wrong place.
    #[test]
    fn test_truncate_to_width_counts_display_columns() {
        assert!(display_width(&truncate_to_width("✅✅✅✅✅✅", 8)) <= 8);
        assert!(display_width(&truncate_to_width("abcdefghij", 8)) <= 8);
        assert!(display_width(&truncate_to_width("日本語テキスト日本語", 10)) <= 10);
    }

    #[test]
    fn test_truncate_to_width_marks_the_cut() {
        assert!(truncate_to_width("abcdefghij", 5).ends_with('\u{2026}'));
        assert!(!truncate_to_width("abc", 5).ends_with('\u{2026}'));
    }
}
