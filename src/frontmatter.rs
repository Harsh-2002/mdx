use std::io::Read;
use std::path::Path;

pub struct FrontMatter {
    pub title: Option<String>,
    pub date: Option<String>,
    pub tags: Vec<String>,
    pub draft: bool,
}

pub fn parse(content: &str) -> FrontMatter {
    let mut fm = FrontMatter {
        title: None,
        date: None,
        tags: Vec::new(),
        draft: false,
    };

    if !content.starts_with("---") {
        return fm;
    }

    let rest = &content[3..];
    if let Some(end) = rest.find("\n---") {
        let block = &rest[..end];
        for line in block.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                match key.as_str() {
                    "title" => fm.title = Some(value),
                    "date" => fm.date = Some(value),
                    "tags" => {
                        fm.tags = value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    "draft" => fm.draft = value == "true",
                    _ => {}
                }
            }
        }
    }

    fm
}

pub fn strip(content: &str) -> &str {
    if !content.starts_with("---") {
        return content;
    }
    let rest = &content[3..];
    if let Some(end) = rest.find("\n---") {
        let after = &rest[end + 4..];
        after.trim_start_matches('\n')
    } else {
        content
    }
}

/// Read just the first 4KB of a file to extract front matter.
/// Used for early tag filtering to avoid reading entire files.
pub fn read_front_matter_only(path: &Path) -> Result<FrontMatter, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = [0u8; 4096];
    let n = file.read(&mut buf)?;
    let content = String::from_utf8_lossy(&buf[..n]);
    Ok(parse(&content))
}
