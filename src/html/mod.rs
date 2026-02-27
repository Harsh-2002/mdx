pub mod assets;

use comrak::options::Plugins;
use comrak::plugins::syntect::SyntectAdapter;
use comrak::{Options, markdown_to_html_with_plugins};

use crate::cli::ThemeName;
use assets::{
    CSS, FAVICON, FILE_ICON_SVG, JS_EDITOR, JS_EDITOR_DRAG_DROP, JS_EDITOR_MULTI, JS_EDITOR_SEARCH,
    JS_INDEX, JS_INIT, JS_KATEX, JS_LIVE, JS_LIVE_MULTI, JS_THEME_EARLY, KATEX_CSS, KATEX_JS,
    PENCIL_SVG, PLUS_SVG, PRINT_SVG,
};

fn comrak_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.alerts = true;
    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.extension.math_dollars = true;
    options.extension.math_code = true;
    options.render.r#unsafe = true;
    options
}

/// Render markdown to an HTML fragment (just the article body).
pub fn render_fragment(markdown: &str, syntax_theme: &str) -> String {
    let options = comrak_options();
    let adapter = SyntectAdapter::new(Some(syntax_theme));
    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);
    markdown_to_html_with_plugins(markdown, &options, &plugins)
}

/// Render markdown to a full HTML page with live reload (SSE) for serve mode.
pub fn render_page(
    markdown: &str,
    syntax_theme: &str,
    theme: &ThemeName,
    title: &str,
    custom_css: &str,
) -> String {
    let body = render_fragment(markdown, syntax_theme);
    let theme_attr = theme_strings(theme);

    let custom_css_block = if custom_css.is_empty() {
        String::new()
    } else {
        format!("<style>{}</style>", custom_css)
    };

    format!(
        r#"<!DOCTYPE html>
<html data-theme="{theme_attr}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title} — Preview</title>
    {FAVICON}
    <style>{CSS}</style>
    {custom_css_block}
    {KATEX_CSS}
    {KATEX_JS}
    <script>{JS_THEME_EARLY}</script>
</head>
<body class="has-editor">
    <div id="progress-bar"></div>
    <button id="sidebar-toggle">&#9776;</button>
    <nav id="sidebar">
        <div id="toc-panel" class="sidebar-panel active"></div>
    </nav>
    <button id="theme-toggle"></button>
    <button id="print-btn" title="Print / Save PDF" onclick="window.print()">{PRINT_SVG}</button>
    <button id="editor-toggle" title="Toggle editor">{PENCIL_SVG}</button>
    <div id="editor-pane">
        <div id="editor-toolbar">
            <span id="editor-filename">{title}</span>
            <span id="save-status"></span>
            <span id="editor-stats"></span>
        </div>
        <div id="editor-format-bar">
            <button class="fmt-btn" data-fmt="bold" title="Bold"><b>B</b></button>
            <button class="fmt-btn" data-fmt="italic" title="Italic"><i>I</i></button>
            <button class="fmt-btn" data-fmt="heading" title="Heading">H</button>
            <button class="fmt-btn" data-fmt="strikethrough" title="Strikethrough"><s>S</s></button>
            <button class="fmt-btn" data-fmt="code" title="Code">&lt;/&gt;</button>
            <button class="fmt-btn" data-fmt="link" title="Link">[]</button>
            <button class="fmt-btn" data-fmt="list" title="List">&#x2022;</button>
            <button class="fmt-btn" data-fmt="quote" title="Blockquote">&gt;</button>
        </div>
        <div id="editor-body">
            <div id="line-numbers" aria-hidden="true"></div>
            <textarea id="editor-textarea" spellcheck="false"></textarea>
        </div>
    </div>
    <article id="content" class="markdown-body">{body}</article>
    <button id="back-to-top"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="19" x2="12" y2="5"/><polyline points="5 12 12 5 19 12"/></svg></button>
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
    <script>{JS_LIVE}</script>
    <script>{JS_EDITOR}</script>
    <script>{JS_EDITOR_SEARCH}</script>
    <script>{JS_EDITOR_DRAG_DROP}</script>
</body>
</html>"#
    )
}

/// Render markdown to a full HTML page for multi-file serve mode.
pub fn render_page_multi(
    markdown: &str,
    syntax_theme: &str,
    theme: &ThemeName,
    title: &str,
    all_files: &[String],
    current_file: &str,
    custom_css: &str,
) -> String {
    let body = render_fragment(markdown, syntax_theme);
    let theme_attr = theme_strings(theme);

    let custom_css_block = if custom_css.is_empty() {
        String::new()
    } else {
        format!("<style>{}</style>", custom_css)
    };

    let mut file_links = String::new();
    for file in all_files {
        let active = if file == current_file {
            r#" class="active""#
        } else {
            ""
        };
        file_links.push_str(&format!(r#"<a href="/{file}"{active}>{file}</a>"#));
    }

    format!(
        r#"<!DOCTYPE html>
<html data-theme="{theme_attr}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title} — Preview</title>
    {FAVICON}
    <style>{CSS}</style>
    {custom_css_block}
    {KATEX_CSS}
    {KATEX_JS}
    <script>{JS_THEME_EARLY}</script>
</head>
<body class="has-editor">
    <div id="progress-bar"></div>
    <button id="sidebar-toggle">&#9776;</button>
    <nav id="sidebar">
        <a id="back-to-index" href="/">&#8592; Index</a>
        <div id="sidebar-tabs">
            <button class="sidebar-tab active" data-panel="file-nav">Files</button>
            <button class="sidebar-tab" data-panel="toc-panel">Contents</button>
        </div>
        <div id="file-nav" class="sidebar-panel active">
            <input id="file-search" type="text" placeholder="Filter files...">
            {file_links}
        </div>
        <div id="toc-panel" class="sidebar-panel"></div>
    </nav>
    <button id="theme-toggle"></button>
    <button id="print-btn" title="Print / Save PDF" onclick="window.print()">{PRINT_SVG}</button>
    <button id="editor-toggle" title="Toggle editor">{PENCIL_SVG}</button>
    <div id="editor-pane">
        <div id="editor-toolbar">
            <span id="editor-filename">{title}</span>
            <span id="save-status"></span>
            <span id="editor-stats"></span>
        </div>
        <div id="editor-format-bar">
            <button class="fmt-btn" data-fmt="bold" title="Bold"><b>B</b></button>
            <button class="fmt-btn" data-fmt="italic" title="Italic"><i>I</i></button>
            <button class="fmt-btn" data-fmt="heading" title="Heading">H</button>
            <button class="fmt-btn" data-fmt="strikethrough" title="Strikethrough"><s>S</s></button>
            <button class="fmt-btn" data-fmt="code" title="Code">&lt;/&gt;</button>
            <button class="fmt-btn" data-fmt="link" title="Link">[]</button>
            <button class="fmt-btn" data-fmt="list" title="List">&#x2022;</button>
            <button class="fmt-btn" data-fmt="quote" title="Blockquote">&gt;</button>
        </div>
        <div id="editor-body">
            <div id="line-numbers" aria-hidden="true"></div>
            <textarea id="editor-textarea" spellcheck="false"></textarea>
        </div>
    </div>
    <article id="content" class="markdown-body">{body}</article>
    <button id="back-to-top"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="19" x2="12" y2="5"/><polyline points="5 12 12 5 19 12"/></svg></button>
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
    <script>{JS_LIVE_MULTI}</script>
    <script>{JS_EDITOR_MULTI}</script>
    <script>{JS_EDITOR_SEARCH}</script>
    <script>{JS_EDITOR_DRAG_DROP}</script>
</body>
</html>"#
    )
}

/// Render markdown to a standalone HTML page (no SSE, for export).
pub fn render_standalone(
    markdown: &str,
    syntax_theme: &str,
    theme: &ThemeName,
    title: &str,
    custom_css: &str,
) -> String {
    let body = render_fragment(markdown, syntax_theme);
    let theme_attr = theme_strings(theme);

    let custom_css_block = if custom_css.is_empty() {
        String::new()
    } else {
        format!("<style>{}</style>", custom_css)
    };

    format!(
        r#"<!DOCTYPE html>
<html data-theme="{theme_attr}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    {FAVICON}
    <style>{CSS}</style>
    {custom_css_block}
    {KATEX_CSS}
    {KATEX_JS}
    <script>{JS_THEME_EARLY}</script>
</head>
<body>
    <div id="progress-bar"></div>
    <button id="sidebar-toggle">&#9776;</button>
    <nav id="sidebar">
        <div id="toc-panel" class="sidebar-panel active"></div>
    </nav>
    <button id="theme-toggle"></button>
    <article id="content" class="markdown-body">{body}</article>
    <button id="back-to-top"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="19" x2="12" y2="5"/><polyline points="5 12 12 5 19 12"/></svg></button>
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
</html>"#
    )
}

/// Render an index page listing markdown files as cards.
pub fn render_index_page(files: &[String], theme: &ThemeName, dir_mode: bool) -> String {
    let theme_attr = theme_strings(theme);

    let new_note_html = if dir_mode {
        format!(
            r#"<div class="file-card new-note-card" id="new-note-card" tabindex="0"><div class="file-icon">{PLUS_SVG}</div><div class="file-name">New Note</div></div><div class="file-card new-note-form" id="new-note-form" style="display:none"><input id="new-note-input" type="text" placeholder="note-name"><div class="new-note-hint">.md &middot; Enter to create &middot; Esc to cancel</div></div>"#
        )
    } else {
        String::new()
    };

    let mut cards = String::new();
    for file in files {
        cards.push_str(&format!(
            r#"<a class="file-card" href="/{file}"><div class="file-icon">{FILE_ICON_SVG}</div><div class="file-name">{file}</div></a>"#
        ));
    }

    let index_js = if dir_mode {
        format!("<script>{JS_INDEX}</script>")
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html data-theme="{theme_attr}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Preview — file index</title>
    {FAVICON}
    <style>{CSS}</style>
    <script>{JS_THEME_EARLY}</script>
</head>
<body>
    <button id="theme-toggle"></button>
    <article class="markdown-body">
        <h1>Markdown Files</h1>
        <div class="file-grid">{new_note_html}{cards}</div>
    </article>
    <script>{JS_INIT}</script>
    {index_js}
</body>
</html>"#
    )
}

fn theme_strings(theme: &ThemeName) -> &'static str {
    match theme {
        ThemeName::Dark => "dark",
        ThemeName::Light => "light",
    }
}
