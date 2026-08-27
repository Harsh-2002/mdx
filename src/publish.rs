use std::path::Path;

use crate::html;
use crate::html::assets::CSS;

pub struct PublishArgs {
    pub dir: String,
    pub out: String,
}

struct Post {
    title: String,
    date: String,
    slug: String,
    description: String,
    markdown: String,
}

pub fn run(args: &PublishArgs) -> Result<(), Box<dyn std::error::Error>> {
    let dir = Path::new(&args.dir);
    if !dir.is_dir() {
        return Err(format!("'{}' is not a directory", args.dir).into());
    }

    let out = Path::new(&args.out);
    std::fs::create_dir_all(out)?;

    // Collect markdown files
    let mut posts = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file()
                && e.path()
                    .extension()
                    .is_some_and(|ext| ext == "md" || ext == "markdown")
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in &entries {
        let path = entry.path();
        let content = std::fs::read_to_string(&path)?;
        let fm = crate::frontmatter::parse(&content);

        if fm.draft {
            continue;
        }

        let markdown = crate::frontmatter::strip(&content).to_string();
        let filename = path.file_stem().unwrap().to_string_lossy().to_string();
        let slug = filename.clone();

        let title = fm.title.unwrap_or_else(|| {
            extract_first_heading(&markdown).unwrap_or_else(|| filename.replace('-', " "))
        });

        let date = fm.date.unwrap_or_else(|| {
            path.metadata()
                .and_then(|m| m.modified())
                .map(|t| {
                    let dt: chrono_lite::DateTime = t.into();
                    dt.date()
                })
                .unwrap_or_else(|_| "".to_string())
        });

        let description = extract_description(&markdown);

        posts.push(Post {
            title,
            date,
            slug,
            description,
            markdown,
        });
    }

    if posts.is_empty() {
        return Err(format!("No markdown files found in '{}'", args.dir).into());
    }

    // Sort by date (newest first)
    posts.sort_by(|a, b| b.date.cmp(&a.date));

    // Write shared CSS
    let css_path = out.join("style.css");
    std::fs::write(&css_path, blog_css())?;

    // Copy assets directory if it exists
    let assets_src = dir.join("assets");
    if assets_src.is_dir() {
        let assets_dst = out.join("assets");
        copy_dir_recursive(&assets_src, &assets_dst)?;
    }

    // Generate individual post pages
    for post in &posts {
        let post_dir = out.join(&post.slug);
        std::fs::create_dir_all(&post_dir)?;

        let page_html = render_blog_page(post);
        std::fs::write(post_dir.join("index.html"), page_html)?;
    }

    // Generate index page
    let index_html = render_blog_index(&posts);
    std::fs::write(out.join("index.html"), index_html)?;

    eprintln!("  Built {} pages -> {}/", posts.len(), args.out);
    eprintln!("  Index page -> {}/index.html", args.out);

    Ok(())
}

fn extract_first_heading(markdown: &str) -> Option<String> {
    for line in markdown.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            return Some(heading.trim().to_string());
        }
    }
    None
}

fn extract_description(markdown: &str) -> String {
    let mut desc = String::new();
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !desc.is_empty() {
                break;
            }
            continue;
        }
        // Skip headings, images, code blocks, front matter
        if trimmed.starts_with('#')
            || trimmed.starts_with("![")
            || trimmed.starts_with("```")
            || trimmed.starts_with("---")
        {
            continue;
        }
        if !desc.is_empty() {
            desc.push(' ');
        }
        desc.push_str(trimmed);
    }
    truncate_description(&desc, MAX_DESCRIPTION_CHARS)
}

/// Max length of a generated meta description, in characters (not bytes).
const MAX_DESCRIPTION_CHARS: usize = 160;

/// Truncate to `max_chars`, preferring a word boundary. UTF-8 safe.
///
/// Slicing at a fixed *byte* offset panics whenever that offset lands inside a
/// multi-byte sequence, which any accented, CJK, or emoji character can cause.
fn truncate_description(desc: &str, max_chars: usize) -> String {
    // Find the byte offset of the character at `max_chars`; `None` means the
    // string is already short enough.
    let Some((cut, _)) = desc.char_indices().nth(max_chars) else {
        return desc.to_string();
    };
    let truncated = &desc[..cut];
    // Try to break at a word boundary. `rfind` returns the byte index of an
    // ASCII space, which is always a valid char boundary.
    match truncated.rfind(' ') {
        Some(last_space) => format!("{}...", &truncated[..last_space]),
        None => format!("{}...", truncated),
    }
}

fn render_blog_page(post: &Post) -> String {
    let body = html::render_fragment(&post.markdown, "base16-ocean.dark");

    format!(
        r#"<!DOCTYPE html>
<html data-theme="dark">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    <meta name="description" content="{description}">
    <meta property="og:title" content="{title}">
    <meta property="og:description" content="{description}">
    {FAVICON}
    <link rel="stylesheet" href="../style.css">
    {KATEX_CSS}
    {KATEX_JS}
    <script>{JS_THEME_EARLY}</script>
</head>
<body>
    <button id="theme-toggle"></button>
    <article class="markdown-body blog-post">
        <header class="post-header">
            <a href="../" class="back-link">&larr; Back</a>
            <h1 class="post-title">{title}</h1>
            <time class="post-date">{date}</time>
        </header>
        {body}
    </article>
    <script type="module">
        import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs';
        mermaid.initialize({{ startOnLoad: false, theme: 'base', themeVariables: getMermaidThemeVars() }});
        document.querySelectorAll('code.language-mermaid').forEach(el => {{
            const pre = el.parentElement;
            const div = document.createElement('div');
            div.className = 'mermaid';
            const src = el.textContent;
            div.textContent = src;
            div.setAttribute('data-original', src);
            pre.replaceWith(div);
        }});
        await mermaid.run();
        window.mermaid = mermaid;
    </script>
    <script>{JS_INIT}</script>
    <script>{JS_KATEX}</script>
</body>
</html>"#,
        title = html_escape(&post.title),
        description = html_escape(&post.description),
        date = html_escape(&post.date),
        body = body,
        FAVICON = html::assets::FAVICON,
        KATEX_CSS = html::assets::KATEX_CSS,
        KATEX_JS = html::assets::KATEX_JS,
        JS_THEME_EARLY = html::assets::JS_THEME_EARLY,
        JS_INIT = html::assets::JS_INIT,
        JS_KATEX = html::assets::JS_KATEX,
    )
}

fn render_blog_index(posts: &[Post]) -> String {
    let mut cards = String::new();
    for post in posts {
        cards.push_str(&format!(
            r#"<a class="blog-card" href="{slug}/">
    <h2 class="blog-card-title">{title}</h2>
    <time class="blog-card-date">{date}</time>
    <p class="blog-card-desc">{desc}</p>
</a>
"#,
            slug = html_escape(&post.slug),
            title = html_escape(&post.title),
            date = html_escape(&post.date),
            desc = html_escape(&post.description),
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html data-theme="dark">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Blog</title>
    {FAVICON}
    <link rel="stylesheet" href="style.css">
    <script>{JS_THEME_EARLY}</script>
</head>
<body>
    <button id="theme-toggle"></button>
    <article class="markdown-body blog-index">
        <h1>Posts</h1>
        <div class="blog-grid">{cards}</div>
    </article>
    <script>{JS_INIT}</script>
</body>
</html>"#,
        cards = cards,
        FAVICON = html::assets::FAVICON,
        JS_THEME_EARLY = html::assets::JS_THEME_EARLY,
        JS_INIT = html::assets::JS_INIT,
    )
}

fn blog_css() -> String {
    format!(
        r#"{base_css}

/* Blog-specific styles */
.blog-index {{
    padding-top: 2rem;
}}

.blog-grid {{
    display: grid;
    gap: 1.5rem;
    margin-top: 1.5rem;
}}

.blog-card {{
    display: block;
    padding: 1.5rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    text-decoration: none;
    color: var(--fg);
    transition: border-color 0.2s, transform 0.2s, box-shadow 0.2s;
}}

.blog-card:hover {{
    border-color: var(--accent);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    text-decoration: none;
}}

.blog-card-title {{
    margin: 0 0 0.25em;
    font-size: 1.25em;
}}

.blog-card-date {{
    display: block;
    font-size: 0.85rem;
    color: var(--fg-muted);
    margin-bottom: 0.5em;
}}

.blog-card-desc {{
    margin: 0;
    color: var(--fg-muted);
    font-size: 0.9rem;
    line-height: 1.5;
}}

.post-header {{
    margin-bottom: 2rem;
}}

.back-link {{
    font-size: 0.9rem;
    color: var(--fg-muted);
    text-decoration: none;
}}

.back-link:hover {{
    color: var(--accent);
}}

.post-title {{
    margin-top: 0.5em;
    border-bottom: none;
    padding-bottom: 0;
}}

.post-date {{
    display: block;
    color: var(--fg-muted);
    font-size: 0.9rem;
}}

@media print {{
    body {{ background: white; color: black; padding: 0; }}
    pre {{ border: 1px solid #ccc; page-break-inside: avoid; }}
    table {{ page-break-inside: avoid; }}
    h1, h2, h3, h4, h5, h6 {{ page-break-after: avoid; }}
    img {{ page-break-inside: avoid; }}
    #theme-toggle {{ display: none !important; }}
}}
"#,
        base_css = CSS
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Minimal DateTime conversion from SystemTime - avoids external chrono dependency
mod chrono_lite {
    pub struct DateTime {
        secs_since_epoch: i64,
    }

    impl From<std::time::SystemTime> for DateTime {
        fn from(t: std::time::SystemTime) -> Self {
            let secs = t
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            DateTime {
                secs_since_epoch: secs,
            }
        }
    }

    impl DateTime {
        pub fn date(&self) -> String {
            // Simple date calculation
            let secs = self.secs_since_epoch;
            let days = secs / 86400;
            // Algorithm from https://howardhinnant.github.io/date_algorithms.html
            let z = days + 719468;
            let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
            let doe = (z - era * 146097) as u32;
            let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
            let y = (yoe as i64) + era * 400;
            let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
            let mp = (5 * doy + 2) / 153;
            let d = doy - (153 * mp + 2) / 5 + 1;
            let m = if mp < 10 { mp + 3 } else { mp - 9 };
            let y = if m <= 2 { y + 1 } else { y };
            format!("{:04}-{:02}-{:02}", y, m, d)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_description_multibyte_boundary() {
        // 200 × "日" = 600 bytes. The old byte-index slice at 160 landed inside
        // the 54th character and panicked; a char-aware one must not.
        let markdown = "日".repeat(200);
        let desc = extract_description(&markdown);
        assert!(desc.ends_with("..."), "long text should be truncated");
        assert_eq!(
            desc.chars().count(),
            163,
            "160 chars plus the ellipsis, got: {}",
            desc
        );
    }

    #[test]
    fn test_extract_description_emoji_boundary() {
        // 4-byte scalars offset by one ASCII byte, so no fixed byte offset
        // lands neatly between two of them.
        let markdown = format!("a{}", "🎉".repeat(200));
        let desc = extract_description(&markdown);
        assert!(desc.ends_with("..."));
    }

    #[test]
    fn test_extract_description_short_text_untouched() {
        let desc = extract_description("A short description with café and 日本.");
        assert_eq!(desc, "A short description with café and 日本.");
        assert!(!desc.ends_with("..."));
    }

    #[test]
    fn test_extract_description_breaks_on_word_boundary() {
        let markdown = "word ".repeat(60); // 300 bytes, ASCII
        let desc = extract_description(&markdown);
        assert!(
            desc.ends_with("word..."),
            "should cut at a space, not mid-word, got: {}",
            desc
        );
    }

    #[test]
    fn test_truncate_description_counts_chars_not_bytes() {
        // 160 three-byte characters is 480 bytes but exactly the limit, so it
        // must survive untruncated.
        let s = "日".repeat(160);
        assert_eq!(truncate_description(&s, 160), s);
        // One more character crosses the limit.
        let s = "日".repeat(161);
        assert!(truncate_description(&s, 160).ends_with("..."));
    }
}
