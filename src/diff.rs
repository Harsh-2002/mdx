use similar::{ChangeTag, TextDiff};

pub struct DiffArgs {
    pub file_a: String,
    pub file_b: String,
    pub unified: bool,
}

pub fn run(args: &DiffArgs) -> Result<(), Box<dyn std::error::Error>> {
    let content_a = if args.file_a == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        std::fs::read_to_string(&args.file_a)
            .map_err(|e| format!("Error reading '{}': {}", args.file_a, e))?
    };

    let content_b = std::fs::read_to_string(&args.file_b)
        .map_err(|e| format!("Error reading '{}': {}", args.file_b, e))?;

    if content_a == content_b {
        eprintln!("  Files are identical");
        return Ok(());
    }

    let diff = TextDiff::from_lines(&content_a, &content_b);

    if args.unified {
        print_unified(&diff, &args.file_a, &args.file_b);
    } else {
        print_side_by_side(&diff, &args.file_a, &args.file_b);
    }

    Ok(())
}

fn print_unified<'a>(diff: &TextDiff<'a, 'a, 'a, str>, name_a: &str, name_b: &str) {
    println!("\x1b[1m--- {}\x1b[0m", name_a);
    println!("\x1b[1m+++ {}\x1b[0m", name_b);

    for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
        println!("\x1b[36m@@ {} @@\x1b[0m", hunk.header());
        for change in hunk.iter_changes() {
            let (sign, color) = match change.tag() {
                ChangeTag::Delete => ("-", "\x1b[31m"),
                ChangeTag::Insert => ("+", "\x1b[32m"),
                ChangeTag::Equal => (" ", "\x1b[90m"),
            };
            let line = change.as_str().unwrap_or("");
            let line = line.strip_suffix('\n').unwrap_or(line);
            println!("{}{}{}\x1b[0m", color, sign, line);
        }
    }
}

fn print_side_by_side<'a>(diff: &TextDiff<'a, 'a, 'a, str>, name_a: &str, name_b: &str) {
    let term_width = terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(80);
    let col_width = (term_width - 3) / 2; // 3 for " | " separator

    // Header
    println!(
        "\x1b[1m{:<col_width$}\x1b[0m | \x1b[1m{}\x1b[0m",
        truncate(name_a, col_width),
        truncate(name_b, col_width),
    );
    println!("{}", "-".repeat(term_width.min(col_width * 2 + 3)));

    for change in diff.iter_all_changes() {
        let line = change.as_str().unwrap_or("");
        let line = line.strip_suffix('\n').unwrap_or(line);

        match change.tag() {
            ChangeTag::Equal => {
                let left = truncate(line, col_width);
                let right = truncate(line, col_width);
                println!(
                    "\x1b[90m{:<col_width$}\x1b[0m | \x1b[90m{}\x1b[0m",
                    left, right
                );
            }
            ChangeTag::Delete => {
                let left = truncate(line, col_width);
                println!("\x1b[31m{:<col_width$}\x1b[0m | ", left);
            }
            ChangeTag::Insert => {
                println!(
                    "{:<col_width$} | \x1b[32m{}\x1b[0m",
                    "",
                    truncate(line, col_width)
                );
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max > 3 {
        format!("{}...", &s[..max - 3])
    } else {
        s[..max].to_string()
    }
}
