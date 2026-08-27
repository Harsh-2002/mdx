use crate::options::{FMT_WIDTH, fmt_options};

pub struct FmtArgs {
    pub file: Option<String>,
    pub in_place: bool,
    pub check: bool,
}

pub fn run(args: &FmtArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (content, file_path) = read_input(&args.file)?;

    let formatted = format_markdown(&content);

    if args.check {
        if formatted != content {
            if let Some(ref path) = file_path {
                eprintln!("{} needs formatting", path);
            } else {
                eprintln!("stdin needs formatting");
            }
            std::process::exit(1);
        }
        return Ok(());
    }

    if args.in_place {
        let path = file_path.ok_or("Cannot use --in-place with stdin")?;
        if formatted != content {
            // Atomic write
            let dir = std::path::Path::new(&path)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            let tmp = dir.join(format!(".md-fmt-tmp-{}", std::process::id()));
            std::fs::write(&tmp, &formatted)?;
            std::fs::rename(&tmp, &path)?;
            eprintln!("  Formatted {}", path);
        } else {
            eprintln!("  {} already formatted", path);
        }
    } else {
        print!("{}", formatted);
    }

    Ok(())
}

fn read_input(
    file: &Option<String>,
) -> Result<(String, Option<String>), Box<dyn std::error::Error>> {
    match file {
        Some(path) => {
            let content = std::fs::read_to_string(path)
                .map_err(|e| format!("Error reading '{}': {}", path, e))?;
            Ok((content, Some(path.clone())))
        }
        None => {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            Ok((buf, None))
        }
    }
}

fn format_markdown(input: &str) -> String {
    // fmt must parse exactly what the renderer parses. When the two sets drift,
    // fmt reparses a construct it does not recognise and writes back the
    // degraded form -- which for `--in-place` is permanent data loss. That is
    // how `> [!NOTE]` used to come back out as `> \[\!NOTE\]`.
    let options = fmt_options(FMT_WIDTH);

    let arena = typed_arena::Arena::new();
    let root = comrak::parse_document(&arena, input, &options);

    let mut result = String::new();
    comrak::format_commonmark(root, &options, &mut result).unwrap();

    // Ensure trailing newline
    if !result.ends_with('\n') {
        result.push('\n');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_preserves_gfm_alert() {
        // Regression: format_markdown built its own comrak Options and omitted
        // `extension.alerts`, so `mdx fmt --in-place` rewrote `> [!NOTE]` as
        // `> \[\!NOTE\]` and destroyed the marker in the user's own file.
        let input = "> [!NOTE]\n> Useful information.\n";
        let out = format_markdown(input);
        assert!(
            out.contains("> [!NOTE]"),
            "alert marker must survive a fmt round-trip, got: {}",
            out
        );
    }

    #[test]
    fn test_fmt_preserves_all_alert_types() {
        for kind in ["NOTE", "TIP", "IMPORTANT", "WARNING", "CAUTION"] {
            let input = format!("> [!{}]\n> Body text.\n", kind);
            let out = format_markdown(&input);
            assert!(
                out.contains(&format!("> [!{}]", kind)),
                "alert type {} must survive fmt, got: {}",
                kind,
                out
            );
        }
    }

    #[test]
    fn test_fmt_preserves_alert_custom_title() {
        let input = "> [!TIP] Pro tip\n> Body.\n";
        let out = format_markdown(input);
        assert!(
            out.contains("> [!TIP] Pro tip"),
            "custom alert title must survive fmt, got: {}",
            out
        );
    }

    #[test]
    fn test_fmt_alert_is_idempotent() {
        // Guards the shape of the emitted marker without hardcoding comrak's
        // exact spacing: whatever fmt writes, fmt must accept unchanged.
        let once = format_markdown("> [!WARNING]\n> Careful.\n");
        let twice = format_markdown(&once);
        assert_eq!(once, twice, "fmt must be stable across repeated runs");
    }

    #[test]
    fn test_fmt_leaves_unknown_bracket_blockquote_alone() {
        // `[!NOPE]` is not an alert type, so it stays ordinary blockquote
        // text. comrak's CommonMark writer escapes `[`, `!` and `]` in text
        // nodes (comrak-0.50.0 src/cm.rs:289-306), so the round-trip form is
        // `> \[\!NOPE\]` -- assert on the word, not on the raw bracket run.
        let out = format_markdown("> [!NOPE]\n> Body.\n");
        assert!(
            out.contains("NOPE"),
            "non-alert bracket text must survive fmt, got: {}",
            out
        );
        assert!(
            !out.contains("[!NOTE]"),
            "non-alert blockquote must not become an alert, got: {}",
            out
        );
        assert_eq!(
            out,
            format_markdown(&out),
            "fmt must be stable for non-alert blockquotes"
        );
    }

    #[test]
    fn test_fmt_alert_title_escapes_are_unescaped_once_then_stable() {
        // comrak unescapes the title at parse time and re-emits it literally,
        // so the first fmt pass drops the backslash. It must not keep drifting.
        let once = format_markdown("> [!NOTE] a\\_b\n> Body.\n");
        assert!(
            once.contains("> [!NOTE] a_b"),
            "alert title is unescaped once, got: {}",
            once
        );
        assert_eq!(once, format_markdown(&once), "and then must be stable");
    }
}
