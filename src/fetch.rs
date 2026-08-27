use std::time::Duration;

use crate::cli::FetchArgs;

/// Maximum response body size (50 MB) — prevents OOM on huge pages.
const MAX_BODY_SIZE: u64 = 50 * 1024 * 1024;

enum FetchResult {
    /// Server provided markdown directly (MFA-enabled site).
    Markdown {
        body: String,
        server_tokens: Option<u64>,
        content_signal: Option<String>,
        final_url: String,
    },
    /// Server returned HTML — needs local conversion.
    Html {
        body: String,
        content_signal: Option<String>,
        final_url: String,
    },
}

fn http_agent() -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        // Without this, .call() returns Err before we can report the status.
        .http_status_as_error(false)
        .build();
    config.into()
}

/// Fetches a URL, extracts content as markdown, and returns it.
/// File output (`-o`) is handled here; terminal rendering is handled by main.
pub fn run(args: &FetchArgs) -> Result<String, Box<dyn std::error::Error>> {
    if !args.url.starts_with("http://") && !args.url.starts_with("https://") {
        return Err("URL must start with http:// or https://".into());
    }

    eprintln!("  Fetching {}...", args.url);
    let result = fetch_content(&args.url)?;

    // Check Content-Signal header
    let content_signal = match &result {
        FetchResult::Markdown { content_signal, .. } => content_signal.clone(),
        FetchResult::Html { content_signal, .. } => content_signal.clone(),
    };
    check_content_signal(content_signal.as_deref());

    let final_url = match &result {
        FetchResult::Markdown { final_url, .. } | FetchResult::Html { final_url, .. } => {
            final_url.clone()
        }
    };

    let (markdown, meta, tokens) = match result {
        FetchResult::Markdown {
            body,
            server_tokens,
            ..
        } => {
            eprintln!("  Server provided markdown directly");
            let token_count = server_tokens.unwrap_or_else(|| crate::estimate_tokens(&body));
            let meta = article_meta_from_front_matter(&body);
            // Strip it, or --metadata emits a second block and comrak reads the
            // first one only.
            let body = crate::frontmatter::strip(&body).to_string();
            (body, meta, token_count)
        }
        FetchResult::Html { body, .. } => {
            let (md, meta) = if args.raw {
                (raw_fallback(&body), extract_meta_only(&body, &final_url))
            } else {
                extract_readable(&body, &final_url)
            };
            let token_count = crate::estimate_tokens(&md);
            (md, meta, token_count)
        }
    };

    if args.tokens {
        eprintln!("  ~{} tokens", tokens);
    }

    let mut output = String::new();

    if args.metadata {
        let token_arg = if args.tokens { Some(tokens) } else { None };
        if let Some(ref m) = meta {
            output.push_str(&format_front_matter(m, &final_url, token_arg));
        } else {
            output.push_str(&format_front_matter(
                &ArticleMeta::default(),
                &final_url,
                token_arg,
            ));
        }
    }

    output.push_str(&markdown);

    if let Some(ref path) = args.output {
        if output.trim().is_empty() {
            return Err(
                format!("Extraction produced no content; refusing to write {}", path).into(),
            );
        }
        if let Some(parent) = std::path::Path::new(path).parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create '{}': {}", parent.display(), e))?;
        }
        std::fs::write(path, &output).map_err(|e| format!("Cannot write '{}': {}", path, e))?;
        eprintln!("  Wrote {}", path);
    }

    Ok(output)
}

fn fetch_content(url: &str) -> Result<FetchResult, Box<dyn std::error::Error>> {
    let agent = http_agent();
    let resp = agent
        .get(url)
        .header("User-Agent", "mdx-cli (https://github.com/Harsh-2002/mdx)")
        .header("Accept", "text/markdown, text/html;q=0.9")
        .call()?;

    let status = resp.status().as_u16();
    if !(200..300).contains(&status) {
        return Err(format!("HTTP {} for {}", status, url).into());
    }

    // Must be read before into_body(), which consumes the extensions holding it.
    let final_url = {
        use ureq::ResponseExt;
        resp.get_uri().to_string()
    };

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    let server_tokens: Option<u64> = resp
        .headers()
        .get("x-markdown-tokens")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse().ok());

    let content_signal: Option<String> = resp
        .headers()
        .get("content-signal")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());

    if content_type.contains("text/markdown") {
        let body = read_body(resp)?;
        return Ok(FetchResult::Markdown {
            body,
            server_tokens,
            content_signal,
            final_url,
        });
    }

    // GitHub serves raw .md as text/plain, and some servers send no type at all.
    let looks_markdown = url_path_is_markdown(url);
    if content_type.contains("text/plain") && looks_markdown {
        let body = read_body(resp)?;
        return Ok(FetchResult::Markdown {
            body,
            server_tokens,
            content_signal,
            final_url,
        });
    }

    let sniff_html = content_type.is_empty();
    if !content_type.contains("text/html")
        && !content_type.contains("application/xhtml")
        && !sniff_html
    {
        return Err(format!(
            "URL returned unsupported content type ({}). Expected text/markdown, text/html, \
             or text/plain for a .md URL.",
            content_type
        )
        .into());
    }

    if sniff_html {
        let body = read_body(resp)?;
        let head = body.trim_start().to_ascii_lowercase();
        if !head.starts_with("<!doctype") && !head.starts_with("<html") {
            return Ok(FetchResult::Markdown {
                body,
                server_tokens,
                content_signal,
                final_url,
            });
        }
        return Ok(FetchResult::Html {
            body,
            content_signal,
            final_url,
        });
    }

    let body = read_body(resp)?;
    Ok(FetchResult::Html {
        body,
        content_signal,
        final_url,
    })
}

fn read_body(resp: ureq::http::Response<ureq::Body>) -> Result<String, Box<dyn std::error::Error>> {
    Ok(resp
        .into_body()
        .with_config()
        .limit(MAX_BODY_SIZE)
        // A page in a legacy encoding, or with a stray invalid byte, must not
        // fail the whole fetch.
        .lossy_utf8(true)
        .read_to_string()?)
}

fn url_path_is_markdown(url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".md") || lower.ends_with(".markdown")
}

/// Check Content-Signal header for ai-input=no directive.
fn check_content_signal(header: Option<&str>) {
    let Some(value) = header else { return };
    for part in value.split(',') {
        let part = part.trim();
        if let Some((key, val)) = part.split_once('=')
            && key.trim().eq_ignore_ascii_case("ai-input")
            && val.trim().eq_ignore_ascii_case("no")
        {
            eprintln!("  Warning: site signals ai-input=no via Content-Signal header");
        }
    }
}

/// Strip dangerous elements from HTML before raw conversion.
fn sanitize_html(html: &str) -> String {
    let mut result = html.to_string();

    // Remove dangerous elements (case-insensitive)
    for tag in &["script", "style", "noscript", "iframe", "object", "embed"] {
        // Remove both opening+content+closing and self-closing variants
        loop {
            let lower = result.to_ascii_lowercase();
            let open = format!("<{}", tag);
            if let Some(start) = lower.find(&open) {
                let close_tag = format!("</{}>", tag);
                if let Some(end_pos) = lower[start..].find(&close_tag) {
                    let end = start + end_pos + close_tag.len();
                    result.replace_range(start..end, "");
                } else {
                    // Self-closing or unclosed — remove to next >
                    if let Some(gt) = result[start..].find('>') {
                        result.replace_range(start..start + gt + 1, "");
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    // Neutralize dangerous URL schemes in href and src attributes
    for attr in &["href", "src"] {
        loop {
            let lower = result.to_ascii_lowercase();
            let mut found = false;
            for scheme in &["javascript:", "data:", "vbscript:"] {
                // Look for attr="scheme..." or attr='scheme...'
                let pattern_dq = format!("{}=\"{}", attr, scheme);
                let pattern_sq = format!("{}='{}", attr, scheme);
                if let Some(pos) = lower.find(&pattern_dq) {
                    let value_start = pos + attr.len() + 2; // skip attr="
                    if let Some(end) = result[value_start..].find('"') {
                        let replacement = format!("{}=\"#\"", attr);
                        result.replace_range(pos..value_start + end + 1, &replacement);
                        found = true;
                        break;
                    }
                } else if let Some(pos) = lower.find(&pattern_sq) {
                    let value_start = pos + attr.len() + 2; // skip attr='
                    if let Some(end) = result[value_start..].find('\'') {
                        let replacement = format!("{}='#'", attr);
                        result.replace_range(pos..value_start + end + 1, &replacement);
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                break;
            }
        }
    }

    result
}

#[derive(Default)]
struct ArticleMeta {
    title: Option<String>,
    byline: Option<String>,
    excerpt: Option<String>,
    published_time: Option<String>,
    image: Option<String>,
    url: Option<String>,
    site_name: Option<String>,
}

/// Metadata without the article body, for --raw. dom_smoothie already parses
/// OpenGraph, Twitter and JSON-LD, so this needs no parser of its own.
fn extract_meta_only(html: &str, url: &str) -> Option<ArticleMeta> {
    let (_, meta) = extract_readable(html, url);
    meta
}

fn extract_readable(html: &str, url: &str) -> (String, Option<ArticleMeta>) {
    let cfg = dom_smoothie::Config {
        text_mode: dom_smoothie::TextMode::Markdown,
        ..Default::default()
    };

    match dom_smoothie::Readability::new(html, Some(url), Some(cfg)) {
        Ok(mut reader) => match reader.parse() {
            Ok(article) => {
                let text = article.text_content.to_string();
                if text.trim().is_empty() {
                    eprintln!(
                        "  Warning: readability returned empty content, falling back to raw conversion"
                    );
                    let md = raw_fallback(html);
                    return (md, None);
                }
                let meta = ArticleMeta {
                    title: if article.title.is_empty() {
                        None
                    } else {
                        Some(article.title.clone())
                    },
                    byline: article.byline.clone(),
                    excerpt: article.excerpt.clone(),
                    published_time: article.published_time.clone(),
                    image: article.image.clone(),
                    url: article.url.clone(),
                    site_name: article.site_name.clone(),
                };
                let mut md = String::new();
                if !article.title.is_empty() {
                    md.push_str(&format!("# {}\n\n", article.title));
                }
                md.push_str(&text);
                (clean_markdown(&md), Some(meta))
            }
            Err(e) => {
                eprintln!(
                    "  Warning: readability extraction failed ({}), falling back to raw conversion",
                    e
                );
                let md = raw_fallback(html);
                (md, None)
            }
        },
        Err(e) => {
            eprintln!(
                "  Warning: readability init failed ({}), falling back to raw conversion",
                e
            );
            let md = raw_fallback(html);
            (md, None)
        }
    }
}

/// Sanitise, then convert. Every path that hands raw page HTML to htmd must go
/// through here: htmd renders <script>/<style> children as text, so an
/// unsanitised fallback turns a JS bundle into article prose.
fn raw_fallback(html: &str) -> String {
    clean_markdown(&convert_raw(&sanitize_html(html)).unwrap_or_default())
}

fn convert_raw(html: &str) -> Result<String, Box<dyn std::error::Error>> {
    let md = htmd::convert(html)?;
    Ok(clean_markdown(&md))
}

/// Remove unnecessary backslash escapes and collapse excessive blank lines.
/// dom_smoothie's markdown mode over-escapes characters like `.`, `(`, `)`,
/// which wastes LLM tokens and hurts readability.
fn clean_markdown(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut consecutive_blanks = 0u32;
    let mut fence = crate::text::FenceTracker::new();

    for line in input.lines() {
        let in_fence = fence.feed(line);

        if in_fence {
            out.push_str(line);
            out.push('\n');
            consecutive_blanks = 0;
            continue;
        }

        if line.trim().is_empty() {
            consecutive_blanks += 1;
            if consecutive_blanks <= 2 {
                out.push('\n');
            }
            continue;
        }
        consecutive_blanks = 0;

        let indent_end = line.len() - line.trim_start().len();
        let mut chars = line.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            if c != '\\' {
                out.push(c);
                continue;
            }
            let Some(&(_, next)) = chars.peek() else {
                out.push(c);
                continue;
            };
            // A backslash in the line's leading run is suppressing block
            // syntax: dropping it turns "1999\. It was" into an ordered list.
            let leading = i <= indent_end + 6
                && line[..i]
                    .trim_start()
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '.');
            let meaningful = matches!(
                next,
                '*' | '_' | '`' | '[' | ']' | '<' | '>' | '~' | '|' | '\\' | '#'
            );
            if meaningful || leading {
                out.push(c);
            }
            out.push(next);
            chars.next();
        }
        out.push('\n');
    }

    let trimmed = out.trim_end();
    let mut result = trimmed.to_string();
    result.push('\n');
    result
}

/// Escape a string value for safe embedding in a YAML double-quoted string.
fn yaml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // A crafted <title> otherwise writes raw ESC/BEL into the file, and
            // `cat` then executes them.
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32))
            }
            '\u{85}' | '\u{2028}' | '\u{2029}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn format_front_matter(meta: &ArticleMeta, url: &str, tokens: Option<u64>) -> String {
    let mut fm = String::from("---\n");
    if let Some(ref title) = meta.title {
        fm.push_str(&format!("title: \"{}\"\n", yaml_escape(title)));
    }
    if let Some(ref byline) = meta.byline {
        fm.push_str(&format!("author: \"{}\"\n", yaml_escape(byline)));
    }
    if let Some(ref date) = meta.published_time {
        fm.push_str(&format!("date: \"{}\"\n", yaml_escape(date)));
    }
    fm.push_str(&format!("source: \"{}\"\n", yaml_escape(url)));
    if let Some(ref excerpt) = meta.excerpt {
        fm.push_str(&format!("excerpt: \"{}\"\n", yaml_escape(excerpt)));
    }
    if let Some(ref image) = meta.image {
        fm.push_str(&format!("image: \"{}\"\n", yaml_escape(image)));
    }
    if let Some(ref og_url) = meta.url {
        fm.push_str(&format!("url: \"{}\"\n", yaml_escape(og_url)));
    }
    if let Some(ref site_name) = meta.site_name {
        fm.push_str(&format!("site_name: \"{}\"\n", yaml_escape(site_name)));
    }
    if let Some(t) = tokens {
        fm.push_str(&format!("tokens: {}\n", t));
    }
    fm.push_str("---\n\n");
    fm
}

/// Shared front-matter parser, so a document yields the same title here as it
/// does in publish and search.
fn article_meta_from_front_matter(markdown: &str) -> Option<ArticleMeta> {
    if !markdown.trim_start().starts_with("---") {
        return None;
    }
    let fm = crate::frontmatter::parse(markdown);
    Some(ArticleMeta {
        title: fm.title,
        byline: fm.author,
        published_time: fm.date,
        excerpt: fm.excerpt,
        image: fm.image,
        site_name: fm.site_name,
        url: fm.url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_strips_script() {
        let html = r#"<p>Hello</p><script>alert("xss")</script><p>World</p>"#;
        let result = sanitize_html(html);
        assert!(!result.to_lowercase().contains("<script"));
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_sanitize_strips_iframe() {
        let html = r#"<p>Text</p><iframe src="https://evil.com"></iframe><p>More</p>"#;
        let result = sanitize_html(html);
        assert!(!result.to_lowercase().contains("<iframe"));
        assert!(result.contains("Text"));
        assert!(result.contains("More"));
    }

    #[test]
    fn test_sanitize_strips_javascript_href() {
        let html = r#"<a href="javascript:alert(1)">Click</a>"#;
        let result = sanitize_html(html);
        assert!(!result.to_lowercase().contains("javascript:"));
        assert!(result.contains("Click"));
    }

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(crate::estimate_tokens(""), 0);
        assert_eq!(crate::estimate_tokens("abcd"), 1);
        assert_eq!(crate::estimate_tokens("abcde"), 2); // 5/4 = 1.25, ceil = 2
        assert_eq!(crate::estimate_tokens("abcdefgh"), 2); // 8/4 = 2
    }

    #[test]
    fn test_article_meta_from_front_matter() {
        let md = "---\ntitle: \"My Page\"\nauthor: \"Jane\"\nimage: \"https://example.com/img.png\"\nsite_name: \"Example\"\n---\n\n# Content";
        let meta = article_meta_from_front_matter(md).unwrap();
        assert_eq!(meta.title.as_deref(), Some("My Page"));
        assert_eq!(meta.byline.as_deref(), Some("Jane"));
        assert_eq!(meta.image.as_deref(), Some("https://example.com/img.png"));
        assert_eq!(meta.site_name.as_deref(), Some("Example"));
    }

    #[test]
    fn test_article_meta_from_front_matter_none() {
        let md = "# No front matter\n\nJust content.";
        assert!(article_meta_from_front_matter(md).is_none());
    }

    #[test]
    fn test_clean_markdown() {
        let input = "# Hello\n\nSome text with escaped \\. period.\n";
        let result = clean_markdown(input);
        assert!(result.contains("# Hello"));
        assert!(result.contains("Some text with escaped . period."));
    }

    #[test]
    fn test_sanitize_strips_style() {
        let html = "<style>.evil { display: none; }</style><p>Safe</p>";
        let result = sanitize_html(html);
        assert!(!result.to_lowercase().contains("<style"));
        assert!(result.contains("Safe"));
    }

    #[test]
    fn test_sanitize_case_insensitive() {
        let html = r#"<SCRIPT>alert(1)</SCRIPT><p>OK</p>"#;
        let result = sanitize_html(html);
        assert!(!result.to_lowercase().contains("<script"));
        assert!(result.contains("OK"));
    }

    #[test]
    fn test_check_content_signal_no_warning() {
        // Should not panic or produce errors
        check_content_signal(None);
        check_content_signal(Some("ai-input=yes"));
        check_content_signal(Some("other=value"));
    }

    #[test]
    fn test_format_front_matter_with_tokens() {
        let meta = ArticleMeta {
            title: Some("Test".to_string()),
            ..Default::default()
        };
        let fm = format_front_matter(&meta, "https://example.com", Some(500));
        assert!(fm.contains("tokens: 500"));
        assert!(fm.contains("title: \"Test\""));
    }

    #[test]
    fn test_format_front_matter_without_tokens() {
        let meta = ArticleMeta {
            title: Some("Test".to_string()),
            ..Default::default()
        };
        let fm = format_front_matter(&meta, "https://example.com", None);
        assert!(!fm.contains("tokens:"));
    }

    #[test]
    fn test_format_front_matter_new_fields() {
        let meta = ArticleMeta {
            title: Some("Test".to_string()),
            image: Some("https://example.com/img.jpg".to_string()),
            url: Some("https://example.com/page".to_string()),
            site_name: Some("Example Site".to_string()),
            ..Default::default()
        };
        let fm = format_front_matter(&meta, "https://example.com", None);
        assert!(fm.contains("image: \"https://example.com/img.jpg\""));
        assert!(fm.contains("url: \"https://example.com/page\""));
        assert!(fm.contains("site_name: \"Example Site\""));
    }
    #[test]
    fn test_clean_markdown_keeps_backslashes_in_code_fences() {
        let input = "```\nr\"\\d+\\s*\"\n```\n";
        let out = clean_markdown(input);
        assert!(out.contains("\\d+"), "regex escapes must survive: {}", out);
        assert!(out.contains("\\s*"), "regex escapes must survive: {}", out);
    }

    #[test]
    fn test_clean_markdown_keeps_blank_lines_in_fences() {
        let input = "```\na\n\n\n\nb\n```\n";
        let out = clean_markdown(input);
        assert!(
            out.contains("a\n\n\n\nb"),
            "diagram spacing must survive: {:?}",
            out
        );
    }

    #[test]
    fn test_clean_markdown_does_not_manufacture_a_list() {
        let out = clean_markdown("1999\\. It was a good year.\n");
        assert!(
            out.starts_with("1999\\."),
            "leading escape must be kept or the line becomes an ol: {:?}",
            out
        );
    }

    #[test]
    fn test_clean_markdown_still_drops_pointless_escapes() {
        let out = clean_markdown("Some text with escaped \\. period.\n");
        assert!(out.contains("escaped . period"), "got: {:?}", out);
    }

    #[test]
    fn test_sanitize_html_is_ascii_offset_safe() {
        // U+0130 grows a byte when lowercased; offsets found in a lowercased
        // copy used to be applied to the original.
        let html = "<p>\u{130}stanbul</p><script>evil()</script><p>after</p>";
        let out = sanitize_html(html);
        assert!(!out.to_lowercase().contains("<script"));
        assert!(out.contains("stanbul"), "content must survive: {}", out);
        assert!(out.contains("after"), "content must survive: {}", out);
    }

    #[test]
    fn test_raw_fallback_strips_scripts() {
        let html = "<html><body><div id=root>App</div>\
                    <script>var SECRET='sk-1';</script></body></html>";
        let md = raw_fallback(html);
        assert!(
            !md.contains("SECRET"),
            "script body must not become prose: {}",
            md
        );
        assert!(md.contains("App"), "real content must survive: {}", md);
    }
    #[test]
    fn test_url_path_is_markdown() {
        assert!(url_path_is_markdown("https://x/README.md"));
        assert!(url_path_is_markdown("https://x/a.MARKDOWN"));
        assert!(url_path_is_markdown("https://x/a.md?raw=1"));
        assert!(url_path_is_markdown("https://x/a.md#top"));
        assert!(!url_path_is_markdown("https://x/index.html"));
        assert!(!url_path_is_markdown("https://x/"));
    }
    #[test]
    fn test_yaml_escape_neutralises_control_characters() {
        let out = yaml_escape("a\u{1b}[31mb\u{7}c");
        assert!(
            !out.contains('\u{1b}'),
            "ESC must not reach the file: {}",
            out
        );
        assert!(
            !out.contains('\u{7}'),
            "BEL must not reach the file: {}",
            out
        );
        assert!(
            out.contains("\\x1b") && out.contains("\\x07"),
            "got: {}",
            out
        );
    }

    #[test]
    fn test_yaml_escape_keeps_ordinary_text() {
        assert_eq!(
            yaml_escape("Caf\u{e9} \u{65e5}\u{672c}"),
            "Caf\u{e9} \u{65e5}\u{672c}"
        );
    }
}
