//! Feature x target parity matrix.
//!
//! Every row pins what a construct does in each render target *today*. A cell
//! marked `Gap` asserts the construct is still missing, so closing the gap
//! fails this test and forces the table to be updated — the matrix cannot
//! silently drift from reality.

use std::process::Command;

#[derive(Clone, Copy)]
enum Cell {
    /// Every probe must appear.
    Ok,
    /// Every probe must appear, but this target renders differently enough to
    /// need its own.
    OkWith(Probes),
    /// At least one probe must be absent. The string says what is lost.
    Gap(&'static str),
}

use Cell::{Gap, Ok as Full, OkWith};

/// Outer slice: every group must match. Inner slice: any one alternative.
type Probes = &'static [&'static [&'static str]];

struct Row {
    fixture: &'static str,
    probes: Probes,
    term: Cell,
    html: Cell,
    txt: Cell,
    json: Cell,
}

const MATRIX: &[Row] = &[
    Row {
        fixture: "heading",
        probes: &[&["heading one"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "emphasis",
        probes: &[&["bolded"], &["italicised"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "strikethrough",
        probes: &[&["struck"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "inline_code",
        probes: &[&["parity_probe"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "code_block",
        probes: &[&["parity_probe"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "link",
        probes: &[&["probe link"], &["example.com/probe"]],
        term: Full,
        html: Full,
        txt: Gap("txt keeps the label, drops the URL"),
        json: Full,
    },
    Row {
        fixture: "autolink",
        probes: &[&["example.com/autoprobe"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "image_local",
        probes: &[&["probe alt"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "list_nested",
        probes: &[&["outer probe"], &["inner probe"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "list_ordered",
        probes: &[&["first probe"], &["second probe"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "task_list",
        probes: &[&["undone probe"], &["done probe"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "table",
        probes: &[&["head probe"], &["cell probe"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "blockquote",
        probes: &[&["quoted probe"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "alert",
        probes: &[&["alert probe"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "footnote",
        probes: &[&["text probe"], &["footnote probe"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "math_inline",
        probes: &[&["x"]],
        term: Full,
        html: Full,
        txt: Gap("txt has no Math arm; the expression is deleted"),
        json: Full,
    },
    Row {
        fixture: "math_display",
        probes: &[&["alpha"]],
        // The terminal substitutes the glyph; HTML emits the source for KaTeX
        // to render in the browser. Both are correct, and different.
        term: OkWith(&[&["\u{3b1}"]]),
        html: Full,
        txt: Gap("txt has no Math arm; the expression is deleted"),
        json: Gap("ast_to_json emits type \"math\" with no literal"),
    },
    Row {
        fixture: "math_frac",
        probes: &[&["frac"]],
        term: Gap("render/math.rs deletes \\frac, leaving {a}{b}"),
        html: Full,
        txt: Gap("txt has no Math arm; the expression is deleted"),
        json: Gap("ast_to_json emits type \"math\" with no literal"),
    },
    Row {
        fixture: "mermaid",
        probes: &[&["graph TD"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "raw_html",
        probes: &[&["html probe"]],
        term: Full,
        html: Full,
        txt: Gap("extract_plain_text has no HtmlBlock arm"),
        json: Gap("ast_to_json keeps the node type, drops the literal"),
    },
    Row {
        fixture: "thematic_break",
        probes: &[&["before"], &["after"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "hard_break",
        probes: &[&["line one probe"], &["line two probe"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "front_matter",
        probes: &[&["body probe"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "html_entity",
        probes: &[&["at&t", "at&amp;t"], &["probe"]],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
    Row {
        fixture: "unicode",
        probes: &[
            &["\u{65e5}\u{672c}\u{8a9e}"],
            &["\u{420}\u{443}\u{441}\u{441}\u{43a}\u{438}\u{439}"],
        ],
        term: Full,
        html: Full,
        txt: Full,
        json: Full,
    },
];

fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/parity"
    ))
    .join(format!("{}.md", name))
}

fn render(target: &str, fixture: &str) -> String {
    let args: Vec<&str> = match target {
        "term" => vec!["-w", "100"],
        "html" => vec!["export", "--to", "html"],
        "txt" => vec!["export", "--to", "txt"],
        "json" => vec!["export", "--to", "json"],
        other => panic!("unknown target {}", other),
    };
    let out = Command::new(env!("CARGO_BIN_EXE_mdx"))
        .args(&args)
        .arg(fixture_path(fixture))
        .env("NO_COLOR", "1")
        .output()
        .unwrap_or_else(|e| panic!("failed to run mdx for {}/{}: {}", target, fixture, e));
    assert!(
        out.status.success(),
        "{} {} exited {}: {}",
        target,
        fixture,
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_lowercase()
}

fn missing(out: &str, probes: Probes) -> Vec<String> {
    probes
        .iter()
        .filter(|group| !group.iter().any(|p| out.contains(&p.to_lowercase())))
        .map(|group| group.join(" | "))
        .collect()
}

fn check(row: &Row, target: &str, cell: Cell) {
    let out = render(target, row.fixture);
    let probes = match cell {
        Cell::OkWith(p) => p,
        _ => row.probes,
    };
    let missing = missing(&out, probes);
    match cell {
        Cell::Ok | Cell::OkWith(_) => assert!(
            missing.is_empty(),
            "{} / {}: expected supported, but {:?} did not survive",
            row.fixture,
            target,
            missing
        ),
        Cell::Gap(why) => assert!(
            !missing.is_empty(),
            "{} / {} is recorded as a gap ({}) but everything survived \u{2014} \
             the gap is closed, so update MATRIX in tests/parity_test.rs",
            row.fixture,
            target,
            why
        ),
    }
}

#[test]
fn parity_terminal() {
    for row in MATRIX {
        check(row, "term", row.term);
    }
}

#[test]
fn parity_html() {
    for row in MATRIX {
        check(row, "html", row.html);
    }
}

#[test]
fn parity_txt() {
    for row in MATRIX {
        check(row, "txt", row.txt);
    }
}

#[test]
fn parity_json() {
    for row in MATRIX {
        check(row, "json", row.json);
    }
}

/// Every fixture on disk must appear in MATRIX, so adding one without a row is
/// a failure rather than silently untested coverage.
#[test]
fn every_fixture_is_in_the_matrix() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/parity");
    let mut on_disk: Vec<String> = std::fs::read_dir(dir)
        .expect("parity fixture dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .map(|e| e.path().file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    on_disk.sort();

    let mut in_matrix: Vec<String> = MATRIX.iter().map(|r| r.fixture.to_string()).collect();
    in_matrix.sort();

    assert_eq!(on_disk, in_matrix, "fixture directory and MATRIX disagree");
}
