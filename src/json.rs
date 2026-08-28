/// Quote and escape a string as a JSON value, including the surrounding quotes.
pub fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\x20' || c == '\x7f' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quotes_and_backslashes() {
        assert_eq!(quote(r#"a"b\c"#), r#""a\"b\\c""#);
    }

    #[test]
    fn test_control_characters() {
        assert_eq!(quote("a\nb\tc\rd"), r#""a\nb\tc\rd""#);
        assert_eq!(quote("\u{1}"), "\"\\u0001\"");
        // DEL is not below 0x20 but is still not safe to emit raw.
        assert_eq!(quote("\u{7f}"), "\"\\u007f\"");
    }

    #[test]
    fn test_unicode_passes_through() {
        assert_eq!(quote("café 日本"), "\"café 日本\"");
    }

    #[test]
    fn test_empty() {
        assert_eq!(quote(""), "\"\"");
    }

    /// A file name carrying a quote used to break serve's SSE payload, and the
    /// browser swallowed the JSON.parse error, so live reload stopped for that
    /// file for the rest of the session.
    #[test]
    fn test_filename_with_a_quote_stays_parseable() {
        let payload = format!(r#"{{"file":{}}}"#, quote(r#"my "draft" notes.md"#));
        assert_eq!(payload, r#"{"file":"my \"draft\" notes.md"}"#);
    }
}
