use comrak::Options;

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
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.extension.math_dollars = true;
    options.extension.math_code = true;
    options.render.width = 80;

    let arena = typed_arena::Arena::new();
    let root = comrak::parse_document(&arena, input, &options);

    let mut output = String::new();
    comrak::format_commonmark(root, &options, &mut output).unwrap();
    let mut result = output;

    // Ensure trailing newline
    if !result.ends_with('\n') {
        result.push('\n');
    }

    result
}
