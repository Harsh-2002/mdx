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

    // mdx extras.
    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.extension.math_dollars = true;
    options.extension.math_code = true;

    // `extension.header_ids` is intentionally left at its default of `None`:
    // the HTML assets assign heading anchors client-side.

    options
}

/// Options for the HTML renderer (`serve`, `export`, `publish`).
pub fn html_options() -> Options<'static> {
    let mut options = markdown_options();
    // Raw HTML and non-http(s) links pass through untouched.
    options.render.r#unsafe = true;
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_only_the_html_path_allows_raw_html() {
        assert!(html_options().render.r#unsafe);
        assert!(!markdown_options().render.r#unsafe);
        assert!(!fmt_options(FMT_WIDTH).render.r#unsafe);
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
