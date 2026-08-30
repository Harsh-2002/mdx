# mdx

A fast terminal markdown renderer and toolchain in Rust. Renders markdown with syntax highlighting, tables, images and Unicode math directly in your terminal — plus a browser preview, formatter, linter, diff, search, format converter, web page fetcher and static site generator.

## Install

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/Harsh-2002/mdx/main/install.sh | sh

# Windows (PowerShell)
irm https://raw.githubusercontent.com/Harsh-2002/mdx/main/install.ps1 | iex

# From source
cargo install --git https://github.com/Harsh-2002/mdx --features serve
```

Installs the binary and sets up completions for bash, zsh and fish. Other shells are skipped.

## Usage

```bash
mdx file.md                          # render in terminal
mdx file.md --pager                  # page through less
cat README.md | mdx                  # read from stdin
mdx https://example.com/doc.md       # render markdown from a URL
mdx https://example.com              # or extract an article from a page

mdx serve file.md                    # browser preview with live reload
mdx serve ./notes/                   # serve a directory as a note-taking app
mdx serve a.md b.md                  # multiple files with sidebar navigation

mdx fetch https://example.com        # web page to markdown
mdx search "query" ./docs/           # BM25 full-text search
mdx export --to pdf file.md          # html | pdf | epub | json | txt

mdx stats file.md                    # word count, headings, reading time
mdx toc file.md                      # table of contents
mdx fmt --check file.md              # format, or check formatting in CI
mdx lint file.md                     # broken links and common issues
mdx diff old.md new.md               # colored side-by-side diff

mdx present file.md                  # terminal slides, split on ---
mdx watch file.md                    # re-render on change
mdx publish ./blog --out ./dist      # static site
mdx update                           # self-update
mdx completions install              # shell completions
```

Run `mdx <command> --help` for per-command flags.

## Options

Accepted before or after any subcommand — `mdx --plain fetch URL` and `mdx fetch URL --plain` are the same command.

| Flag | Honored by |
|------|------------|
| `-w, --width <N>` | terminal output |
| `--color auto\|always\|never` | terminal output |
| `--plain` | terminal output |
| `--theme dark\|light` | terminal, serve, watch, present, publish, `export --to html` |
| `--syntax-theme <NAME>` | every target that highlights code |
| `--css <FILE>` | serve, `export --to html`, publish |

Top level only: `-p, --pager` (`mdx fetch` has its own `-p`; `mdx serve` uses `-p` for `--port`), `--list-syntax-themes`, `--generate-man`.

## `mdx serve`

Live browser preview with hot reload. Includes a markdown editor (`e`), search and replace (`Ctrl+F` / `Ctrl+H`), drag-and-drop image upload into `assets/`, a table-of-contents sidebar, dark/light toggle (`t`) and print/PDF export. Agents sending `Accept: text/markdown` get raw markdown with `X-Markdown-Tokens` and `Vary: Accept`.

```bash
mdx serve file.md -p 8080            # specific port
mdx serve file.md --host 0.0.0.0     # expose on your LAN (read below)
mdx serve file.md --unsafe-html      # render raw HTML in the document
mdx --css custom.css serve file.md   # inject custom CSS
```

**Security defaults.** The server is unauthenticated — anyone who can reach the port can read your files, overwrite them, create new ones and upload into `assets/`. So:

- **Binds `127.0.0.1`.** `--host 0.0.0.0` exposes it to your network.
- **Answers only on an address** (`127.0.0.1`, `::1`, `localhost`, a LAN IP), never a domain name. Binding loopback alone does not stop a web page pointing a domain at `127.0.0.1` and reaching the server through your browser; requiring an address in `Host` does. Writes carrying a cross-origin `Origin` are refused.
- **Raw HTML is dropped**, so a `<script>` in a downloaded or agent-written document cannot run. `--unsafe-html` renders it the way GitHub does: `<div>` and `<details>` pass, `<script>` and `<iframe>` stay neutralised. Note an HTML block runs to the next blank line, so `<details>` collapsibles and `<div align="center">` badge blocks are dropped whole — tags *and* content. Markdown images, tables, code fences, math, mermaid and alerts are unaffected.

`mdx export` is different: it writes a file you open yourself, so it converts your document faithfully, tags and all.

## `mdx fetch`

Downloads a page, extracts the main content with readability, and renders clean markdown. Piped, it writes raw markdown — good for LLM pipelines. Supports [Markdown for Agents](https://developers.cloudflare.com/fundamentals/reference/markdown-for-agents/): sends `Accept: text/markdown` so MFA sites return pre-converted markdown.

| Flag | Description |
|------|-------------|
| `-o, --output <FILE>` | Write to a file instead of stdout |
| `--raw` | Convert the whole page, skipping readability |
| `--metadata` | Prepend YAML front matter (title, author, date, source) |
| `--tokens` | Print an estimated token count to stderr |
| `-p, --pager` | Page the rendered output |
| `--json` | Emit a JSON object instead of markdown |
| `--max-tokens <N>` | Truncate to roughly N tokens, at a block boundary |

`mdx <url>` runs the same pipeline but always renders for the terminal.

`--json` carries the requested and final URLs, HTTP status, content type, how the content was extracted (`server-markdown`, `readability` or `raw`), byte count, elapsed time, token estimate, metadata, any warnings, and the markdown itself — so a pipeline can tell a clean extraction from a fallback without parsing stderr.

```bash
mdx fetch --json https://example.com | jq -r .content
mdx fetch --json --max-tokens 4000 URL | jq '{tokens, truncated, warnings}'
```

`--max-tokens` cuts at the last block boundary under the budget, never inside a fenced code block, and appends `*[truncated]*`. Token counts are a `chars/4` estimate, not a tokenizer.

## `mdx export`

`-o` writes to a file in every format. Without it, HTML, JSON and TXT print to stdout while PDF and EPUB default to the input name with a new extension.

HTML output is a single file, but math and mermaid load KaTeX and mermaid.js from a CDN, so those two need a network connection to display. PDF output is byte-reproducible: exporting the same document twice gives the same file, and `SOURCE_DATE_EPOCH` sets the embedded timestamp. EPUB embeds local images, splits chapters on headings, and maps front matter to metadata. PDF renders mermaid with a local [mmdc](https://github.com/mermaid-js/mermaid-cli); without it the diagram is written in as a labelled source block and the export still succeeds. `--allow-remote-render` falls back to the kroki.io web API instead, which uploads your diagram source to a third party — so it is off by default.

## Markdown support

GFM plus the extensions technical documentation actually uses. This table is backed by `tests/parity_test.rs`, which fails if a cell drifts.

| Construct | Terminal | HTML / serve | txt | JSON |
|---|---|---|---|---|
| Headings, emphasis, strikethrough | ✅ | ✅ | ✅ | ✅ |
| Inline code, code blocks | ✅ | ✅ | ✅ | ✅ |
| Links, autolinks | ✅ | ✅ | ✅ | ✅ |
| Images | ✅ | ✅ | alt text only | ✅ |
| Lists, ordered lists, task lists | ✅ | ✅ | ✅ | ✅ |
| Tables | ✅ | ✅ | ✅ | ✅ |
| Block quotes, GFM alerts | ✅ | ✅ | ✅ | ✅ |
| Footnotes, inline footnotes | ✅ | ✅ | ✅ | ✅ |
| Math (inline and display) | Unicode | KaTeX | ✅ | ✅ |
| Mermaid | source | SVG | ✅ | ✅ |
| Description lists | ✅ | ✅ | ✅ | ✅ |
| `==highlight==`, `^superscript^` | ✅ | ✅ | ✅ | ✅ |
| `[[wikilinks]]` | ✅ | ✅ | ✅ | ✅ |
| Raw HTML | ✅ | opt-in | ✅ | ✅ |
| Front matter | stripped | stripped | stripped | ✅ |

PDF and EPUB render the same document model; they are absent here only because their output is binary rather than text-comparable.

## Front matter

`publish`, `export --to epub`, `search --tag` and `fetch --metadata` read YAML front matter:

```yaml
---
title: My Post
date: 2024-01-15
author: Jane
lang: en
tags: rust, cli
draft: true
---
```

## Credits

Built on [comrak](https://github.com/kivikakk/comrak) (parsing), [syntect](https://github.com/trishume/syntect) (highlighting), [clap](https://github.com/clap-rs/clap) (CLI), [axum](https://github.com/tokio-rs/axum) + [tokio](https://github.com/tokio-rs/tokio) (serve), [notify](https://github.com/notify-rs/notify) (live reload), [ratatui](https://github.com/ratatui/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm) (watch/present), [similar](https://github.com/mitsuhiko/similar) (diff), [rayon](https://github.com/rayon-rs/rayon) + [walkdir](https://github.com/BurntSushi/walkdir) (search), [textwrap](https://github.com/mgeisler/textwrap), [image](https://github.com/image-rs/image), [genpdfi](https://github.com/theiskaa/genpdfi) + [printpdf](https://github.com/fschutt/printpdf) (PDF), [epub-builder](https://github.com/lise-henry/epub-builder), [ureq](https://github.com/algesten/ureq), [dom_smoothie](https://github.com/niklak/dom_smoothie) (readability) and [htmd](https://github.com/letmutex/htmd).

## License

MIT
