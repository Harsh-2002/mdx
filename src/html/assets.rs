pub const CSS: &str = r#"
:root {
    --bg: #1e1e1e;
    --fg: #d4d4d4;
    --fg-muted: #9e9e9e;
    --border: #3a3a3a;
    --bg-subtle: #252525;
    --bg-code: #2a2a2a;
    --accent: #e0a545;
    --accent-bg: rgba(224, 165, 69, 0.1);
    --accent-green: #4ec963;
    --accent-red: #f47067;
    --accent-yellow: #e0a545;
    --accent-purple: #c49bff;
}

[data-theme="light"] {
    --bg: #faf8f5;
    --fg: #37352f;
    --fg-muted: #787774;
    --border: #e3e2de;
    --bg-subtle: #f1efe9;
    --bg-code: #edebe5;
    --accent: #2383e2;
    --accent-bg: rgba(35, 131, 226, 0.08);
    --accent-green: #2a8546;
    --accent-red: #d44c47;
    --accent-yellow: #a07816;
    --accent-purple: #7c5fc0;
}

* { box-sizing: border-box; }
html { scroll-behavior: smooth; }

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans", Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 1.6;
    color: var(--fg);
    background: var(--bg);
    margin: 0;
    padding: 2rem 1rem;
    transition: padding-left 0.3s;
}

.markdown-body {
    max-width: 900px;
    margin: 0 auto;
}

h1, h2, h3, h4, h5, h6 {
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 600;
    line-height: 1.25;
}

h1 { font-size: 2em; padding-bottom: 0.3em; border-bottom: 1px solid var(--border); }
h2 { font-size: 1.5em; padding-bottom: 0.3em; border-bottom: 1px solid var(--border); }
h3 { font-size: 1.25em; }
h4 { font-size: 1em; }
h5 { font-size: 0.875em; }
h6 { font-size: 0.85em; color: var(--fg-muted); }

p { margin: 0 0 1em; }

a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }

strong { font-weight: 600; }

code {
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace;
    font-size: 85%;
    padding: 0.2em 0.4em;
    background: var(--bg-code);
    border-radius: 6px;
}

pre {
    position: relative;
    background: var(--bg-code);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 1em;
    overflow-x: auto;
    line-height: 1.45;
    margin: 0 0 1em;
}

pre code {
    padding: 0;
    background: none;
    border-radius: 0;
    font-size: 85%;
}

.copy-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    opacity: 0;
    transition: opacity 0.2s;
    background: var(--bg-subtle);
    color: var(--fg-muted);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 12px;
    cursor: pointer;
}

.copy-btn:hover {
    color: var(--fg);
    background: var(--border);
}

pre:hover .copy-btn { opacity: 1; }

.code-lang {
    position: absolute;
    top: 0;
    left: 0;
    font-size: 0.7rem;
    color: var(--fg-muted);
    background: var(--border);
    padding: 1px 8px;
    border-radius: 0 0 6px 0;
    pointer-events: none;
}

blockquote {
    margin: 0 0 1em;
    padding: 0 1em;
    color: var(--fg-muted);
    border-left: 3px solid var(--border);
}

ul, ol { margin: 0 0 1em; padding-left: 2em; }
li + li { margin-top: 0.25em; }
li > ul, li > ol { margin: 0.25em 0 0; }

input[type="checkbox"] {
    margin-right: 0.5em;
    vertical-align: middle;
}

table {
    border-collapse: collapse;
    width: 100%;
    margin: 0 0 1em;
    overflow: auto;
    display: block;
}

th, td {
    padding: 6px 13px;
    border: 1px solid var(--border);
}

th {
    font-weight: 600;
    background: var(--bg-subtle);
}

tr:nth-child(even) { background: var(--bg-subtle); }

hr {
    height: 2px;
    background: var(--border);
    border: none;
    margin: 1.5em 0;
}

img { max-width: 100%; height: auto; }

del { color: var(--fg-muted); }

.markdown-alert {
    padding: 0.5em 1em;
    margin: 0 0 1em;
    border-left: 4px solid;
    border-radius: 0 6px 6px 0;
    background: var(--bg-subtle);
}

.markdown-alert-note { border-left-color: var(--accent); }
.markdown-alert-tip { border-left-color: var(--accent-green); }
.markdown-alert-important { border-left-color: var(--accent-purple); }
.markdown-alert-warning { border-left-color: var(--accent-yellow); }
.markdown-alert-caution { border-left-color: var(--accent-red); }

.markdown-alert-title {
    font-weight: 600;
    margin-bottom: 0.25em;
}

.markdown-alert-note .markdown-alert-title { color: var(--accent); }
.markdown-alert-tip .markdown-alert-title { color: var(--accent-green); }
.markdown-alert-important .markdown-alert-title { color: var(--accent-purple); }
.markdown-alert-warning .markdown-alert-title { color: var(--accent-yellow); }
.markdown-alert-caution .markdown-alert-title { color: var(--accent-red); }

sup a { text-decoration: none; }
.footnote-definition { font-size: 0.9em; color: var(--fg-muted); margin: 0.5em 0; }

.mermaid {
    text-align: center;
    margin: 1.5em 0;
    padding: 1em;
    border-radius: 8px;
    background: var(--bg-subtle);
    border: 1px solid var(--border);
}

/* Theme toggle button */
#theme-toggle {
    position: fixed;
    top: 12px;
    right: 12px;
    z-index: 110;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: 1px solid var(--border);
    background: var(--bg-subtle);
    color: var(--fg);
    font-size: 18px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s;
    line-height: 1;
    padding: 0;
}

#theme-toggle:hover { background: var(--border); }

/* Sidebar */
#sidebar {
    position: fixed;
    left: 0;
    top: 0;
    width: 252px;
    height: 100vh;
    background: var(--bg-subtle);
    border-right: 1px solid var(--border);
    transform: translateX(-100%);
    transition: transform 0.3s;
    z-index: 99;
    padding: 3.5rem 0 0;
    display: flex;
    flex-direction: column;
}

#sidebar.open { transform: translateX(0); }

html.sidebar-restore #sidebar { transform: translateX(0); transition: none; }
html.sidebar-restore body { padding-left: calc(252px + 1.5rem); transition: none; }

#back-to-index {
    display: block;
    padding: 0.6rem 1rem;
    color: var(--fg-muted);
    text-decoration: none;
    font-size: 0.85rem;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}

#back-to-index:hover { color: var(--accent); text-decoration: none; }

#sidebar-tabs {
    display: flex;
    border-bottom: 1px solid var(--border);
    padding: 0 0.5rem;
    flex-shrink: 0;
}

.sidebar-tab {
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--fg-muted);
    padding: 0.5rem 1rem;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 500;
    font-family: inherit;
}

.sidebar-tab:hover { color: var(--fg); }

.sidebar-tab.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
}

.sidebar-panel {
    display: none;
    overflow-y: auto;
    flex: 1;
    padding: 0.75rem 1rem;
    scrollbar-width: none;
    -ms-overflow-style: none;
}
.sidebar-panel::-webkit-scrollbar { display: none; }

.sidebar-panel.active { display: block; }

#file-nav a {
    display: block;
    padding: 0.4rem 0.75rem;
    color: var(--fg-muted);
    text-decoration: none;
    font-size: 0.9em;
    border-radius: 4px;
    margin-bottom: 2px;
    word-wrap: break-word;
    overflow-wrap: break-word;
}

#file-nav a:hover {
    color: var(--fg);
    background: var(--border);
    text-decoration: none;
}

#file-nav a.active {
    color: var(--accent);
    background: var(--accent-bg);
}

#file-search {
    display: block;
    width: 100%;
    padding: 0.4rem 0.75rem;
    margin-bottom: 0.5rem;
    font-size: 0.85rem;
    font-family: inherit;
    color: var(--fg);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    outline: none;
    box-sizing: border-box;
}

#file-search:focus { border-color: var(--accent); }
#file-search::placeholder { color: var(--fg-muted); }

#toc-panel ul {
    list-style: none;
    padding: 0;
    margin: 0;
}

#toc-panel li { margin: 0.3em 0; }

#toc-panel a {
    color: var(--fg-muted);
    text-decoration: none;
    font-size: 0.9em;
    display: block;
    padding: 2px 4px;
    border-radius: 4px;
    word-wrap: break-word;
    overflow-wrap: break-word;
    transition: color 0.2s, background 0.2s;
}

#toc-panel a:hover {
    color: var(--fg);
    background: var(--border);
}

#toc-panel a.toc-active {
    color: var(--accent);
    background: var(--accent-bg);
}

#sidebar-toggle {
    position: fixed;
    top: 12px;
    left: 12px;
    z-index: 101;
    width: 36px;
    height: 36px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg-subtle);
    color: var(--fg);
    font-size: 18px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
}

#sidebar-toggle:hover { background: var(--border); }

body.sidebar-open { padding-left: calc(252px + 1.5rem); }

/* Reading progress bar */
#progress-bar {
    position: fixed;
    top: 0;
    left: 0;
    width: 0%;
    height: 2px;
    background: var(--accent);
    z-index: 200;
    pointer-events: none;
    transition: width 0.1s;
}

/* Back to top button */
#back-to-top {
    display: none;
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 100;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: 1px solid var(--border);
    background: var(--bg-subtle);
    color: var(--fg);
    cursor: pointer;
    align-items: center;
    justify-content: center;
    transition: background 0.2s;
    padding: 0;
}

#back-to-top:hover { background: var(--border); }
#back-to-top.visible { display: flex; }

/* File gallery */
.file-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1rem;
    margin-top: 1.5rem;
}

.file-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 1.5rem 1rem;
    border: 1px solid var(--border);
    border-top: 3px solid transparent;
    border-radius: 8px;
    text-decoration: none;
    color: var(--fg);
    transition: border-color 0.2s, transform 0.2s, box-shadow 0.2s;
}

.file-card:hover {
    border-color: var(--accent);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    text-decoration: none;
}

.file-icon {
    color: var(--fg-muted);
    margin-bottom: 0.75rem;
}

.file-card:hover .file-icon { color: var(--accent); }

.file-name {
    font-size: 0.9rem;
    text-align: center;
    word-break: break-all;
}

/* Editor */
#editor-toggle {
    position: fixed;
    top: 12px;
    right: 60px;
    z-index: 110;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: 1px solid var(--border);
    background: var(--bg-subtle);
    color: var(--fg);
    font-size: 18px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s;
    line-height: 1;
    padding: 0;
}

#editor-toggle:hover { background: var(--border); }

#editor-pane {
    display: none;
    flex-direction: column;
    max-width: 1400px;
    max-height: calc(100vh - 4rem);
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 2px 16px rgba(0,0,0,0.12);
    width: calc(100% - 4rem);
    height: calc(100vh - 6rem);
    z-index: 102;
}

body.editor-open #editor-pane {
    display: flex;
    animation: editorFadeIn 0.15s ease;
}
body.editor-open .markdown-body { display: none; }
body.editor-open #progress-bar,
body.editor-open #back-to-top { display: none !important; }
body.editor-open #sidebar,
body.editor-open #sidebar-toggle { display: none !important; }
body.editor-open #editor-toggle {
    background: var(--accent);
    color: var(--bg);
    border-color: var(--accent);
}

@keyframes editorFadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
}

#editor-toolbar {
    display: flex;
    align-items: center;
    padding: 10px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-subtle);
    flex-shrink: 0;
    font-size: 0.85rem;
    gap: 10px;
}

#editor-filename {
    color: var(--fg-muted);
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    font-size: 0.8rem;
}

#editor-stats {
    color: var(--fg-muted);
    font-size: 0.75rem;
    margin-left: auto;
}

#save-status {
    font-size: 0.8rem;
    font-weight: 500;
}

#editor-format-bar {
    display: flex;
    gap: 2px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-subtle);
    flex-shrink: 0;
    flex-wrap: wrap;
}

.fmt-btn {
    background: none;
    border: 1px solid transparent;
    border-radius: 6px;
    color: var(--fg-muted);
    cursor: pointer;
    padding: 4px 10px;
    font-size: 13px;
    font-family: inherit;
    line-height: 1.4;
    transition: all 0.15s;
}

.fmt-btn:hover {
    background: var(--border);
    color: var(--fg);
}

.fmt-btn:active {
    transform: scale(0.95);
}

#editor-body {
    display: flex;
    flex: 1;
    overflow: hidden;
}

#line-numbers {
    padding: 1em 0.5em 1em 0.75em;
    text-align: right;
    color: var(--fg-muted);
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace;
    font-size: 14px;
    line-height: 1.6;
    overflow: hidden;
    flex-shrink: 0;
    user-select: none;
    min-width: 3em;
    opacity: 0.5;
    border-right: 1px solid var(--border);
    white-space: pre;
    background: var(--bg-subtle);
}

#editor-textarea {
    flex: 1;
    width: 100%;
    border: none;
    outline: none;
    resize: none;
    padding: 1em 1.2em;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace;
    font-size: 14px;
    line-height: 1.6;
    tab-size: 4;
    color: var(--fg);
    background: var(--bg);
    caret-color: var(--accent);
}

#editor-textarea::selection {
    background: var(--accent-bg);
}

@media print {
    body {
        background: white !important;
        color: black !important;
        padding: 1cm !important;
    }
    .markdown-body { max-width: none; }
    pre {
        border: 1px solid #ccc;
        break-inside: avoid;
        page-break-inside: avoid;
    }
    table { break-inside: avoid; page-break-inside: avoid; }
    blockquote { break-inside: avoid; page-break-inside: avoid; }
    h1, h2, h3, h4, h5, h6 { break-after: avoid; page-break-after: avoid; }
    img { break-inside: avoid; page-break-inside: avoid; max-width: 100%; }
    a { color: black !important; text-decoration: underline; }
    a[href^="http"]::after { content: " (" attr(href) ")"; font-size: 0.8em; color: #666; }
    code { background: #f5f5f5 !important; color: black !important; }
    pre { background: #f5f5f5 !important; }
    pre code { color: black !important; }
    .copy-btn, .code-lang, #theme-toggle, #sidebar, #sidebar-toggle, #progress-bar, #back-to-top, #editor-pane, #editor-toggle, #print-btn { display: none !important; }
    .mermaid { border: 1px solid #ccc; }
    [data-theme] { --fg: black; --bg: white; --border: #ccc; --bg-subtle: #f5f5f5; --bg-code: #f5f5f5; }
}

/* Print / PDF button */
#print-btn {
    position: fixed;
    top: 12px;
    right: 108px;
    z-index: 110;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: 1px solid var(--border);
    background: var(--bg-subtle);
    color: var(--fg);
    font-size: 18px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s;
    line-height: 1;
    padding: 0;
}
#print-btn:hover { background: var(--border); }

/* Editor search bar */
.editor-search-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-subtle);
    flex-shrink: 0;
    flex-wrap: wrap;
}
.editor-search-bar input {
    padding: 4px 8px;
    font-size: 13px;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    color: var(--fg);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    outline: none;
    min-width: 140px;
}
.editor-search-bar input:focus { border-color: var(--accent); }
.editor-search-bar .search-count {
    font-size: 12px;
    color: var(--fg-muted);
    min-width: 50px;
}
.editor-search-bar button {
    background: none;
    border: 1px solid transparent;
    border-radius: 4px;
    color: var(--fg-muted);
    cursor: pointer;
    padding: 3px 8px;
    font-size: 12px;
    font-family: inherit;
}
.editor-search-bar button:hover {
    background: var(--border);
    color: var(--fg);
}
.editor-search-bar .search-toggle {
    padding: 3px 6px;
    font-weight: 600;
    border: 1px solid var(--border);
    border-radius: 4px;
}
.editor-search-bar .search-toggle.active {
    background: var(--accent-bg);
    color: var(--accent);
    border-color: var(--accent);
}

/* Drag & drop overlay */
.editor-drop-overlay {
    position: absolute;
    inset: 0;
    background: var(--accent-bg);
    border: 2px dashed var(--accent);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.1rem;
    color: var(--accent);
    z-index: 200;
    pointer-events: none;
}
#editor-body { position: relative; }

@media (max-width: 767px) {
    body { padding: 1rem 0.5rem; }
    body.sidebar-open { padding-left: 0.5rem; }
    html.sidebar-restore body { padding-left: 0.5rem; }
    #sidebar.open { box-shadow: 2px 0 12px rgba(0,0,0,0.3); }
    body.editor-open #editor-pane {
        top: 0;
        left: 0;
        transform: none;
        width: 100%;
        max-width: none;
        height: 100vh;
        max-height: none;
        border-radius: 0;
        border: none;
        box-shadow: none;
    }
    #editor-format-bar {
        flex-wrap: nowrap;
        overflow-x: auto;
        -webkit-overflow-scrolling: touch;
        scrollbar-width: none;
    }
    #editor-format-bar::-webkit-scrollbar { display: none; }
    #editor-toolbar { font-size: 0.8rem; }
}

/* New note card (directory mode) */
.new-note-card {
    border-style: dashed;
    cursor: pointer;
    color: var(--fg-muted);
}
.new-note-card:hover { border-color: var(--accent); }
.new-note-card:hover .file-icon { color: var(--accent); }

.new-note-form { justify-content: center; padding: 1rem; }

#new-note-input {
    width: 100%;
    padding: 0.5rem;
    font-size: 0.9rem;
    font-family: inherit;
    color: var(--fg);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    outline: none;
    text-align: center;
}
#new-note-input:focus { border-color: var(--accent); }

.new-note-hint {
    font-size: 0.7rem;
    color: var(--fg-muted);
    margin-top: 0.5rem;
    text-align: center;
}
"#;

/// Tiny script injected in <head> to prevent theme flash on load.
pub const JS_THEME_EARLY: &str = r#"(function(){
    var s = localStorage.getItem('md-theme');
    if (!s) s = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    document.documentElement.setAttribute('data-theme', s);
    if (localStorage.getItem('md-sidebar-open') === 'true') document.documentElement.classList.add('sidebar-restore');
})();
function getMermaidThemeVars() {
    var isDark = (document.documentElement.getAttribute('data-theme') || 'dark') === 'dark';
    if (isDark) {
        return {
            darkMode: true,
            background: '#1e1e1e',
            primaryColor: '#2a2a2a',
            primaryTextColor: '#d4d4d4',
            primaryBorderColor: '#3a3a3a',
            secondaryColor: '#252525',
            secondaryTextColor: '#d4d4d4',
            secondaryBorderColor: '#3a3a3a',
            tertiaryColor: '#252525',
            tertiaryTextColor: '#9e9e9e',
            tertiaryBorderColor: '#3a3a3a',
            lineColor: '#e0a545',
            textColor: '#d4d4d4',
            mainBkg: '#2a2a2a',
            nodeBorder: '#3a3a3a',
            clusterBkg: '#252525',
            clusterBorder: '#3a3a3a',
            titleColor: '#e0a545',
            edgeLabelBackground: '#252525',
            nodeTextColor: '#d4d4d4',
            noteBkgColor: '#2a2a2a',
            noteTextColor: '#d4d4d4',
            noteBorderColor: '#3a3a3a'
        };
    } else {
        return {
            darkMode: false,
            background: '#faf8f5',
            primaryColor: '#f1efe9',
            primaryTextColor: '#37352f',
            primaryBorderColor: '#e3e2de',
            secondaryColor: '#edebe5',
            secondaryTextColor: '#37352f',
            secondaryBorderColor: '#e3e2de',
            tertiaryColor: '#edebe5',
            tertiaryTextColor: '#787774',
            tertiaryBorderColor: '#e3e2de',
            lineColor: '#2383e2',
            textColor: '#37352f',
            mainBkg: '#f1efe9',
            nodeBorder: '#e3e2de',
            clusterBkg: '#edebe5',
            clusterBorder: '#e3e2de',
            titleColor: '#2383e2',
            edgeLabelBackground: '#f1efe9',
            nodeTextColor: '#37352f',
            noteBkgColor: '#f1efe9',
            noteTextColor: '#37352f',
            noteBorderColor: '#e3e2de'
        };
    }
}"#;

/// SVG document icon for the file gallery cards.
pub const FILE_ICON_SVG: &str = r#"<svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>"#;

/// JS for copy buttons, sidebar, and theme toggle — runs on all pages (serve + standalone).
pub const JS_INIT: &str = r##"
function initCopyButtons() {
    document.querySelectorAll('pre > code').forEach(function(block) {
        if (block.parentElement.querySelector('.copy-btn')) return;
        var btn = document.createElement('button');
        btn.className = 'copy-btn';
        btn.textContent = 'Copy';
        btn.onclick = function() {
            navigator.clipboard.writeText(block.textContent);
            btn.textContent = 'Copied!';
            setTimeout(function() { btn.textContent = 'Copy'; }, 2000);
        };
        block.parentElement.appendChild(btn);
        var cls = block.className.match(/language-(\S+)/);
        if (cls && cls[1] !== 'mermaid' && !block.parentElement.querySelector('.code-lang')) {
            var lang = document.createElement('span');
            lang.className = 'code-lang';
            lang.textContent = cls[1];
            block.parentElement.appendChild(lang);
        }
    });
}

function initThemeToggle() {
    var btn = document.getElementById('theme-toggle');
    if (!btn) return;
    var sunSvg = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>';
    var moonSvg = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>';
    function update() {
        var theme = document.documentElement.getAttribute('data-theme') || 'dark';
        btn.innerHTML = theme === 'dark' ? sunSvg : moonSvg;
    }
    btn.onclick = function() {
        var current = document.documentElement.getAttribute('data-theme') || 'dark';
        var next = current === 'dark' ? 'light' : 'dark';
        document.documentElement.setAttribute('data-theme', next);
        localStorage.setItem('md-theme', next);
        update();
        if (window.mermaid) {
            window.mermaid.initialize({ startOnLoad: false, theme: 'base', themeVariables: getMermaidThemeVars() });
            document.querySelectorAll('.mermaid[data-processed]').forEach(function(el) { el.removeAttribute('data-processed'); });
            document.querySelectorAll('.mermaid').forEach(function(el) {
                if (el.getAttribute('data-original')) { el.innerHTML = el.getAttribute('data-original'); }
            });
            window.mermaid.run();
        }
    };
    var saved = localStorage.getItem('md-theme');
    if (saved) {
        document.documentElement.setAttribute('data-theme', saved);
    } else {
        var systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        document.documentElement.setAttribute('data-theme', systemDark ? 'dark' : 'light');
        window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function(e) {
            if (!localStorage.getItem('md-theme')) {
                var t = e.matches ? 'dark' : 'light';
                document.documentElement.setAttribute('data-theme', t);
                update();
            }
        });
    }
    update();
}

function initSidebar() {
    var sidebar = document.getElementById('sidebar');
    var toggle = document.getElementById('sidebar-toggle');
    if (!sidebar || !toggle) return;
    var tocPanel = document.getElementById('toc-panel');
    var hasToc = false;
    if (tocPanel) {
        var headings = document.querySelectorAll('#content h1, #content h2, #content h3');
        if (headings.length > 0) {
            var html = '<ul>';
            headings.forEach(function(h, i) {
                if (!h.id) h.id = 'heading-' + i;
                var level = parseInt(h.tagName[1]) - 1;
                html += '<li style="margin-left:' + level + 'em"><a href="#' + h.id + '">' + h.textContent + '</a></li>';
            });
            html += '</ul>';
            tocPanel.innerHTML = html;
            hasToc = true;
            tocPanel.querySelectorAll('a').forEach(function(a) {
                a.onclick = function(e) {
                    e.preventDefault();
                    var target = document.getElementById(a.getAttribute('href').substring(1));
                    if (target) target.scrollIntoView({ behavior: 'smooth' });
                };
            });
        } else {
            tocPanel.innerHTML = '';
        }
    }
    var fileNav = document.getElementById('file-nav');
    var hasFiles = fileNav && fileNav.querySelectorAll('a').length > 0;
    var tabsContainer = document.getElementById('sidebar-tabs');
    if (!hasToc && !hasFiles) {
        toggle.style.display = 'none';
        return;
    }
    toggle.style.display = '';
    if (tabsContainer) {
        if (hasToc && hasFiles) {
            tabsContainer.style.display = '';
        } else {
            tabsContainer.style.display = 'none';
            if (hasFiles) {
                fileNav.classList.add('active');
                if (tocPanel) tocPanel.classList.remove('active');
            } else if (hasToc) {
                tocPanel.classList.add('active');
                if (fileNav) fileNav.classList.remove('active');
            }
        }
        var tabs = tabsContainer.querySelectorAll('.sidebar-tab');
        tabs.forEach(function(tab) {
            tab.onclick = function() {
                tabs.forEach(function(t) { t.classList.remove('active'); });
                tab.classList.add('active');
                sidebar.querySelectorAll('.sidebar-panel').forEach(function(p) { p.classList.remove('active'); });
                var panel = document.getElementById(tab.getAttribute('data-panel'));
                if (panel) panel.classList.add('active');
                localStorage.setItem('md-sidebar-tab', tab.getAttribute('data-panel'));
            };
        });
    }
    var restored = document.documentElement.classList.contains('sidebar-restore');
    if (restored) {
        sidebar.classList.add('open');
        document.body.classList.add('sidebar-open');
        document.documentElement.classList.remove('sidebar-restore');
        var savedTab = localStorage.getItem('md-sidebar-tab');
        if (savedTab && tabsContainer) {
            var allTabs = tabsContainer.querySelectorAll('.sidebar-tab');
            allTabs.forEach(function(t) { t.classList.remove('active'); });
            sidebar.querySelectorAll('.sidebar-panel').forEach(function(p) { p.classList.remove('active'); });
            var targetTab = tabsContainer.querySelector('[data-panel="' + savedTab + '"]');
            var targetPanel = document.getElementById(savedTab);
            if (targetTab && targetPanel) {
                targetTab.classList.add('active');
                targetPanel.classList.add('active');
            }
        }
    }
    toggle.onclick = function() {
        sidebar.classList.toggle('open');
        document.body.classList.toggle('sidebar-open');
        localStorage.setItem('md-sidebar-open', sidebar.classList.contains('open'));
    };
    if (!window._outsideInited) {
        window._outsideInited = true;
        document.addEventListener('click', function(e) {
            var sb = document.getElementById('sidebar');
            var tgl = document.getElementById('sidebar-toggle');
            if (window.innerWidth <= 767 && sb && sb.classList.contains('open')
                && !sb.contains(e.target) && e.target !== tgl && !tgl.contains(e.target)) {
                sb.classList.remove('open');
                document.body.classList.remove('sidebar-open');
                localStorage.setItem('md-sidebar-open', false);
            }
        });
    }
    if (fileNav) {
        var activeLink = fileNav.querySelector('a.active');
        if (activeLink) activeLink.scrollIntoView({ block: 'nearest' });
    }
}

function initScrollSpy() {
    if (window._tocObserver) window._tocObserver.disconnect();
    var tocPanel = document.getElementById('toc-panel');
    if (!tocPanel) return;
    var headings = document.querySelectorAll('#content h1, #content h2, #content h3');
    if (!headings.length) return;
    var observer = new IntersectionObserver(function(entries) {
        entries.forEach(function(entry) {
            if (entry.isIntersecting) {
                tocPanel.querySelectorAll('a').forEach(function(a) { a.classList.remove('toc-active'); });
                var link = tocPanel.querySelector('a[href="#' + entry.target.id + '"]');
                if (link) link.classList.add('toc-active');
            }
        });
    }, { rootMargin: '0px 0px -80% 0px' });
    headings.forEach(function(h) { observer.observe(h); });
    window._tocObserver = observer;
}

function initKeyboardShortcuts() {
    if (window._kbInited) return;
    window._kbInited = true;
    document.addEventListener('keydown', function(e) {
        if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA' || e.target.isContentEditable) return;
        if (e.key === '[') {
            var toggle = document.getElementById('sidebar-toggle');
            if (toggle) toggle.click();
        } else if (e.key === ']') {
            var tc = document.getElementById('sidebar-tabs');
            if (tc && tc.style.display !== 'none') {
                var tabs = tc.querySelectorAll('.sidebar-tab');
                var idx = -1;
                tabs.forEach(function(t, i) { if (t.classList.contains('active')) idx = i; });
                var next = (idx + 1) % tabs.length;
                tabs[next].click();
            }
        } else if (e.key === 't') {
            var themeBtn = document.getElementById('theme-toggle');
            if (themeBtn) themeBtn.click();
        } else if (e.key === 'e') {
            var editorToggle = document.getElementById('editor-toggle');
            if (editorToggle) editorToggle.click();
        } else if (e.key === 'Escape') {
            var sb = document.getElementById('sidebar');
            if (sb && sb.classList.contains('open')) {
                document.getElementById('sidebar-toggle').click();
            }
        }
    });
}

function initProgressBar() {
    var bar = document.getElementById('progress-bar');
    if (!bar) return;
    window.addEventListener('scroll', function() {
        var scrollTop = window.scrollY;
        var docHeight = document.documentElement.scrollHeight - window.innerHeight;
        bar.style.width = docHeight > 0 ? (scrollTop / docHeight * 100) + '%' : '0%';
    });
}

function initFileSearch() {
    var input = document.getElementById('file-search');
    if (!input) return;
    input.addEventListener('keyup', function() {
        var q = input.value.toLowerCase();
        var links = document.querySelectorAll('#file-nav a');
        links.forEach(function(a) {
            a.style.display = a.textContent.toLowerCase().indexOf(q) !== -1 ? '' : 'none';
        });
    });
}

function initBackToTop() {
    var btn = document.getElementById('back-to-top');
    if (!btn) return;
    window.addEventListener('scroll', function() {
        btn.classList.toggle('visible', window.scrollY > 300);
    });
    btn.onclick = function() {
        window.scrollTo({ top: 0, behavior: 'smooth' });
    };
}

initCopyButtons();
initThemeToggle();
initSidebar();
initScrollSpy();
initKeyboardShortcuts();
initProgressBar();
initFileSearch();
initBackToTop();
"##;

/// JS for SSE live reload — only included in serve mode pages.
pub const JS_LIVE: &str = r#"
var es = new EventSource('/events');
es.onmessage = function(e) {
    if (e.data === 'reload') {
        var scrollY = window.scrollY;
        fetch('/raw').then(function(r) { return r.text(); }).then(function(html) {
            document.getElementById('content').innerHTML = html;
            if (window.mermaid) {
                window.mermaid.initialize({ startOnLoad: false, theme: 'base', themeVariables: getMermaidThemeVars() });
                document.querySelectorAll('code.language-mermaid').forEach(function(el) {
                    var pre = el.parentElement;
                    var div = document.createElement('div');
                    div.className = 'mermaid';
                    var src = el.textContent;
                    div.textContent = src;
                    div.setAttribute('data-original', src);
                    pre.replaceWith(div);
                });
                mermaid.run();
            }
            initCopyButtons();
            if (typeof renderMath === 'function') renderMath();
            initSidebar();
            initScrollSpy();
            window.scrollTo(0, scrollY);
            if (typeof window._editorOnReload === 'function') window._editorOnReload();
        });
    }
};
es.onerror = function() {
    es.close();
    setTimeout(function() { location.reload(); }, 2000);
};
"#;

/// KaTeX CSS link for math rendering.
pub const KATEX_CSS: &str = r#"<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/katex.min.css">"#;

/// KaTeX JS script tags.
pub const KATEX_JS: &str =
    r#"<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/katex.min.js"></script>"#;

/// JS snippet to render math elements emitted by comrak's math_dollars extension.
pub const JS_KATEX: &str = r#"
function renderMath() {
    if (typeof katex === 'undefined') {
        setTimeout(renderMath, 100);
        return;
    }
    document.querySelectorAll('[data-math-style]').forEach(function(el) {
        var displayMode = el.getAttribute('data-math-style') === 'display';
        try {
            katex.render(el.textContent, el, { displayMode: displayMode, throwOnError: false });
        } catch(e) {}
    });
}
renderMath();
"#;

/// SVG plus icon for the "New Note" card in directory mode.
pub const PLUS_SVG: &str = r#"<svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>"#;

/// Inline SVG favicon — white M centered on gold rounded square.
pub const FAVICON: &str = r#"<link rel="icon" type="image/svg+xml" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'><rect width='32' height='32' rx='7' fill='%23e0a545'/><path d='M10 22V10l6 7 6-7v12' fill='none' stroke='white' stroke-width='2.8' stroke-linecap='round' stroke-linejoin='round'/></svg>">"#;

/// SVG print/download icon for the PDF button.
pub const PRINT_SVG: &str = r#"<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 6 2 18 2 18 9"/><path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"/><rect x="6" y="14" width="12" height="8"/></svg>"#;

/// SVG pencil icon for the editor toggle button (default state).
pub const PENCIL_SVG: &str = r#"<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/></svg>"#;

/// SVG eye icon shown when editor is open (click to go back to preview).
pub const EYE_SVG: &str = r#"<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>"#;

/// JS for the markdown editor in single-file serve mode.
pub const JS_EDITOR: &str = r#"
(function() {
    var textarea = document.getElementById('editor-textarea');
    var status = document.getElementById('save-status');
    var toggle = document.getElementById('editor-toggle');
    var lineNums = document.getElementById('line-numbers');
    var statsEl = document.getElementById('editor-stats');
    if (!textarea || !toggle) return;

    var saveTimer = null;
    var saving = false;
    var loaded = false;
    window._editorSkipReload = false;
    var sourceUrl = '/source';
    var pencilSvg = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/></svg>';
    var eyeSvg = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>';

    function updateToggleIcon() {
        var open = document.body.classList.contains('editor-open');
        toggle.innerHTML = open ? eyeSvg : pencilSvg;
        toggle.title = open ? 'Switch to preview' : 'Toggle editor';
    }

    function toggleEditor() {
        document.body.classList.toggle('editor-open');
        var open = document.body.classList.contains('editor-open');
        localStorage.setItem('md-editor-open', open);
        updateToggleIcon();
        if (open && !loaded) loadSource();
        if (!open && saveTimer) {
            clearTimeout(saveTimer);
            saveTimer = null;
            doSave();
        }
    }
    toggle.onclick = toggleEditor;

    if (localStorage.getItem('md-editor-open') === 'true') {
        document.body.classList.add('editor-open');
        updateToggleIcon();
        loadSource();
    }

    function loadSource() {
        fetch(sourceUrl).then(function(r) { return r.text(); }).then(function(text) {
            textarea.value = text;
            loaded = true;
            setStatus('saved');
            updateLineNumbers();
            updateStats();
        });
    }

    textarea.addEventListener('input', function() {
        setStatus('modified');
        clearTimeout(saveTimer);
        saveTimer = setTimeout(doSave, 800);
        updateLineNumbers();
        updateStats();
    });

    textarea.addEventListener('scroll', function() {
        if (lineNums) lineNums.scrollTop = textarea.scrollTop;
    });

    textarea.addEventListener('keydown', function(e) {
        if ((e.ctrlKey || e.metaKey) && e.key === 's') {
            e.preventDefault();
            clearTimeout(saveTimer);
            saveTimer = null;
            doSave();
            return;
        }
        if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
            e.preventDefault();
            wrapSelection('**', '**');
            return;
        }
        if ((e.ctrlKey || e.metaKey) && e.key === 'i') {
            e.preventDefault();
            wrapSelection('*', '*');
            return;
        }
        if (e.key === 'Tab') {
            e.preventDefault();
            if (e.shiftKey) {
                var start = this.selectionStart;
                var ls = this.value.lastIndexOf('\n', start - 1) + 1;
                var ch = this.value.charAt(ls);
                if (ch === '\t') {
                    this.value = this.value.substring(0, ls) + this.value.substring(ls + 1);
                    this.selectionStart = this.selectionEnd = Math.max(ls, start - 1);
                } else {
                    var m = this.value.substring(ls).match(/^ {1,4}/);
                    if (m) {
                        this.value = this.value.substring(0, ls) + this.value.substring(ls + m[0].length);
                        this.selectionStart = this.selectionEnd = Math.max(ls, start - m[0].length);
                    }
                }
            } else {
                var start = this.selectionStart, end = this.selectionEnd;
                this.value = this.value.substring(0, start) + '\t' + this.value.substring(end);
                this.selectionStart = this.selectionEnd = start + 1;
            }
            this.dispatchEvent(new Event('input'));
            return;
        }
        if (e.key === 'Enter' && !e.ctrlKey && !e.metaKey) {
            e.preventDefault();
            var start = this.selectionStart;
            var ls = this.value.lastIndexOf('\n', start - 1) + 1;
            var line = this.value.substring(ls, start);
            var indent = (line.match(/^[\t ]*/) || [''])[0];
            var ins = '\n' + indent;
            this.value = this.value.substring(0, start) + ins + this.value.substring(this.selectionEnd);
            this.selectionStart = this.selectionEnd = start + ins.length;
            this.dispatchEvent(new Event('input'));
            return;
        }
        var pairs = {'(': ')', '[': ']', '`': '`'};
        if (pairs[e.key] && !e.ctrlKey && !e.metaKey) {
            e.preventDefault();
            var start = this.selectionStart, end = this.selectionEnd;
            var sel = this.value.substring(start, end);
            this.value = this.value.substring(0, start) + e.key + sel + pairs[e.key] + this.value.substring(end);
            if (sel.length > 0) {
                this.selectionStart = start + 1;
                this.selectionEnd = end + 1;
            } else {
                this.selectionStart = this.selectionEnd = start + 1;
            }
            this.dispatchEvent(new Event('input'));
            return;
        }
        if (e.key === ')' || e.key === ']') {
            if (this.value.charAt(this.selectionStart) === e.key) {
                e.preventDefault();
                this.selectionStart = this.selectionEnd = this.selectionStart + 1;
                return;
            }
        }
    });

    function doSave() {
        if (saving) return;
        saving = true;
        setStatus('saving');
        window._editorSkipReload = true;
        fetch(sourceUrl, { method: 'PUT', body: textarea.value, headers: {'Content-Type': 'text/plain'} })
            .then(function(r) { setStatus(r.ok ? 'saved' : 'error'); saving = false; })
            .catch(function() { setStatus('error'); saving = false; });
    }

    function setStatus(s) {
        var labels = { saved: 'Saved', modified: 'Modified', saving: 'Saving\u2026', error: 'Error' };
        var colors = { saved: 'var(--accent-green)', modified: 'var(--accent-yellow)', saving: 'var(--fg-muted)', error: 'var(--accent-red)' };
        status.textContent = labels[s] || '';
        status.style.color = colors[s] || '';
    }

    function updateLineNumbers() {
        if (!lineNums) return;
        var count = textarea.value.split('\n').length;
        var nums = '';
        for (var i = 1; i <= count; i++) nums += i + '\n';
        lineNums.textContent = nums;
    }

    function updateStats() {
        if (!statsEl) return;
        var text = textarea.value;
        var lines = text.split('\n').length;
        var words = text.trim() ? text.trim().split(/\s+/).length : 0;
        statsEl.textContent = lines + ' ln \u00b7 ' + words + ' words';
    }

    function wrapSelection(before, after) {
        var start = textarea.selectionStart, end = textarea.selectionEnd;
        var sel = textarea.value.substring(start, end);
        textarea.value = textarea.value.substring(0, start) + before + sel + after + textarea.value.substring(end);
        textarea.selectionStart = start + before.length;
        textarea.selectionEnd = start + before.length + sel.length;
        textarea.focus();
        textarea.dispatchEvent(new Event('input'));
    }

    function prefixLine(prefix) {
        var start = textarea.selectionStart;
        var ls = textarea.value.lastIndexOf('\n', start - 1) + 1;
        textarea.value = textarea.value.substring(0, ls) + prefix + textarea.value.substring(ls);
        textarea.selectionStart = textarea.selectionEnd = start + prefix.length;
        textarea.focus();
        textarea.dispatchEvent(new Event('input'));
    }

    document.querySelectorAll('.fmt-btn').forEach(function(btn) {
        btn.addEventListener('mousedown', function(e) { e.preventDefault(); });
        btn.onclick = function() {
            var f = btn.getAttribute('data-fmt');
            if (f === 'bold') wrapSelection('**', '**');
            else if (f === 'italic') wrapSelection('*', '*');
            else if (f === 'heading') prefixLine('## ');
            else if (f === 'code') wrapSelection('`', '`');
            else if (f === 'link') wrapSelection('[', '](url)');
            else if (f === 'list') prefixLine('- ');
            else if (f === 'quote') prefixLine('> ');
            else if (f === 'strikethrough') wrapSelection('~~', '~~');
        };
    });

    window._editorOnReload = function() {
        if (window._editorSkipReload) {
            window._editorSkipReload = false;
            return;
        }
        if (document.activeElement !== textarea) {
            loadSource();
        }
    };
})();
"#;

/// JS for the markdown editor in multi-file serve mode.
pub const JS_EDITOR_MULTI: &str = r#"
(function() {
    var textarea = document.getElementById('editor-textarea');
    var status = document.getElementById('save-status');
    var toggle = document.getElementById('editor-toggle');
    var lineNums = document.getElementById('line-numbers');
    var statsEl = document.getElementById('editor-stats');
    if (!textarea || !toggle) return;

    var currentFile = window.location.pathname.substring(1);
    var saveTimer = null;
    var saving = false;
    var loaded = false;
    window._editorSkipReload = false;
    var sourceUrl = '/' + currentFile + '/source';
    var pencilSvg = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/></svg>';
    var eyeSvg = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>';

    function updateToggleIcon() {
        var open = document.body.classList.contains('editor-open');
        toggle.innerHTML = open ? eyeSvg : pencilSvg;
        toggle.title = open ? 'Switch to preview' : 'Toggle editor';
    }

    function toggleEditor() {
        document.body.classList.toggle('editor-open');
        var open = document.body.classList.contains('editor-open');
        localStorage.setItem('md-editor-open', open);
        updateToggleIcon();
        if (open && !loaded) loadSource();
        if (!open && saveTimer) {
            clearTimeout(saveTimer);
            saveTimer = null;
            doSave();
        }
    }
    toggle.onclick = toggleEditor;

    if (localStorage.getItem('md-editor-open') === 'true') {
        document.body.classList.add('editor-open');
        updateToggleIcon();
        loadSource();
    }

    function loadSource() {
        fetch(sourceUrl).then(function(r) { return r.text(); }).then(function(text) {
            textarea.value = text;
            loaded = true;
            setStatus('saved');
            updateLineNumbers();
            updateStats();
        });
    }

    textarea.addEventListener('input', function() {
        setStatus('modified');
        clearTimeout(saveTimer);
        saveTimer = setTimeout(doSave, 800);
        updateLineNumbers();
        updateStats();
    });

    textarea.addEventListener('scroll', function() {
        if (lineNums) lineNums.scrollTop = textarea.scrollTop;
    });

    textarea.addEventListener('keydown', function(e) {
        if ((e.ctrlKey || e.metaKey) && e.key === 's') {
            e.preventDefault();
            clearTimeout(saveTimer);
            saveTimer = null;
            doSave();
            return;
        }
        if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
            e.preventDefault();
            wrapSelection('**', '**');
            return;
        }
        if ((e.ctrlKey || e.metaKey) && e.key === 'i') {
            e.preventDefault();
            wrapSelection('*', '*');
            return;
        }
        if (e.key === 'Tab') {
            e.preventDefault();
            if (e.shiftKey) {
                var start = this.selectionStart;
                var ls = this.value.lastIndexOf('\n', start - 1) + 1;
                var ch = this.value.charAt(ls);
                if (ch === '\t') {
                    this.value = this.value.substring(0, ls) + this.value.substring(ls + 1);
                    this.selectionStart = this.selectionEnd = Math.max(ls, start - 1);
                } else {
                    var m = this.value.substring(ls).match(/^ {1,4}/);
                    if (m) {
                        this.value = this.value.substring(0, ls) + this.value.substring(ls + m[0].length);
                        this.selectionStart = this.selectionEnd = Math.max(ls, start - m[0].length);
                    }
                }
            } else {
                var start = this.selectionStart, end = this.selectionEnd;
                this.value = this.value.substring(0, start) + '\t' + this.value.substring(end);
                this.selectionStart = this.selectionEnd = start + 1;
            }
            this.dispatchEvent(new Event('input'));
            return;
        }
        if (e.key === 'Enter' && !e.ctrlKey && !e.metaKey) {
            e.preventDefault();
            var start = this.selectionStart;
            var ls = this.value.lastIndexOf('\n', start - 1) + 1;
            var line = this.value.substring(ls, start);
            var indent = (line.match(/^[\t ]*/) || [''])[0];
            var ins = '\n' + indent;
            this.value = this.value.substring(0, start) + ins + this.value.substring(this.selectionEnd);
            this.selectionStart = this.selectionEnd = start + ins.length;
            this.dispatchEvent(new Event('input'));
            return;
        }
        var pairs = {'(': ')', '[': ']', '`': '`'};
        if (pairs[e.key] && !e.ctrlKey && !e.metaKey) {
            e.preventDefault();
            var start = this.selectionStart, end = this.selectionEnd;
            var sel = this.value.substring(start, end);
            this.value = this.value.substring(0, start) + e.key + sel + pairs[e.key] + this.value.substring(end);
            if (sel.length > 0) {
                this.selectionStart = start + 1;
                this.selectionEnd = end + 1;
            } else {
                this.selectionStart = this.selectionEnd = start + 1;
            }
            this.dispatchEvent(new Event('input'));
            return;
        }
        if (e.key === ')' || e.key === ']') {
            if (this.value.charAt(this.selectionStart) === e.key) {
                e.preventDefault();
                this.selectionStart = this.selectionEnd = this.selectionStart + 1;
                return;
            }
        }
    });

    function doSave() {
        if (saving) return;
        saving = true;
        setStatus('saving');
        window._editorSkipReload = true;
        fetch(sourceUrl, { method: 'PUT', body: textarea.value, headers: {'Content-Type': 'text/plain'} })
            .then(function(r) { setStatus(r.ok ? 'saved' : 'error'); saving = false; })
            .catch(function() { setStatus('error'); saving = false; });
    }

    function setStatus(s) {
        var labels = { saved: 'Saved', modified: 'Modified', saving: 'Saving\u2026', error: 'Error' };
        var colors = { saved: 'var(--accent-green)', modified: 'var(--accent-yellow)', saving: 'var(--fg-muted)', error: 'var(--accent-red)' };
        status.textContent = labels[s] || '';
        status.style.color = colors[s] || '';
    }

    function updateLineNumbers() {
        if (!lineNums) return;
        var count = textarea.value.split('\n').length;
        var nums = '';
        for (var i = 1; i <= count; i++) nums += i + '\n';
        lineNums.textContent = nums;
    }

    function updateStats() {
        if (!statsEl) return;
        var text = textarea.value;
        var lines = text.split('\n').length;
        var words = text.trim() ? text.trim().split(/\s+/).length : 0;
        statsEl.textContent = lines + ' ln \u00b7 ' + words + ' words';
    }

    function wrapSelection(before, after) {
        var start = textarea.selectionStart, end = textarea.selectionEnd;
        var sel = textarea.value.substring(start, end);
        textarea.value = textarea.value.substring(0, start) + before + sel + after + textarea.value.substring(end);
        textarea.selectionStart = start + before.length;
        textarea.selectionEnd = start + before.length + sel.length;
        textarea.focus();
        textarea.dispatchEvent(new Event('input'));
    }

    function prefixLine(prefix) {
        var start = textarea.selectionStart;
        var ls = textarea.value.lastIndexOf('\n', start - 1) + 1;
        textarea.value = textarea.value.substring(0, ls) + prefix + textarea.value.substring(ls);
        textarea.selectionStart = textarea.selectionEnd = start + prefix.length;
        textarea.focus();
        textarea.dispatchEvent(new Event('input'));
    }

    document.querySelectorAll('.fmt-btn').forEach(function(btn) {
        btn.addEventListener('mousedown', function(e) { e.preventDefault(); });
        btn.onclick = function() {
            var f = btn.getAttribute('data-fmt');
            if (f === 'bold') wrapSelection('**', '**');
            else if (f === 'italic') wrapSelection('*', '*');
            else if (f === 'heading') prefixLine('## ');
            else if (f === 'code') wrapSelection('`', '`');
            else if (f === 'link') wrapSelection('[', '](url)');
            else if (f === 'list') prefixLine('- ');
            else if (f === 'quote') prefixLine('> ');
            else if (f === 'strikethrough') wrapSelection('~~', '~~');
        };
    });

    window._editorOnReload = function() {
        if (window._editorSkipReload) {
            window._editorSkipReload = false;
            return;
        }
        if (document.activeElement !== textarea) {
            loadSource();
        }
    };
})();
"#;

/// JS for multi-file SSE live reload — sends JSON with filename.
pub const JS_LIVE_MULTI: &str = r#"
var es = new EventSource('/events');
var currentFile = window.location.pathname.substring(1);
es.onmessage = function(e) {
    try {
        var data = JSON.parse(e.data);
        if (data.file === currentFile || e.data === 'reload') {
            var scrollY = window.scrollY;
            fetch('/' + currentFile + '/raw').then(function(r) { return r.text(); }).then(function(html) {
                document.getElementById('content').innerHTML = html;
                if (window.mermaid) {
                    window.mermaid.initialize({ startOnLoad: false, theme: 'base', themeVariables: getMermaidThemeVars() });
                    document.querySelectorAll('code.language-mermaid').forEach(function(el) {
                        var pre = el.parentElement;
                        var div = document.createElement('div');
                        div.className = 'mermaid';
                        var src = el.textContent;
                        div.textContent = src;
                        div.setAttribute('data-original', src);
                        pre.replaceWith(div);
                    });
                    mermaid.run();
                }
                initCopyButtons();
                if (typeof renderMath === 'function') renderMath();
                initSidebar();
                initScrollSpy();
                window.scrollTo(0, scrollY);
                if (typeof window._editorOnReload === 'function') window._editorOnReload();
            });
        }
    } catch(_) {
        if (e.data === 'reload') { location.reload(); }
    }
};
es.onerror = function() {
    es.close();
    setTimeout(function() { location.reload(); }, 2000);
};
"#;

/// JS for search & replace in the editor (Ctrl+F / Ctrl+H).
pub const JS_EDITOR_SEARCH: &str = r#"
(function() {
    var textarea = document.getElementById('editor-textarea');
    var editorPane = document.getElementById('editor-pane');
    if (!textarea || !editorPane) return;

    var searchBar = null;
    var matches = [];
    var currentMatch = -1;
    var caseSensitive = false;
    var useRegex = false;
    var showReplace = false;

    function openSearch(withReplace) {
        showReplace = !!withReplace;
        if (searchBar) {
            searchBar.remove();
        }
        searchBar = document.createElement('div');
        searchBar.className = 'editor-search-bar';
        searchBar.innerHTML =
            '<input type="text" id="search-input" placeholder="Search...">' +
            '<span class="search-count" id="search-count"></span>' +
            '<button id="search-prev" title="Previous">&uarr;</button>' +
            '<button id="search-next" title="Next">&darr;</button>' +
            '<button class="search-toggle" id="search-case" title="Case sensitive">Aa</button>' +
            '<button class="search-toggle" id="search-regex" title="Regex">.*</button>' +
            (showReplace ? '<br><input type="text" id="replace-input" placeholder="Replace...">' +
            '<button id="replace-one">Replace</button>' +
            '<button id="replace-all">All</button>' : '') +
            '<button id="search-close" title="Close">&times;</button>';

        var formatBar = document.getElementById('editor-format-bar');
        if (formatBar) {
            formatBar.parentNode.insertBefore(searchBar, formatBar.nextSibling);
        } else {
            editorPane.insertBefore(searchBar, editorPane.firstChild);
        }

        var input = document.getElementById('search-input');
        input.focus();
        input.addEventListener('input', doSearch);
        input.addEventListener('keydown', function(e) {
            if (e.key === 'Enter') {
                e.preventDefault();
                if (e.shiftKey) goToPrev(); else goToNext();
            }
            if (e.key === 'Escape') { closeSearch(); }
        });

        document.getElementById('search-prev').onclick = goToPrev;
        document.getElementById('search-next').onclick = goToNext;
        document.getElementById('search-close').onclick = closeSearch;

        var caseBtn = document.getElementById('search-case');
        caseBtn.onclick = function() {
            caseSensitive = !caseSensitive;
            caseBtn.classList.toggle('active', caseSensitive);
            doSearch();
        };
        var regexBtn = document.getElementById('search-regex');
        regexBtn.onclick = function() {
            useRegex = !useRegex;
            regexBtn.classList.toggle('active', useRegex);
            doSearch();
        };

        if (showReplace) {
            var replaceInput = document.getElementById('replace-input');
            replaceInput.addEventListener('keydown', function(e) {
                if (e.key === 'Escape') closeSearch();
            });
            document.getElementById('replace-one').onclick = replaceOne;
            document.getElementById('replace-all').onclick = replaceAll;
        }

        // If text is selected, use it as search term
        var sel = textarea.value.substring(textarea.selectionStart, textarea.selectionEnd);
        if (sel && sel.indexOf('\n') === -1) {
            input.value = sel;
            doSearch();
        }
    }

    function closeSearch() {
        if (searchBar) {
            searchBar.remove();
            searchBar = null;
        }
        matches = [];
        currentMatch = -1;
        textarea.focus();
    }

    function doSearch() {
        var input = document.getElementById('search-input');
        var countEl = document.getElementById('search-count');
        if (!input) return;

        var query = input.value;
        matches = [];
        currentMatch = -1;

        if (!query) {
            countEl.textContent = '';
            return;
        }

        var text = textarea.value;
        try {
            if (useRegex) {
                var flags = caseSensitive ? 'g' : 'gi';
                var re = new RegExp(query, flags);
                var m;
                while ((m = re.exec(text)) !== null) {
                    matches.push({ start: m.index, end: m.index + m[0].length });
                    if (m[0].length === 0) re.lastIndex++;
                }
            } else {
                var searchText = caseSensitive ? text : text.toLowerCase();
                var searchQuery = caseSensitive ? query : query.toLowerCase();
                var idx = 0;
                while ((idx = searchText.indexOf(searchQuery, idx)) !== -1) {
                    matches.push({ start: idx, end: idx + searchQuery.length });
                    idx += searchQuery.length || 1;
                }
            }
        } catch(e) {
            // Invalid regex
            countEl.textContent = 'invalid';
            return;
        }

        if (matches.length > 0) {
            // Find closest match to cursor
            var cursor = textarea.selectionStart;
            currentMatch = 0;
            for (var i = 0; i < matches.length; i++) {
                if (matches[i].start >= cursor) { currentMatch = i; break; }
            }
            highlightMatch();
        }
        countEl.textContent = matches.length > 0 ? (currentMatch + 1) + ' of ' + matches.length : 'No results';
    }

    function highlightMatch() {
        if (currentMatch < 0 || currentMatch >= matches.length) return;
        var m = matches[currentMatch];
        textarea.focus();
        textarea.setSelectionRange(m.start, m.end);
        // Scroll into view
        var lineNum = textarea.value.substring(0, m.start).split('\n').length;
        var lineHeight = parseFloat(getComputedStyle(textarea).lineHeight) || 22;
        textarea.scrollTop = Math.max(0, (lineNum - 3) * lineHeight);

        var countEl = document.getElementById('search-count');
        if (countEl) countEl.textContent = (currentMatch + 1) + ' of ' + matches.length;
    }

    function goToNext() {
        if (matches.length === 0) return;
        currentMatch = (currentMatch + 1) % matches.length;
        highlightMatch();
    }

    function goToPrev() {
        if (matches.length === 0) return;
        currentMatch = (currentMatch - 1 + matches.length) % matches.length;
        highlightMatch();
    }

    function replaceOne() {
        if (matches.length === 0 || currentMatch < 0) return;
        var replaceInput = document.getElementById('replace-input');
        if (!replaceInput) return;
        var m = matches[currentMatch];
        var replacement = replaceInput.value;
        textarea.value = textarea.value.substring(0, m.start) + replacement + textarea.value.substring(m.end);
        textarea.dispatchEvent(new Event('input'));
        doSearch();
    }

    function replaceAll() {
        if (matches.length === 0) return;
        var replaceInput = document.getElementById('replace-input');
        if (!replaceInput) return;
        var replacement = replaceInput.value;
        // Replace from end to start to preserve positions
        for (var i = matches.length - 1; i >= 0; i--) {
            var m = matches[i];
            textarea.value = textarea.value.substring(0, m.start) + replacement + textarea.value.substring(m.end);
        }
        textarea.dispatchEvent(new Event('input'));
        doSearch();
    }

    document.addEventListener('keydown', function(e) {
        if (!document.body.classList.contains('editor-open')) return;
        if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
            e.preventDefault();
            openSearch(false);
        }
        if ((e.ctrlKey || e.metaKey) && e.key === 'h') {
            e.preventDefault();
            openSearch(true);
        }
    });
})();
"#;

/// JS for drag & drop image upload in the editor.
pub const JS_EDITOR_DRAG_DROP: &str = r#"
(function() {
    var textarea = document.getElementById('editor-textarea');
    var editorBody = document.getElementById('editor-body');
    if (!textarea || !editorBody) return;

    var overlay = null;
    var dragCounter = 0;

    function showOverlay() {
        if (overlay) return;
        overlay = document.createElement('div');
        overlay.className = 'editor-drop-overlay';
        overlay.textContent = 'Drop image to upload';
        editorBody.appendChild(overlay);
    }

    function hideOverlay() {
        if (overlay) { overlay.remove(); overlay = null; }
    }

    editorBody.addEventListener('dragenter', function(e) {
        e.preventDefault();
        dragCounter++;
        if (e.dataTransfer && e.dataTransfer.types.indexOf('Files') !== -1) {
            showOverlay();
        }
    });

    editorBody.addEventListener('dragleave', function(e) {
        e.preventDefault();
        dragCounter--;
        if (dragCounter <= 0) { dragCounter = 0; hideOverlay(); }
    });

    editorBody.addEventListener('dragover', function(e) {
        e.preventDefault();
    });

    editorBody.addEventListener('drop', function(e) {
        e.preventDefault();
        dragCounter = 0;
        hideOverlay();

        var files = e.dataTransfer ? e.dataTransfer.files : null;
        if (files && files.length > 0) {
            for (var i = 0; i < files.length; i++) {
                if (files[i].type.startsWith('image/')) {
                    uploadFile(files[i]);
                }
            }
        }
    });

    // Also handle paste
    textarea.addEventListener('paste', function(e) {
        var items = e.clipboardData ? e.clipboardData.items : [];
        for (var i = 0; i < items.length; i++) {
            if (items[i].type.startsWith('image/')) {
                e.preventDefault();
                var file = items[i].getAsFile();
                if (file) uploadFile(file);
                return;
            }
        }
    });

    function uploadFile(file) {
        var formData = new FormData();
        formData.append('file', file);

        var cursor = textarea.selectionStart;
        var placeholder = '![Uploading...]()';
        textarea.value = textarea.value.substring(0, cursor) + placeholder + textarea.value.substring(cursor);
        textarea.dispatchEvent(new Event('input'));

        fetch('/upload', { method: 'POST', body: formData })
            .then(function(r) { return r.json(); })
            .then(function(data) {
                var md = '![](' + data.path + ')';
                textarea.value = textarea.value.replace(placeholder, md);
                textarea.dispatchEvent(new Event('input'));
            })
            .catch(function() {
                textarea.value = textarea.value.replace(placeholder, '');
                textarea.dispatchEvent(new Event('input'));
            });
    }
})();
"#;

/// JS for the "New Note" card on the index page in directory mode.
pub const JS_INDEX: &str = r#"
(function() {
    var card = document.getElementById('new-note-card');
    var form = document.getElementById('new-note-form');
    var input = document.getElementById('new-note-input');
    if (!card || !form || !input) return;

    card.onclick = function() {
        card.style.display = 'none';
        form.style.display = '';
        input.value = '';
        input.focus();
    };

    input.addEventListener('keydown', function(e) {
        if (e.key === 'Enter') {
            var name = input.value.trim();
            if (!name) return;
            if (!name.endsWith('.md')) name += '.md';
            input.disabled = true;
            fetch('/create', { method: 'POST', body: name, headers: {'Content-Type': 'text/plain'} })
                .then(function(r) {
                    if (r.ok) return r.text();
                    return r.text().then(function(msg) { throw new Error(msg); });
                })
                .then(function(filename) {
                    window.location.href = '/' + encodeURIComponent(filename);
                })
                .catch(function(err) {
                    input.disabled = false;
                    input.style.borderColor = 'var(--accent-red)';
                    input.value = '';
                    input.placeholder = err.message || 'Error';
                    setTimeout(function() {
                        input.style.borderColor = '';
                        input.placeholder = 'note-name';
                    }, 2000);
                });
        }
        if (e.key === 'Escape') {
            form.style.display = 'none';
            card.style.display = '';
            input.value = '';
        }
    });
})();
"#;
