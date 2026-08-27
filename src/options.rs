//! Shared comrak option sets.
//!
//! Every entry point that hands markdown to comrak must agree on which
//! extensions are enabled. When they drift apart, the formatter reparses a
//! construct it was never told about and writes back the degraded form --
//! which for `mdx fmt --in-place` is permanent data loss in the user's file.
//! `mdx fmt` shipped without `extension.alerts`, so every `> [!NOTE]` came
//! back out as a backslash-escaped blockquote line (`> \[\!NOTE\]`): the
//! marker is gone and the escapes are permanent. One constructor, three
//! call sites.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::cli::ThemeName;

use comrak::Options;

/// Wrap column used by `mdx fmt` when re-emitting CommonMark.
///
/// Deliberately a fixed value rather than the global `--width`: `mdx fmt`
/// writes to a file, so its output has to be reproducible regardless of the
/// terminal it happens to run in. Threading the terminal width through would
/// make `mdx fmt --check` pass or fail depending on the tty it was invoked
/// from, and would reflow every file the first time it ran in a wide window.
pub const FMT_WIDTH: usize = 80;

/// The markdown constructs mdx understands, everywhere.
///
/// `comrak::Options<'c>` is generic over the lifetime borrowed by
/// `parse.broken_link_callback` and the reference map. We install neither, so
/// `'static` costs nothing and lets callers hold the value freely.
///
/// Adding an extension here is not free: the terminal renderer needs a match
/// arm for any new node kind it can now produce. Do not enable one without it.
pub fn markdown_options() -> Options<'static> {
    let mut options = Options::default();

    // GFM extensions.
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.alerts = true;

    // Docs extensions. Each needs a render arm in every target -- the terminal
    // renderer's `_ => {}` means an extension without one renders as nothing.
    options.extension.description_lists = true;
    options.extension.inline_footnotes = true;
    options.extension.highlight = true;
    options.extension.superscript = true;
    options.extension.wikilinks_title_after_pipe = true;

    // mdx extras.
    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.extension.math_dollars = true;
    options.extension.math_code = true;

    options.render.gfm_quirks = true;
    options.parse.relaxed_tasklist_matching = true;
    options.parse.tasklist_in_table = true;

    options
}

/// Whether raw HTML embedded in markdown is rendered as-is.
///
/// Off by default: markdown from a repo, a download or an agent can carry a
/// `<script>` tag, and serve/publish put the result straight into a browser.
/// Commands that want raw HTML (`serve --unsafe-html`, `publish --unsafe-html`,
/// `export`) set this once at startup, before anything is rendered, so worker
/// threads inherit the value. Process-wide on purpose: threading it through
/// every render_* signature would push render_page_multi to 8 parameters and
/// trip clippy::too_many_arguments.
static ALLOW_RAW_HTML: AtomicBool = AtomicBool::new(false);

/// Set whether raw HTML in markdown is rendered. Call once, before rendering.
pub fn set_allow_raw_html(allow: bool) {
    ALLOW_RAW_HTML.store(allow, Ordering::Relaxed);
}

/// Whether raw HTML in markdown is currently rendered.
pub fn allow_raw_html() -> bool {
    ALLOW_RAW_HTML.load(Ordering::Relaxed)
}

static TAGFILTER: AtomicBool = AtomicBool::new(true);

/// Turn GFM's tagfilter off. Only `export` does: it converts a document the
/// user named into a file they open themselves, rather than serving an origin.
pub fn set_tagfilter(on: bool) {
    TAGFILTER.store(on, Ordering::Relaxed);
}

/// Options for the HTML renderer (`serve`, `export`, `publish`).
pub fn html_options() -> Options<'static> {
    let mut options = markdown_options();
    options.extension.tagfilter = TAGFILTER.load(Ordering::Relaxed);
    // Emit id= on every heading, so `mdx toc` links resolve and deep links
    // work. Only the HTML targets: the terminal has nothing to link to. The
    // browser assets assign `heading-N` only when an id is absent, so these
    // win. Empty prefix keeps the id equal to the bare anchor.
    options.extension.header_ids = Some(String::new());
    // Raw HTML and non-http(s) links pass through only when the user opted in.
    // With this off, comrak replaces raw HTML with `<!-- raw HTML omitted -->`
    // and blanks javascript:/vbscript:/file:/data: hrefs.
    options.render.r#unsafe = allow_raw_html();
    options
}

/// Options for the CommonMark formatter (`mdx fmt`).
///
/// `width` is the wrap column; `0` disables wrapping. `parse` is deliberately
/// left at comrak's defaults on this path.
pub fn fmt_options(width: usize) -> Options<'static> {
    let mut options = markdown_options();
    options.render.width = width;
    options
}

pub const DEFAULT_SYNTAX_THEME: &str = "base16-ocean.dark";

/// How a document should look, independent of where it is rendered.
///
/// Process-wide because serve renders from spawned watcher threads and from
/// axum handlers holding only `State<Arc<AppState>>`, and watch/present take
/// no args struct at all -- none of them has a path back to the parsed flags.
#[derive(Debug, Clone)]
pub struct Presentation {
    pub theme: ThemeName,
    pub syntax_theme: String,
    pub custom_css: String,
}

impl Default for Presentation {
    fn default() -> Self {
        Self {
            theme: ThemeName::Dark,
            syntax_theme: DEFAULT_SYNTAX_THEME.to_string(),
            custom_css: String::new(),
        }
    }
}

static PRESENTATION: OnceLock<Presentation> = OnceLock::new();

/// Install presentation settings. Called once from `main`, before rendering.
pub fn set_presentation(p: Presentation) {
    let _ = PRESENTATION.set(p);
}

pub fn presentation() -> &'static Presentation {
    PRESENTATION.get_or_init(Presentation::default)
}

pub fn theme() -> &'static ThemeName {
    &presentation().theme
}

pub fn syntax_theme() -> &'static str {
    &presentation().syntax_theme
}

pub fn custom_css() -> &'static str {
    &presentation().custom_css
}

/// A bad path warns and yields no CSS rather than aborting the command.
pub fn load_css_file(path: Option<&str>) -> String {
    match path {
        Some(p) => std::fs::read_to_string(p).unwrap_or_else(|e| {
            eprintln!("Warning: could not read CSS file '{}': {}", p, e);
            String::new()
        }),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::render_fragment;

    /// The invariant this module exists to hold: no path may quietly parse a
    /// smaller subset of markdown than the terminal renderer does.
    #[test]
    fn test_every_path_enables_the_same_extensions() {
        let base = markdown_options();
        for opts in [html_options(), fmt_options(FMT_WIDTH)] {
            assert_eq!(opts.extension.strikethrough, base.extension.strikethrough);
            assert_eq!(opts.extension.table, base.extension.table);
            assert_eq!(opts.extension.autolink, base.extension.autolink);
            assert_eq!(opts.extension.tasklist, base.extension.tasklist);
            assert_eq!(opts.extension.footnotes, base.extension.footnotes);
            assert_eq!(opts.extension.alerts, base.extension.alerts);
            assert_eq!(
                opts.extension.front_matter_delimiter,
                base.extension.front_matter_delimiter
            );
            assert_eq!(opts.extension.math_dollars, base.extension.math_dollars);
            assert_eq!(opts.extension.math_code, base.extension.math_code);
        }
    }

    #[test]
    fn test_alerts_enabled_on_every_path() {
        assert!(markdown_options().extension.alerts);
        assert!(html_options().extension.alerts);
        assert!(
            fmt_options(FMT_WIDTH).extension.alerts,
            "regression: mdx fmt used to strip GFM alerts from the user's file"
        );
    }

    /// The switch is process-wide, so every assertion that depends on it lives
    /// in this one test -- two `#[test]`s flipping it would race under the
    /// parallel test runner.
    #[test]
    fn raw_html_is_off_unless_opted_in() {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                set_allow_raw_html(false);
            }
        }
        let _reset = Reset;

        // Default: off on every path.
        set_allow_raw_html(false);
        assert!(
            !html_options().render.r#unsafe,
            "raw HTML must be off unless the user opted in"
        );
        assert!(!markdown_options().render.r#unsafe);
        assert!(!fmt_options(FMT_WIDTH).render.r#unsafe);

        let out = render_fragment("<script>alert(1)</script>\n", "base16-ocean.dark");
        assert!(!out.contains("alert(1)"), "script must be dropped: {}", out);
        assert!(
            out.contains("<!-- raw HTML omitted -->"),
            "expected comrak's omitted marker: {}",
            out
        );

        let out = render_fragment("[x](javascript:alert(1))\n", "base16-ocean.dark");
        assert!(
            !out.contains("javascript:"),
            "dangerous href must be blanked: {}",
            out
        );

        // Opt-in: only the HTML path changes.
        set_allow_raw_html(true);
        assert!(html_options().render.r#unsafe);
        assert!(!markdown_options().render.r#unsafe);
        assert!(!fmt_options(FMT_WIDTH).render.r#unsafe);

        // GFM's tagfilter stays on: --unsafe-html renders raw HTML the way
        // GitHub does, so <div>/<details> pass but <script> is neutralised.
        let out = render_fragment("<div class=\"x\">kept</div>\n", "base16-ocean.dark");
        assert!(
            out.contains("<div class=\"x\">"),
            "opt-in must pass ordinary raw HTML through: {}",
            out
        );
        let out = render_fragment("<script>alert(1)</script>\n", "base16-ocean.dark");
        assert!(
            out.contains("&lt;script>"),
            "tagfilter must neutralise script even with the opt-in: {}",
            out
        );
    }
    #[test]
    fn test_only_the_fmt_path_wraps() {
        assert_eq!(fmt_options(FMT_WIDTH).render.width, 80);
        assert_eq!(fmt_options(0).render.width, 0, "0 disables wrapping");
        assert_eq!(markdown_options().render.width, 0);
        assert_eq!(html_options().render.width, 0);
    }

    #[test]
    fn test_fmt_path_keeps_footnote_definitions_folded() {
        // fmt_options touches only `render`; `parse` stays at comrak's
        // defaults. Pinned so a future edit that reaches into `parse` on the
        // fmt path has to justify itself here.
        assert!(!fmt_options(FMT_WIDTH).parse.leave_footnote_definitions);
    }
}
