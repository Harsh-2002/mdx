use std::path::PathBuf;

use comrak::nodes::{AlertType, AstNode, ListType, NodeValue};
#[cfg(not(feature = "images"))]
use genpdfi::Position;
#[cfg(feature = "images")]
use genpdfi::Scale;
use genpdfi::elements;
use genpdfi::style;
use genpdfi::{Element, Mm};

use crate::parse::{CodeStyle, inline_text, parse_markdown};

/// A4 content area dimensions (margins: 20/15/20/15mm from 210×297mm)
#[cfg(feature = "images")]
const CONTENT_WIDTH_MM: f32 = 180.0;
#[cfg(feature = "images")]
const MAX_IMAGE_HEIGHT_MM: f32 = 230.0;

pub struct ExportArgs {
    pub file: Option<String>,
    pub to: String,
    pub output: Option<String>,
    /// Opt-in to rendering mermaid diagrams via the kroki.io web API, which
    /// uploads the diagram source. Off unless the user passes the flag.
    pub allow_remote_render: bool,
}

/// Write `content` to `output`, or to stdout when no path was given.
///
/// `-o` used to be read only by the pdf and epub arms, so
/// `mdx export --to html -o out.html f.md` exited 0 having written nothing.
fn emit(content: &str, output: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    match output {
        Some(path) => {
            if let Some(parent) = std::path::Path::new(path).parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Cannot create '{}': {}", parent.display(), e))?;
            }
            std::fs::write(path, content).map_err(|e| format!("Cannot write '{}': {}", path, e))?;
            eprintln!("  Wrote {}", path);
        }
        None => {
            use std::io::Write;
            let stdout = std::io::stdout();
            let mut w = stdout.lock();
            // A closed pipe (`| head`) is not an error worth reporting.
            if let Err(e) = w.write_all(content.as_bytes())
                && e.kind() != std::io::ErrorKind::BrokenPipe
            {
                return Err(Box::new(e));
            }
        }
    }
    Ok(())
}

/// Derive an output path from the input by swapping its extension.
///
/// Uses `Path::with_extension` rather than a string replace: `f.replace(".md",
/// ".pdf")` is global, so `notes.md.backup.md` became `notes.pdf.backup.pdf`,
/// and a directory named `v1.markdown/` was renamed in the output path too.
fn default_output_path(file: Option<&str>, ext: &str) -> String {
    match file {
        Some(f) => std::path::Path::new(f)
            .with_extension(ext)
            .to_string_lossy()
            .into_owned(),
        None => format!("output.{}", ext),
    }
}

pub fn run(args: &ExportArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Export converts a document the user named into a file they asked for, so
    // it keeps rendering raw HTML. serve and publish default the other way and
    // gate it behind --unsafe-html.
    crate::options::set_allow_raw_html(true);
    // export writes a file the user opens, not an origin: convert their
    // document faithfully rather than applying GFM's tag filter.
    crate::options::set_tagfilter(false);

    let content = read_input(&args.file)?;
    let title = args.file.as_deref().unwrap_or("document");

    if args.allow_remote_render && args.to != "pdf" {
        eprintln!("  --allow-remote-render only affects --to pdf; ignoring");
    }

    match args.to.as_str() {
        "html" => {
            let html = crate::html::render_standalone(
                &content,
                crate::options::syntax_theme(),
                crate::options::theme(),
                title,
                crate::options::custom_css(),
            );
            emit(&html, args.output.as_deref())?;
        }
        "json" => {
            let arena = typed_arena::Arena::new();
            let root = parse_markdown(&arena, &content);
            let json = ast_to_json(root, 0);
            emit(&format!("{}\n", json), args.output.as_deref())?;
        }
        "txt" => {
            let arena = typed_arena::Arena::new();
            let root = parse_markdown(&arena, &content);
            let text = extract_plain_text(root);
            emit(&text, args.output.as_deref())?;
        }
        "pdf" => {
            let output_path = args
                .output
                .clone()
                .unwrap_or_else(|| default_output_path(args.file.as_deref(), "pdf"));
            export_pdf(&content, &output_path, args.allow_remote_render)?;
        }
        "epub" => {
            let output_path = args
                .output
                .clone()
                .unwrap_or_else(|| default_output_path(args.file.as_deref(), "epub"));
            export_epub(&content, &output_path, args.file.as_deref())?;
        }
        other => {
            return Err(format!(
                "Unsupported format: '{}'. Supported: html, json, txt, pdf, epub",
                other
            )
            .into());
        }
    }

    Ok(())
}

// ─── PDF Export via genpdfi ───────────────────────────────────────────────────

/// Sans-serif font file stems to look for, best first.
///
/// Helvetica/Arial first for metric fidelity, then the metric-compatible
/// clones every mainstream distro ships. Nimbus Sans is the URW Helvetica
/// clone installed with ghostscript.
const SANS_CANDIDATES: &[&str] = &[
    "Helvetica",
    "Arial",
    "LiberationSans-Regular",
    "DejaVuSans",
    "NimbusSans-Regular",
    "FreeSans",
    "Verdana",
];

/// Monospace font file stems to look for, best first.
const MONO_CANDIDATES: &[&str] = &[
    "Courier",
    "CourierNew",
    "LiberationMono-Regular",
    "DejaVuSansMono",
    "NimbusMonoPS-Regular",
    "FreeMono",
];

/// Directories to search for fonts, per platform.
fn font_search_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = if cfg!(target_os = "macos") {
        vec![
            "/System/Library/Fonts".into(),
            "/System/Library/Fonts/Supplemental".into(),
            "/Library/Fonts".into(),
        ]
    } else if cfg!(target_os = "windows") {
        vec!["C:\\Windows\\Fonts".into()]
    } else {
        vec![
            "/usr/share/fonts".into(),
            "/usr/local/share/fonts".into(),
            "/usr/share/texmf/fonts".into(),
        ]
    };
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        dirs.push(home.join(".fonts"));
        dirs.push(home.join(".local/share/fonts"));
    }
    dirs
}

/// Find a usable font file by trying each candidate stem in turn.
///
/// markdown2pdf's own lookup reads only the direct children of a few hardcoded
/// directories, but Debian and Ubuntu store fonts one level down
/// (`/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`), so it finds nothing and
/// every PDF export fails with "Could not find a font for built-in metrics" --
/// on the GitHub ubuntu runners too. Walk recursively instead.
fn find_font_file(candidates: &[&str]) -> Option<PathBuf> {
    let dirs = font_search_dirs();
    for stem in candidates {
        let want: Vec<String> = ["ttf", "otf"]
            .iter()
            .map(|ext| format!("{}.{}", stem, ext).to_lowercase())
            .collect();
        for dir in &dirs {
            if !dir.is_dir() {
                continue;
            }
            for entry in walkdir::WalkDir::new(dir)
                .max_depth(4)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_lowercase();
                // .ttc collections are not supported by the font parser.
                if name.ends_with(".ttc") {
                    continue;
                }
                if want.contains(&name) {
                    return Some(entry.into_path());
                }
            }
        }
    }
    None
}

/// Load a PDF font family, preferring a real system font file.
///
/// Falls back to markdown2pdf's built-in metrics loader so macOS and Windows,
/// where the hardcoded lookup does work, behave exactly as before.
fn load_pdf_font(
    candidates: &[&str],
    builtin: &str,
) -> Result<genpdfi::fonts::FontFamily<genpdfi::fonts::FontData>, Box<dyn std::error::Error>> {
    if let Some(path) = find_font_file(candidates)
        && let Ok(family) =
            markdown2pdf::fonts::load_font_family(markdown2pdf::fonts::FontSource::File(path))
    {
        return Ok(family);
    }
    markdown2pdf::fonts::load_builtin_font_family(builtin).map_err(|e| {
        format!(
            "Font error: {}. Searched for {:?} under {:?}.",
            e,
            candidates,
            font_search_dirs()
        )
        .into()
    })
}

const HEADING_SIZES: [u8; 6] = [24, 20, 16, 14, 12, 11];

/// Width of the A4 content area in mm (210mm page, 15mm side margins).
const RULE_WIDTH_MM: f32 = 180.0;

/// Font size of the decorative rule under H1/H2.
const RULE_FONT_SIZE: u8 = 4;

/// Smallest size a heading will be shrunk to in order to fit.
const MIN_HEADING_SIZE: u8 = 8;

/// Shrink a heading until its longest unbreakable word fits the content width.
///
/// genpdfi breaks lines on whitespace, so a single word wider than the content
/// area has no break opportunity and aborts the entire export with "Page
/// overflowed while trying to wrap a string" -- one long word in one heading
/// takes the whole document down. Stepping the size down keeps every character
/// rather than truncating, and only affects headings that would otherwise fail.
///
/// Bounded by `MIN_HEADING_SIZE`, below which a heading stops being readable.
/// A single word still too wide at that size -- roughly 100+ characters with no
/// break in it -- will still fail the export.
fn fit_heading_size(doc: &genpdfi::Document, text: &str, size: u8) -> u8 {
    let Some(longest) = text.split_whitespace().max_by_key(|w| w.chars().count()) else {
        return size;
    };
    let mut size = size;
    while size > MIN_HEADING_SIZE {
        let width: f32 = style::Style::new()
            .with_font_size(size)
            .bold()
            .str_width(doc.font_cache(), longest)
            .into();
        if !width.is_finite() || width <= RULE_WIDTH_MM {
            break;
        }
        size -= 1;
    }
    size
}

/// How many rule glyphs fit across the content width.
///
/// This was a hardcoded `repeat(200)`. Under markdown2pdf's built-in Helvetica
/// metrics that happened not to overflow, but a real embedded font reports true
/// advance widths, and 200 box-drawing glyphs are far wider than the page. The
/// string contains no spaces, so genpdfi has no break opportunity and the whole
/// export dies with "Page overflowed while trying to wrap a string" -- on every
/// document with an H1 or H2.
fn rule_repeat_count(doc: &genpdfi::Document, rule_char: &str, size: u8) -> usize {
    let style = style::Style::new().with_font_size(size);
    let width: f32 = style.str_width(doc.font_cache(), rule_char).into();
    if !width.is_finite() || width <= 0.0 {
        // Font gave us nothing usable; pick a count that fits any plausible glyph.
        return 60;
    }
    ((RULE_WIDTH_MM / width).floor() as usize).clamp(1, 400)
}

/// A wrapper element that draws a filled background color behind its content.
/// Uses PDF layers: background on current layer, content on next layer (on top).
struct FilledElement<E: Element> {
    element: E,
    bg_color: style::Color,
    pad_v: Mm,
    pad_h: Mm,
    corner_radius: f32,
}

impl<E: Element> FilledElement<E> {
    fn new(element: E, bg_color: style::Color, pad_v: impl Into<Mm>, pad_h: impl Into<Mm>) -> Self {
        Self {
            element,
            bg_color,
            pad_v: pad_v.into(),
            pad_h: pad_h.into(),
            corner_radius: 4.0,
        }
    }

    fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
}

impl<E: Element> Element for FilledElement<E> {
    fn render(
        &mut self,
        context: &genpdfi::Context,
        area: genpdfi::render::Area<'_>,
        style: style::Style,
    ) -> Result<genpdfi::RenderResult, genpdfi::error::Error> {
        // Render content on the NEXT PDF layer (will appear visually on top)
        let mut content_area = area.next_layer();
        content_area.add_margins(genpdfi::Margins::vh(self.pad_v, self.pad_h));

        let mut result = self.element.render(context, content_area, style)?;

        // Calculate total size including padding, capped to available area
        let total_height = result.size.height + self.pad_v + self.pad_v;
        let area_height = area.size().height;
        let total_height = if total_height > area_height {
            area_height
        } else {
            total_height
        };
        let total_width = area.size().width;

        // Draw background on the ORIGINAL layer (behind text)
        draw_filled_background(
            self.bg_color,
            total_width,
            total_height,
            self.corner_radius,
            context,
            area,
            style,
        );

        result.size.width = total_width;
        result.size.height = total_height;

        Ok(result)
    }
}

/// Draw a filled background behind content. Tries rounded corners (image-based)
/// first, falls back to a thick-line rectangle if that fails or images feature is off.
#[cfg(feature = "images")]
fn draw_filled_background(
    color: style::Color,
    total_width: Mm,
    total_height: Mm,
    corner_radius: f32,
    context: &genpdfi::Context,
    area: genpdfi::render::Area<'_>,
    style: style::Style,
) {
    let w_f32: f32 = total_width.into();
    let h_f32: f32 = total_height.into();
    if !render_rounded_bg_on_area(w_f32, h_f32, corner_radius, color, context, area, style) {
        // Fallback: image rendering failed, but area was consumed. No background drawn.
        // This only happens if temp dir is unwritable or image encoding fails.
    }
}

#[cfg(not(feature = "images"))]
fn draw_filled_background(
    color: style::Color,
    total_width: Mm,
    total_height: Mm,
    _corner_radius: f32,
    _context: &genpdfi::Context,
    area: genpdfi::render::Area<'_>,
    _style: style::Style,
) {
    let mid_y = total_height / 2.0;
    let bg_style = style::LineStyle::new()
        .with_thickness(total_height)
        .with_color(color);
    area.draw_line(
        vec![
            Position::new(Mm::from(0), mid_y),
            Position::new(total_width, mid_y),
        ],
        bg_style,
    );
}

/// Create a rounded-rect background image, save to temp file, load via genpdfi,
/// and render it on the given area. Returns true on success.
/// Uses temp file to bridge image 0.25 (our crate) → image 0.24 (genpdfi's crate).
#[cfg(feature = "images")]
fn render_rounded_bg_on_area(
    width_mm: f32,
    height_mm: f32,
    radius_mm: f32,
    color: style::Color,
    context: &genpdfi::Context,
    area: genpdfi::render::Area<'_>,
    pdf_style: style::Style,
) -> bool {
    let dpi = 144.0_f32;
    let px_w = (width_mm * dpi / 25.4).round().max(1.0) as u32;
    let px_h = (height_mm * dpi / 25.4).round().max(1.0) as u32;
    // Clamp radius so it never exceeds half the smaller dimension
    let r = ((radius_mm * dpi / 25.4).round() as u32)
        .min(px_w / 2)
        .min(px_h / 2);

    let (cr, cg, cb) = match color {
        style::Color::Rgb(r, g, b) => (r, g, b),
        _ => (243, 244, 248),
    };

    let mut img = image::RgbImage::from_pixel(px_w, px_h, image::Rgb([255, 255, 255]));

    if r > 0 {
        // Corner centers: inset by r from each edge
        let cx_left = r;
        let cx_right = px_w.saturating_sub(r);
        let cy_top = r;
        let cy_bottom = px_h.saturating_sub(r);
        let r_sq = r * r;

        for y in 0..px_h {
            for x in 0..px_w {
                let inside = if x < cx_left && y < cy_top {
                    // Top-left corner
                    let dx = cx_left - x;
                    let dy = cy_top - y;
                    dx * dx + dy * dy <= r_sq
                } else if x >= cx_right && y < cy_top {
                    // Top-right corner
                    let dx = x - cx_right;
                    let dy = cy_top - y;
                    dx * dx + dy * dy <= r_sq
                } else if x < cx_left && y >= cy_bottom {
                    // Bottom-left corner
                    let dx = cx_left - x;
                    let dy = y - cy_bottom;
                    dx * dx + dy * dy <= r_sq
                } else if x >= cx_right && y >= cy_bottom {
                    // Bottom-right corner
                    let dx = x - cx_right;
                    let dy = y - cy_bottom;
                    dx * dx + dy * dy <= r_sq
                } else {
                    true
                };

                if inside {
                    img.put_pixel(x, y, image::Rgb([cr, cg, cb]));
                }
            }
        }
    } else {
        // No rounding needed — fill entire image
        for y in 0..px_h {
            for x in 0..px_w {
                img.put_pixel(x, y, image::Rgb([cr, cg, cb]));
            }
        }
    }

    // Save as PNG (lossless — avoids JPEG compression artifacts on solid colors).
    // Use PID in filename to avoid race conditions with concurrent exports.
    let temp_path = std::env::temp_dir().join(format!("md-pdf-code-bg-{}.png", std::process::id()));
    if img.save(&temp_path).is_err() {
        return false;
    }

    let ok = if let Ok(mut bg_element) = elements::Image::from_path(&temp_path) {
        bg_element.set_dpi(dpi);
        let _ = bg_element.render(context, area, pdf_style);
        true
    } else {
        false
    };

    let _ = std::fs::remove_file(&temp_path);
    ok
}

/// Walk the AST to find the first H1 heading's text for use as document title.
fn extract_title<'a>(root: &'a AstNode<'a>) -> Option<String> {
    for node in root.descendants() {
        let data = node.data.borrow();
        if let NodeValue::Heading(h) = &data.value
            && h.level == 1
        {
            drop(data);
            let title = inline_text(node, CodeStyle::Bare);
            if !title.is_empty() {
                return Some(title);
            }
        }
    }
    None
}

// ─── EPUB Export via epub-builder ────────────────────────────────────────────

/// Split a document into chapters at top-level headings.
///
/// A single `content.xhtml` gives a reader no navigation at all, so Apple
/// Books, Kobo and Calibre show one undifferentiated blob no matter how many
/// headings the source had. Splits on `#`, falling back to `##` when the
/// document has fewer than two `#` headings (the common "one H1 title, H2
/// sections" shape). Fence-aware, so a `# comment` inside a code block is not
/// mistaken for a heading.
///
/// Returns `(title, markdown)` pairs. Content before the first heading becomes
/// its own leading chapter.
fn split_chapters(markdown: &str) -> Vec<(String, String)> {
    for depth in [1usize, 2] {
        let prefix = format!("{} ", "#".repeat(depth));
        let mut fence = crate::text::FenceTracker::new();
        let mut chapters: Vec<(String, String)> = Vec::new();
        let mut current = String::new();
        let mut current_title: Option<String> = None;

        for line in markdown.lines() {
            let in_fence = fence.feed(line);
            if !in_fence
                && let Some(rest) = line.strip_prefix(&prefix)
                && !rest.trim().is_empty()
            {
                if current_title.is_some() || !current.trim().is_empty() {
                    chapters.push((
                        current_title.take().unwrap_or_default(),
                        std::mem::take(&mut current),
                    ));
                }
                current_title = Some(rest.trim().to_string());
            }
            current.push_str(line);
            current.push('\n');
        }
        if current_title.is_some() || !current.trim().is_empty() {
            chapters.push((current_title.unwrap_or_default(), current));
        }

        // Only accept this depth if it actually produced navigation.
        if chapters.len() > 1 {
            return chapters;
        }
    }
    vec![(String::new(), markdown.to_string())]
}

/// Wrap an XHTML body fragment in the document skeleton every chapter needs.
fn wrap_xhtml(title: &str, body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css" />
</head>
<body>
{body}
</body>
</html>"#
    )
}

fn export_epub(
    markdown: &str,
    output_path: &str,
    source_file: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    use epub_builder::{EpubBuilder, EpubContent, EpubVersion, ZipLibrary};

    let fm = crate::frontmatter::parse(markdown);

    let title = fm
        .title
        .clone()
        .or_else(|| {
            let arena = typed_arena::Arena::new();
            let root = parse_markdown(&arena, markdown);
            extract_title(root)
        })
        .or_else(|| {
            source_file.map(|f| {
                std::path::Path::new(f)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
        })
        .unwrap_or_else(|| "Untitled".to_string());

    let html_fragment = crate::html::render_fragment(markdown, crate::options::syntax_theme());

    let base_dir = source_file
        .map(|f| {
            std::path::Path::new(f)
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf()
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let (processed_html, images) = process_images(&html_fragment, &base_dir);

    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
    builder.epub_version(EpubVersion::V30);
    builder.metadata("title", &title)?;
    // EPUB 3 requires dc:language; readers and validators reject a book without
    // it. Front matter can override the default.
    builder.metadata("lang", fm.lang.clone().unwrap_or_else(|| "en".to_string()))?;
    if let Some(ref author) = fm.author {
        builder.metadata("author", author.clone())?;
    }
    for tag in &fm.tags {
        builder.metadata("subject", tag)?;
    }

    builder.stylesheet(crate::html::assets::EPUB_CSS.as_bytes())?;

    for (epub_path, mime, bytes) in &images {
        builder.add_resource(epub_path, bytes.as_slice(), mime)?;
    }

    // One XHTML document per chapter, so readers get real navigation. The
    // images were already rewritten above, so re-render per chapter from the
    // markdown and reuse the shared resource pool.
    // Strip front matter first, or the YAML block becomes a phantom chapter 1.
    let chapters = split_chapters(crate::frontmatter::strip(markdown));
    if chapters.len() > 1 {
        for (i, (chapter_title, body)) in chapters.iter().enumerate() {
            let fragment = crate::html::render_fragment(body, crate::options::syntax_theme());
            let (fragment, _) = process_images(&fragment, &base_dir);
            let xhtml = wrap_xhtml(
                if chapter_title.is_empty() {
                    &title
                } else {
                    chapter_title
                },
                &html_to_xhtml(&fragment),
            );
            let href = format!("chapter_{:03}.xhtml", i + 1);
            let label = if chapter_title.is_empty() {
                title.clone()
            } else {
                chapter_title.clone()
            };
            builder.add_content(EpubContent::new(&href, xhtml.as_bytes()).title(label))?;
        }
    } else {
        let xhtml = wrap_xhtml(&title, &html_to_xhtml(&processed_html));
        builder.add_content(EpubContent::new("content.xhtml", xhtml.as_bytes()).title(&title))?;
    }

    let mut output_file = std::fs::File::create(output_path)?;
    builder.generate(&mut output_file)?;

    eprintln!("  Written to {}", output_path);
    Ok(())
}

fn process_images(
    html: &str,
    base_dir: &std::path::Path,
) -> (String, Vec<(String, String, Vec<u8>)>) {
    let mut images = Vec::new();
    let mut result = String::with_capacity(html.len());
    let mut remaining = html;

    while let Some(img_start) = remaining.find("<img") {
        result.push_str(&remaining[..img_start]);
        let after_img = &remaining[img_start..];

        if let Some(tag_end) = after_img.find('>') {
            let tag = &after_img[..=tag_end];

            if let Some(src) = extract_attr(tag, "src")
                && !src.starts_with("http://")
                && !src.starts_with("https://")
                && !src.starts_with("data:")
            {
                let file_path = base_dir.join(&src);
                if let Ok(bytes) = std::fs::read(&file_path) {
                    let filename = std::path::Path::new(&src)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let epub_path = format!("images/{}", filename);
                    let mime = mime_from_ext(&src);
                    let new_tag =
                        tag.replace(&format!("\"{}\"", src), &format!("\"{}\"", epub_path));
                    result.push_str(&new_tag);
                    images.push((epub_path, mime, bytes));
                    remaining = &remaining[img_start + tag_end + 1..];
                    continue;
                }
            }

            result.push_str(tag);
            remaining = &remaining[img_start + tag_end + 1..];
        } else {
            result.push_str(after_img);
            remaining = "";
        }
    }
    result.push_str(remaining);

    (result, images)
}

fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr_name);
    if let Some(start) = tag.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = tag[value_start..].find('"') {
            return Some(tag[value_start..value_start + end].to_string());
        }
    }
    None
}

fn mime_from_ext(path: &str) -> String {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn html_to_xhtml(html: &str) -> String {
    html.replace("<br>", "<br />").replace("<hr>", "<hr />")
}

// ─── PDF Export via genpdfi ───────────────────────────────────────────────────

pub fn export_pdf(
    markdown: &str,
    output_path: &str,
    allow_remote_render: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::atomic::Ordering;

    // Park the opt-in for the duration of this export; see ALLOW_REMOTE_RENDER.
    ALLOW_REMOTE_RENDER.store(allow_remote_render, Ordering::Relaxed);
    MERMAID_WARNED.store(false, Ordering::Relaxed);

    #[cfg(not(feature = "url"))]
    if allow_remote_render {
        eprintln!("  --allow-remote-render has no effect in this build (no remote support)");
    }

    let arena = typed_arena::Arena::new();
    let root = parse_markdown(&arena, markdown);

    let font = load_pdf_font(SANS_CANDIDATES, "Helvetica")?;
    let mut doc = genpdfi::Document::new(font);

    // Monospace family for code blocks.
    let courier = load_pdf_font(MONO_CANDIDATES, "Courier")?;
    let courier_ref = doc.add_font_family(courier);

    let title = extract_title(root).unwrap_or_else(|| "document".to_string());
    doc.set_title(&title);
    doc.set_font_size(11);
    doc.set_line_spacing(1.25);
    doc.set_paper_size(genpdfi::PaperSize::A4);
    doc.set_minimal_conformance();

    let mut decorator = genpdfi::SimplePageDecorator::new();
    decorator.set_margins(genpdfi::Margins::trbl(20, 15, 20, 15));
    decorator.set_header(|page| {
        let mut p = elements::Paragraph::default();
        p.set_alignment(genpdfi::Alignment::Right);
        p.push_styled(
            format!("Page {}", page),
            style::Style::new()
                .with_font_size(9)
                .with_color(style::Color::Rgb(150, 150, 155)),
        );
        elements::PaddedElement::new(p, genpdfi::Margins::trbl(0, 0, 3, 0))
    });
    doc.set_page_decorator(decorator);

    // Track temp files from mermaid rendering
    let mut temp_files: Vec<PathBuf> = Vec::new();
    let mut first_h1_seen = false;

    // Walk AST and push PDF elements
    render_blocks(
        &mut doc,
        root,
        &mut temp_files,
        courier_ref,
        &mut first_h1_seen,
    );

    // Render footnote definitions at the end of the document
    render_footnotes(&mut doc, root, courier_ref);

    // Render to file
    doc.render_to_file(output_path)
        .map_err(|e| format!("PDF write error: {}", e))?;

    // Clean up temp files
    for path in &temp_files {
        let _ = std::fs::remove_file(path);
    }
    // remove_dir only ever succeeded on an empty directory, so a failed mmdc run
    // (which leaves mmdc_output.png behind) leaked the directory forever. The
    // directory is now this process's alone, so removing it recursively is safe.
    let _ = std::fs::remove_dir_all(mermaid_temp_dir());

    eprintln!("  Wrote {}", output_path);
    Ok(())
}

fn render_blocks<'a>(
    doc: &mut genpdfi::Document,
    node: &'a AstNode<'a>,
    temp_files: &mut Vec<PathBuf>,
    mono_font: genpdfi::fonts::FontFamily<genpdfi::fonts::Font>,
    first_h1_seen: &mut bool,
) {
    for child in node.children() {
        render_block(doc, child, temp_files, mono_font, first_h1_seen);
    }
}

fn render_block<'a>(
    doc: &mut genpdfi::Document,
    node: &'a AstNode<'a>,
    temp_files: &mut Vec<PathBuf>,
    mono_font: genpdfi::fonts::FontFamily<genpdfi::fonts::Font>,
    first_h1_seen: &mut bool,
) {
    let data = node.data.borrow();
    match &data.value {
        NodeValue::Document => {
            drop(data);
            render_blocks(doc, node, temp_files, mono_font, first_h1_seen);
        }
        NodeValue::Heading(h) => {
            let level = h.level as usize;
            let size = HEADING_SIZES
                .get(level.saturating_sub(1))
                .copied()
                .unwrap_or(11);
            drop(data);

            // Page break before H1 (except the very first one)
            if level == 1 {
                if *first_h1_seen {
                    doc.push(elements::PageBreak::new());
                }
                *first_h1_seen = true;
            }

            doc.push(elements::Break::new(1.5_f32));
            let size = fit_heading_size(doc, &inline_text(node, CodeStyle::Bare), size);
            let mut p = elements::Paragraph::default();
            let base = style::Style::new()
                .with_font_size(size)
                .bold()
                .with_color(style::Color::Rgb(25, 25, 35));
            collect_inline(&mut p, node, base, mono_font);
            doc.push(p);

            // Underline for H1 and H2
            if level <= 2 {
                let rule_char = if level == 1 { "\u{2501}" } else { "\u{2500}" };
                let rule_color = if level == 1 {
                    style::Color::Rgb(50, 50, 60)
                } else {
                    style::Color::Rgb(210, 212, 218)
                };
                let count = rule_repeat_count(doc, rule_char, RULE_FONT_SIZE);
                let mut rule = elements::Paragraph::default();
                rule.push_styled(
                    rule_char.repeat(count),
                    style::Style::new()
                        .with_font_size(RULE_FONT_SIZE)
                        .with_color(rule_color),
                );
                doc.push(rule);
            }
            doc.push(elements::Break::new(0.8_f32));
        }
        NodeValue::DescriptionTerm => {
            drop(data);
            let mut p = elements::Paragraph::default();
            let base = style::Style::new().with_font_size(11).bold();
            collect_inline(&mut p, node, base, mono_font);
            doc.push(p);
        }
        NodeValue::DescriptionDetails => {
            drop(data);
            for child in node.children() {
                render_block(doc, child, temp_files, mono_font, first_h1_seen);
            }
            doc.push(elements::Break::new(0.3_f32));
        }
        NodeValue::Paragraph => {
            drop(data);
            let mut p = elements::Paragraph::default();
            let base = style::Style::new()
                .with_font_size(11)
                .with_color(style::Color::Rgb(30, 30, 30));
            collect_inline(&mut p, node, base, mono_font);
            doc.push(p);
            doc.push(elements::Break::new(0.5_f32));

            // Embed any local images found in this paragraph
            embed_inline_images(doc, node);
        }
        NodeValue::CodeBlock(cb) => {
            let info = cb.info.clone();
            let literal = cb.literal.clone();
            drop(data);

            let is_mermaid = info == "mermaid";

            // Mermaid diagrams: render as an image when a renderer is available
            // (local mmdc, or kroki.io when --allow-remote-render was passed).
            if is_mermaid
                && let Some((img_element, path)) =
                    render_mermaid_to_image(&literal, temp_files.len())
            {
                doc.push(elements::Break::new(0.5_f32));
                doc.push(img_element);
                doc.push(elements::Break::new(0.5_f32));
                temp_files.push(path);
                return;
            }

            // No renderer available: fall through to a labelled source block so the
            // export still succeeds, and say once how to get a real diagram.
            if is_mermaid {
                warn_mermaid_not_rendered();
            }

            // Regular code block: monospace font with soft background
            doc.push(elements::Break::new(0.5_f32));

            // Language label above the code block
            let label = if is_mermaid {
                Some("mermaid (diagram not rendered - showing source)".to_string())
            } else if info.is_empty() {
                None
            } else {
                Some(info.split_whitespace().next().unwrap_or(&info).to_string())
            };
            if let Some(label) = label {
                let mut lang_p = elements::Paragraph::default();
                lang_p.push_styled(
                    format!("  {}", label),
                    style::Style::new()
                        .with_font_size(8)
                        .with_font_family(mono_font)
                        .with_color(style::Color::Rgb(130, 130, 140)),
                );
                doc.push(lang_p);
                doc.push(elements::Break::new(0.15_f32));
            }

            let code_style = style::Style::new()
                .with_font_size(9)
                .with_font_family(mono_font)
                .with_color(style::Color::Rgb(40, 42, 54));

            // Courier at 9pt ≈ 2.4mm/char → ~71 chars fit in content width minus padding
            let max_chars = 71;
            let mut layout = elements::LinearLayout::vertical();
            for line in literal.lines() {
                for display in wrap_code_line(line, max_chars) {
                    let mut p = elements::Paragraph::default();
                    p.push_styled(display, code_style);
                    layout.push(p);
                }
            }

            // Soft gray background behind code content
            doc.push(FilledElement::new(
                layout,
                style::Color::Rgb(243, 244, 248),
                3, // vertical padding (mm)
                4, // horizontal padding (mm)
            ));
            doc.push(elements::Break::new(0.5_f32));
        }
        NodeValue::List(list) => {
            let lt = list.list_type;
            let start = list.start;
            drop(data);
            render_list(doc, node, lt, start, temp_files, mono_font, first_h1_seen);
            doc.push(elements::Break::new(0.3_f32));
        }
        NodeValue::Item(_) | NodeValue::TaskItem(_) => {
            // Handled by render_list
            drop(data);
        }
        NodeValue::BlockQuote => {
            drop(data);
            doc.push(elements::Break::new(0.3_f32));
            let bar_color = style::Color::Rgb(180, 185, 195);
            for child in node.children() {
                let cd = child.data.borrow();
                if matches!(&cd.value, NodeValue::Paragraph) {
                    drop(cd);
                    let mut p = elements::Paragraph::default();
                    let qs = style::Style::new()
                        .with_font_size(11)
                        .italic()
                        .with_color(style::Color::Rgb(80, 80, 95));
                    p.push_styled(
                        "  \u{2503} ",
                        style::Style::new().with_color(bar_color).bold(),
                    );
                    collect_inline(&mut p, child, qs, mono_font);
                    doc.push(p);
                    doc.push(elements::Break::new(0.2_f32));
                } else {
                    drop(cd);
                    render_block(doc, child, temp_files, mono_font, first_h1_seen);
                }
            }
            doc.push(elements::Break::new(0.3_f32));
        }
        NodeValue::Table(_) => {
            drop(data);
            render_table(doc, node, mono_font);
        }
        NodeValue::ThematicBreak => {
            drop(data);
            doc.push(elements::Break::new(0.5_f32));
            let mut p = elements::Paragraph::default();
            p.set_alignment(genpdfi::Alignment::Center);
            p.push_styled(
                "\u{2500}".repeat(60),
                style::Style::new().with_color(style::Color::Rgb(200, 200, 205)),
            );
            doc.push(p);
            doc.push(elements::Break::new(0.5_f32));
        }
        NodeValue::FrontMatter(_)
        | NodeValue::HtmlBlock(_)
        | NodeValue::HtmlInline(_)
        | NodeValue::FootnoteDefinition(_) => {
            drop(data);
        }
        NodeValue::Alert(alert) => {
            let alert_type = alert.alert_type;
            drop(data);
            render_alert_block(doc, node, alert_type, temp_files, mono_font, first_h1_seen);
        }
        _ => {
            drop(data);
            render_blocks(doc, node, temp_files, mono_font, first_h1_seen);
        }
    }
}

fn collect_inline<'a>(
    p: &mut elements::Paragraph,
    node: &'a AstNode<'a>,
    base: style::Style,
    mono_font: genpdfi::fonts::FontFamily<genpdfi::fonts::Font>,
) {
    for child in node.children() {
        let data = child.data.borrow();
        match &data.value {
            NodeValue::Text(t) => {
                let t = t.clone();
                drop(data);
                p.push_styled(t, base);
            }
            NodeValue::Code(c) => {
                let literal = c.literal.clone();
                drop(data);
                let cs = style::Style::new()
                    .with_font_size(10)
                    .with_font_family(mono_font)
                    .with_color(style::Color::Rgb(200, 55, 90));
                p.push_styled(format!("`{}`", literal), cs);
            }
            NodeValue::Emph => {
                drop(data);
                collect_inline(p, child, base.italic(), mono_font);
            }
            NodeValue::Strong => {
                drop(data);
                collect_inline(p, child, base.bold(), mono_font);
            }
            NodeValue::Strikethrough => {
                drop(data);
                collect_inline(
                    p,
                    child,
                    base.with_color(style::Color::Rgb(150, 150, 150)),
                    mono_font,
                );
            }
            NodeValue::Link(link) => {
                let url = link.url.clone();
                drop(data);
                let link_style = base.with_color(style::Color::Rgb(0, 95, 204)).underline();
                let text = inline_text(child, CodeStyle::Bare);
                p.push_link(text, url, link_style);
            }
            NodeValue::Image(img) => {
                let title = img.title.clone();
                drop(data);
                // Prefer alt text from children, fall back to title attribute
                let text = inline_text(child, CodeStyle::Bare);
                let text = if text.is_empty() { title } else { text };
                if !text.is_empty() {
                    p.push_styled(format!("[{}]", text), base.italic());
                }
            }
            NodeValue::SoftBreak => {
                drop(data);
                p.push_styled(" ", base);
            }
            NodeValue::LineBreak => {
                drop(data);
                p.push_styled("\n", base);
            }
            NodeValue::Math(m) => {
                let literal = m.literal.clone();
                drop(data);
                p.push_styled(literal, base.italic());
            }
            NodeValue::FootnoteReference(r) => {
                let name = r.name.clone();
                drop(data);
                p.push_styled(
                    format!("[{}]", name),
                    style::Style::new()
                        .with_font_size(9)
                        .with_color(style::Color::Rgb(100, 100, 110)),
                );
            }
            _ => {
                drop(data);
                collect_inline(p, child, base, mono_font);
            }
        }
    }
}

/// Scan a paragraph node tree for Image nodes (including nested in links/emphasis)
/// and embed local files as block-level images.
/// URLs and missing files fall back to the [alt text] already rendered by collect_inline.
#[cfg(feature = "images")]
fn embed_inline_images<'a>(doc: &mut genpdfi::Document, node: &'a AstNode<'a>) {
    for descendant in node.descendants() {
        let data = descendant.data.borrow();
        if let NodeValue::Image(img) = &data.value {
            let url = img.url.clone();
            drop(data);

            // Skip remote URLs and data URIs — only embed local files
            if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("data:")
            {
                continue;
            }

            let path = std::path::Path::new(&url);
            if !path.exists() {
                continue;
            }

            if let Ok(img_element) = elements::Image::from_path(path) {
                let img_element = scale_image_to_fit(img_element, path);
                doc.push(img_element.with_alignment(genpdfi::Alignment::Center));
                doc.push(elements::Break::new(0.5_f32));
            }
        } else {
            drop(data);
        }
    }
}

#[cfg(not(feature = "images"))]
fn embed_inline_images<'a>(_doc: &mut genpdfi::Document, _node: &'a AstNode<'a>) {}

fn render_list<'a>(
    doc: &mut genpdfi::Document,
    node: &'a AstNode<'a>,
    list_type: ListType,
    start: usize,
    temp_files: &mut Vec<PathBuf>,
    mono_font: genpdfi::fonts::FontFamily<genpdfi::fonts::Font>,
    first_h1_seen: &mut bool,
) {
    let body_style = style::Style::new()
        .with_font_size(11)
        .with_color(style::Color::Rgb(30, 30, 30));

    for (i, item) in node.children().enumerate() {
        // Check if this is a task list item
        let task_checked = {
            let item_data = item.data.borrow();
            if let NodeValue::TaskItem(task) = &item_data.value {
                Some(task.symbol.is_some())
            } else {
                None
            }
        };

        let bullet = if let Some(checked) = task_checked {
            if checked {
                "\u{2611}  ".to_string() // ☑
            } else {
                "\u{2610}  ".to_string() // ☐
            }
        } else {
            match list_type {
                ListType::Bullet => "\u{2022}  ".to_string(),
                ListType::Ordered => format!("{}.  ", start + i),
            }
        };

        let mut first_para = true;
        for item_child in item.children() {
            let cd = item_child.data.borrow();
            match &cd.value {
                NodeValue::Paragraph => {
                    drop(cd);
                    let mut p = elements::Paragraph::default();
                    if first_para {
                        p.push_styled(&bullet, body_style.bold());
                        first_para = false;
                    } else {
                        // Continuation paragraph — indent to align
                        p.push_styled("    ", body_style);
                    }
                    collect_inline(&mut p, item_child, body_style, mono_font);
                    doc.push(p);
                    doc.push(elements::Break::new(0.15_f32));
                }
                NodeValue::List(sub_list) => {
                    let lt = sub_list.list_type;
                    let st = sub_list.start;
                    drop(cd);
                    // Nested list with indentation
                    doc.push(elements::PaddedElement::new(
                        elements::Break::new(0.0_f32),
                        genpdfi::Margins::trbl(0, 0, 0, 6),
                    ));
                    render_list(
                        doc,
                        item_child,
                        lt,
                        st,
                        temp_files,
                        mono_font,
                        first_h1_seen,
                    );
                }
                _ => {
                    drop(cd);
                    render_block(doc, item_child, temp_files, mono_font, first_h1_seen);
                }
            }
        }
    }
}

/// Hard-wrap an over-long code line instead of truncating it.
///
/// genpdfi cannot break a string with no spaces, and the previous behaviour
/// silently discarded everything past the cut.
fn wrap_code_line(line: &str, max_chars: usize) -> Vec<String> {
    if max_chars == 0 || line.chars().count() <= max_chars {
        return vec![line.to_string()];
    }
    let mut out = Vec::new();
    let mut current = String::new();
    for c in line.chars() {
        if current.chars().count() == max_chars {
            out.push(std::mem::take(&mut current));
        }
        current.push(c);
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

fn render_table<'a>(
    doc: &mut genpdfi::Document,
    node: &'a AstNode<'a>,
    mono_font: genpdfi::fonts::FontFamily<genpdfi::fonts::Font>,
) {
    // Widest row, not the first: a row with more cells than the header would
    // otherwise make push() fail and silently drop the whole row.
    let num_cols = node
        .children()
        .filter(|r| matches!(&r.data.borrow().value, NodeValue::TableRow(_)))
        .map(|r| r.children().count())
        .max()
        .unwrap_or(0);
    if num_cols == 0 {
        return;
    }

    let alignments: Vec<genpdfi::Alignment> = match &node.data.borrow().value {
        NodeValue::Table(t) => t
            .alignments
            .iter()
            .map(|a| match a {
                comrak::nodes::TableAlignment::Right => genpdfi::Alignment::Right,
                comrak::nodes::TableAlignment::Center => genpdfi::Alignment::Center,
                _ => genpdfi::Alignment::Left,
            })
            .collect(),
        _ => Vec::new(),
    };

    let column_weights = vec![1; num_cols];
    let mut table = elements::TableLayout::new(column_weights);
    table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));

    let mut is_header = true;
    for row_node in node.children() {
        let rd = row_node.data.borrow();
        if !matches!(&rd.value, NodeValue::TableRow(_)) {
            drop(rd);
            continue;
        }
        drop(rd);

        let cell_style = if is_header {
            style::Style::new()
                .with_font_size(10)
                .bold()
                .with_color(style::Color::Rgb(25, 25, 35))
        } else {
            style::Style::new()
                .with_font_size(10)
                .with_color(style::Color::Rgb(30, 30, 30))
        };

        let mut row = table.row();
        let mut cells = 0usize;
        for (col, cell_node) in row_node.children().enumerate() {
            let mut p = elements::Paragraph::default();
            if let Some(a) = alignments.get(col) {
                p.set_alignment(*a);
            }
            collect_inline(&mut p, cell_node, cell_style, mono_font);
            cells += 1;
            if is_header {
                row.push_element(
                    FilledElement::new(
                        elements::PaddedElement::new(p, genpdfi::Margins::trbl(1, 1, 1, 1)),
                        style::Color::Rgb(240, 241, 245),
                        1,
                        1,
                    )
                    .with_corner_radius(0.0),
                );
            } else {
                row.push_element(elements::PaddedElement::new(
                    p,
                    genpdfi::Margins::trbl(1, 1, 1, 1),
                ));
            }
        }
        // Short rows must be padded or push() rejects the whole row.
        for _ in cells..num_cols {
            row.push_element(elements::Paragraph::default());
        }
        if row.push().is_err() {
            eprintln!("  Warning: skipped a table row the PDF layout could not fit");
        }
        is_header = false;
    }

    doc.push(elements::Break::new(0.3_f32));
    doc.push(table);
    doc.push(elements::Break::new(0.5_f32));
}

// ─── Alert Block Rendering ───────────────────────────────────────────────────

fn render_alert_block<'a>(
    doc: &mut genpdfi::Document,
    node: &'a AstNode<'a>,
    alert_type: AlertType,
    temp_files: &mut Vec<PathBuf>,
    mono_font: genpdfi::fonts::FontFamily<genpdfi::fonts::Font>,
    first_h1_seen: &mut bool,
) {
    let (label, color) = match alert_type {
        AlertType::Note => ("Note", style::Color::Rgb(9, 105, 218)),
        AlertType::Tip => ("Tip", style::Color::Rgb(26, 127, 55)),
        AlertType::Important => ("Important", style::Color::Rgb(130, 80, 223)),
        AlertType::Warning => ("Warning", style::Color::Rgb(191, 135, 0)),
        AlertType::Caution => ("Caution", style::Color::Rgb(207, 34, 46)),
    };

    doc.push(elements::Break::new(0.3_f32));

    // Bold colored label
    let mut label_p = elements::Paragraph::default();
    label_p.push_styled(
        format!("  \u{2502} {}", label),
        style::Style::new()
            .with_font_size(11)
            .bold()
            .with_color(color),
    );
    doc.push(label_p);
    doc.push(elements::Break::new(0.2_f32));

    // Render children with blockquote-style prefix
    for child in node.children() {
        let cd = child.data.borrow();
        if matches!(&cd.value, NodeValue::Paragraph) {
            drop(cd);
            let mut p = elements::Paragraph::default();
            let qs = style::Style::new()
                .with_font_size(11)
                .with_color(style::Color::Rgb(55, 55, 65));
            p.push_styled("  \u{2502} ", style::Style::new().with_color(color));
            collect_inline(&mut p, child, qs, mono_font);
            doc.push(p);
            doc.push(elements::Break::new(0.2_f32));
        } else {
            drop(cd);
            render_block(doc, child, temp_files, mono_font, first_h1_seen);
        }
    }

    doc.push(elements::Break::new(0.3_f32));
}

// ─── Footnote Rendering ─────────────────────────────────────────────────────

fn render_footnotes<'a>(
    doc: &mut genpdfi::Document,
    root: &'a AstNode<'a>,
    mono_font: genpdfi::fonts::FontFamily<genpdfi::fonts::Font>,
) {
    let _ = mono_font; // available if needed in future
    let mut footnotes: Vec<(String, String)> = Vec::new();

    for node in root.descendants() {
        let data = node.data.borrow();
        if let NodeValue::FootnoteDefinition(fd) = &data.value {
            let name = fd.name.clone();
            drop(data);
            let text = inline_text(node, CodeStyle::Bare);
            footnotes.push((name, text));
        }
    }

    if footnotes.is_empty() {
        return;
    }

    // Separator
    doc.push(elements::Break::new(1.5_f32));
    let mut sep = elements::Paragraph::default();
    sep.push_styled(
        "\u{2500}".repeat(40),
        style::Style::new().with_color(style::Color::Rgb(200, 200, 205)),
    );
    doc.push(sep);
    doc.push(elements::Break::new(0.5_f32));

    // Footnote entries
    let fn_style = style::Style::new()
        .with_font_size(9)
        .with_color(style::Color::Rgb(80, 80, 90));

    for (name, text) in &footnotes {
        let mut p = elements::Paragraph::default();
        p.push_styled(format!("[{}] ", name), fn_style.bold());
        p.push_styled(text, fn_style);
        doc.push(p);
        doc.push(elements::Break::new(0.2_f32));
    }
}

// ─── Mermaid Diagram Rendering ───────────────────────────────────────────────

/// Whether the user opted in to remote diagram rendering for this export.
///
/// Set once by `export_pdf` before the AST walk. The mermaid renderer is reached
/// through the recursive `render_block`/`render_list` walk, and `render_list`
/// already takes 7 arguments, so threading a flag down would trip
/// `clippy::too_many_arguments`. Defaults to false: diagram source never leaves
/// the machine unless `--allow-remote-render` was passed.
static ALLOW_REMOTE_RENDER: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Set on the first unrendered diagram so the advice is printed once per export,
/// however many mermaid blocks the document contains.
static MERMAID_WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Temp directory for mermaid intermediates, scoped to this process.
///
/// Concurrent exports previously shared one directory and fixed file names, so
/// they overwrote each other's input and output. Same PID reasoning as the
/// code-block background above; owning the whole directory also makes the
/// end-of-export cleanup safe to do recursively.
fn mermaid_temp_dir() -> PathBuf {
    std::env::temp_dir().join(format!("md-mermaid-export-{}", std::process::id()))
}

/// Print, at most once per export, why a mermaid diagram was left as source.
fn warn_mermaid_not_rendered() {
    use std::sync::atomic::Ordering;
    if MERMAID_WARNED.swap(true, Ordering::Relaxed) {
        return;
    }

    #[cfg(feature = "url")]
    if ALLOW_REMOTE_RENDER.load(Ordering::Relaxed) {
        eprintln!(
            "  Mermaid diagram not rendered (source shown instead) - no usable image \
             came back from mmdc or from kroki.io"
        );
        return;
    }

    #[cfg(feature = "url")]
    eprintln!(
        "  Mermaid diagram not rendered (source shown instead) - install mmdc to \
         render locally, or pass --allow-remote-render to upload the source to kroki.io"
    );

    #[cfg(not(feature = "url"))]
    eprintln!(
        "  Mermaid diagram not rendered (source shown instead) - install mmdc \
         (npm install -g @mermaid-js/mermaid-cli) to render it locally"
    );
}

/// Render a mermaid code block as a PNG image for PDF embedding.
/// Returns the genpdfi Image element and the temp file path for cleanup.
fn render_mermaid_to_image(code: &str, index: usize) -> Option<(elements::Image, PathBuf)> {
    let png_bytes = render_mermaid_png(code)?;

    let temp_dir = mermaid_temp_dir();
    let _ = std::fs::create_dir_all(&temp_dir);

    // Convert PNG (RGBA) to JPEG (RGB) — genpdfi doesn't support alpha channel
    let jpg_path = temp_dir.join(format!("diagram_{}.jpg", index));
    if convert_png_to_jpeg(&png_bytes, &jpg_path).is_err() {
        return None;
    }

    elements::Image::from_path(&jpg_path).ok().map(|img| {
        let img = scale_image_to_fit(img, &jpg_path);
        (img.with_alignment(genpdfi::Alignment::Center), jpg_path)
    })
}

/// Scale a genpdfi Image element to fit within page content margins.
/// Uses pixel dimensions from the file, converts to mm at 300 DPI,
/// then applies uniform downscale if needed (never upscales).
#[cfg(feature = "images")]
fn scale_image_to_fit(img: elements::Image, path: &std::path::Path) -> elements::Image {
    if let Ok((px_w, px_h)) = image::image_dimensions(path) {
        if px_w == 0 || px_h == 0 {
            return img;
        }
        let w_mm = 25.4 * px_w as f32 / 300.0;
        let h_mm = 25.4 * px_h as f32 / 300.0;
        let scale = (1.0_f32)
            .min(CONTENT_WIDTH_MM / w_mm)
            .min(MAX_IMAGE_HEIGHT_MM / h_mm);
        if scale < 1.0 {
            return img.with_scale(Scale::new(scale, scale));
        }
    }
    img
}

#[cfg(not(feature = "images"))]
fn scale_image_to_fit(img: elements::Image, _path: &std::path::Path) -> elements::Image {
    img
}

/// Convert PNG bytes (may have alpha) to JPEG file (no alpha).
/// Composites transparent pixels onto a white background.
#[cfg(feature = "images")]
fn convert_png_to_jpeg(
    png_bytes: &[u8],
    output: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::load_from_memory(png_bytes)?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut rgb = image::RgbImage::new(w, h);
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let a = pixel[3] as f32 / 255.0;
        // Alpha composite onto white background
        let r = (pixel[0] as f32 * a + 255.0 * (1.0 - a)) as u8;
        let g = (pixel[1] as f32 * a + 255.0 * (1.0 - a)) as u8;
        let b = (pixel[2] as f32 * a + 255.0 * (1.0 - a)) as u8;
        rgb.put_pixel(x, y, image::Rgb([r, g, b]));
    }
    rgb.save(output)?;
    Ok(())
}

#[cfg(not(feature = "images"))]
fn convert_png_to_jpeg(
    _png_bytes: &[u8],
    _output: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    Err("Image conversion requires the 'images' feature".into())
}

/// Try to render mermaid to PNG. Uses the local mmdc CLI; falls back to the
/// kroki.io web API only when the user passed `--allow-remote-render`, because
/// that fallback sends the diagram source to a third-party server.
fn render_mermaid_png(code: &str) -> Option<Vec<u8>> {
    if let Some(data) = render_mermaid_mmdc(code) {
        return Some(data);
    }

    #[cfg(feature = "url")]
    if ALLOW_REMOTE_RENDER.load(std::sync::atomic::Ordering::Relaxed)
        && let Some(data) = render_mermaid_kroki(code)
    {
        return Some(data);
    }

    None
}

/// Render mermaid to PNG using the mmdc CLI (mermaid-cli, works offline).
fn render_mermaid_mmdc(code: &str) -> Option<Vec<u8>> {
    let temp_dir = mermaid_temp_dir();
    let _ = std::fs::create_dir_all(&temp_dir);
    let input_path = temp_dir.join("mmdc_input.mmd");
    let output_path = temp_dir.join("mmdc_output.png");

    // Prepend neutral theme for clean, soft diagram style
    let themed_code = format!("%%{{init: {{\"theme\": \"neutral\"}}}}%%\n{}", code);
    std::fs::write(&input_path, themed_code).ok()?;

    let status = std::process::Command::new("mmdc")
        .args([
            "-i",
            &input_path.to_string_lossy(),
            "-o",
            &output_path.to_string_lossy(),
            "-b",
            "white",
            "-s",
            "2",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()?;

    let _ = std::fs::remove_file(&input_path);

    if status.success() {
        let data = std::fs::read(&output_path).ok()?;
        let _ = std::fs::remove_file(&output_path);
        Some(data)
    } else {
        None
    }
}

/// Render mermaid to PNG using the kroki.io web API (no browser needed).
///
/// Uploads the diagram source. Only ever called with `--allow-remote-render`.
#[cfg(feature = "url")]
fn render_mermaid_kroki(code: &str) -> Option<Vec<u8>> {
    const KROKI_ENDPOINT: &str = "https://kroki.io/mermaid/png";
    // Use neutral theme for clean, soft diagram style
    let themed_code = format!("%%{{init: {{\"theme\": \"neutral\"}}}}%%\n{}", code);
    eprintln!(
        "  Uploading mermaid source ({} bytes) to {}",
        themed_code.len(),
        KROKI_ENDPOINT
    );
    let resp = ureq::post(KROKI_ENDPOINT)
        .header("Content-Type", "text/plain")
        .send(&themed_code)
        .ok()?;

    let body = resp.into_body().read_to_vec().ok()?;

    if body.len() > 100 { Some(body) } else { None }
}

// ─── Other export formats ────────────────────────────────────────────────────

fn read_input(file: &Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    match file {
        Some(path) => Ok(std::fs::read_to_string(path)
            .map_err(|e| format!("Error reading '{}': {}", path, e))?),
        None => {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

fn ast_to_json<'a>(node: &'a AstNode<'a>, depth: usize) -> String {
    let data = node.data.borrow();
    let indent = "  ".repeat(depth);
    let indent1 = "  ".repeat(depth + 1);

    let node_type = match &data.value {
        NodeValue::Document => "document",
        NodeValue::Heading(_) => "heading",
        NodeValue::Paragraph => "paragraph",
        NodeValue::Text(_) => "text",
        NodeValue::Code(_) => "code",
        NodeValue::CodeBlock(_) => "code_block",
        NodeValue::Link(_) => "link",
        NodeValue::Image(_) => "image",
        NodeValue::List(_) => "list",
        NodeValue::Item(_) => "item",
        NodeValue::BlockQuote => "blockquote",
        NodeValue::ThematicBreak => "thematic_break",
        NodeValue::Table(_) => "table",
        NodeValue::TableRow(_) => "table_row",
        NodeValue::TableCell => "table_cell",
        NodeValue::Emph => "emphasis",
        NodeValue::Strong => "strong",
        NodeValue::Strikethrough => "strikethrough",
        NodeValue::SoftBreak => "softbreak",
        NodeValue::LineBreak => "linebreak",
        NodeValue::HtmlBlock(_) => "html_block",
        NodeValue::HtmlInline(_) => "html_inline",
        NodeValue::FrontMatter(_) => "front_matter",
        NodeValue::FootnoteDefinition(_) => "footnote_definition",
        NodeValue::FootnoteReference(_) => "footnote_reference",
        NodeValue::Math(_) => "math",
        NodeValue::TaskItem(_) => "task_item",
        NodeValue::Highlight => "highlight",
        NodeValue::Superscript => "superscript",
        NodeValue::WikiLink(_) => "wikilink",
        NodeValue::DescriptionList => "description_list",
        NodeValue::DescriptionItem(_) => "description_item",
        NodeValue::DescriptionTerm => "description_term",
        NodeValue::DescriptionDetails => "description_details",
        NodeValue::Alert(_) => "alert",
        _ => "other",
    };

    let mut props = Vec::new();

    match &data.value {
        NodeValue::Text(t) => props.push(format!("{}\"value\": {}", indent1, json_escape(t))),
        NodeValue::Code(c) => {
            props.push(format!("{}\"value\": {}", indent1, json_escape(&c.literal)))
        }
        NodeValue::Math(m) => {
            props.push(format!(
                "{}\"literal\": {}",
                indent1,
                json_escape(&m.literal)
            ));
            props.push(format!("{}\"display\": {}", indent1, m.display_math));
            props.push(format!("{}\"dollar\": {}", indent1, m.dollar_math));
        }
        NodeValue::HtmlBlock(hb) => {
            props.push(format!(
                "{}\"literal\": {}",
                indent1,
                json_escape(&hb.literal)
            ));
        }
        NodeValue::HtmlInline(h) => {
            props.push(format!("{}\"literal\": {}", indent1, json_escape(h)));
        }
        NodeValue::Alert(a) => {
            props.push(format!(
                "{}\"alert_type\": {}",
                indent1,
                json_escape(&format!("{:?}", a.alert_type))
            ));
        }
        NodeValue::FootnoteDefinition(f) => {
            props.push(format!("{}\"name\": {}", indent1, json_escape(&f.name)));
        }
        NodeValue::FootnoteReference(f) => {
            props.push(format!("{}\"name\": {}", indent1, json_escape(&f.name)));
        }
        NodeValue::TaskItem(t) => {
            props.push(format!("{}\"checked\": {}", indent1, t.symbol.is_some()));
        }
        NodeValue::WikiLink(link) => {
            props.push(format!("{}\"url\": {}", indent1, json_escape(&link.url)));
        }
        NodeValue::CodeBlock(cb) => {
            props.push(format!("{}\"info\": {}", indent1, json_escape(&cb.info)));
            props.push(format!(
                "{}\"literal\": {}",
                indent1,
                json_escape(&cb.literal)
            ));
        }
        NodeValue::Heading(h) => {
            props.push(format!("{}\"level\": {}", indent1, h.level));
        }
        NodeValue::Link(link) => {
            props.push(format!("{}\"url\": {}", indent1, json_escape(&link.url)));
            props.push(format!(
                "{}\"title\": {}",
                indent1,
                json_escape(&link.title)
            ));
        }
        NodeValue::Image(img) => {
            props.push(format!("{}\"url\": {}", indent1, json_escape(&img.url)));
            props.push(format!("{}\"title\": {}", indent1, json_escape(&img.title)));
        }
        NodeValue::FrontMatter(fm) => {
            props.push(format!("{}\"value\": {}", indent1, json_escape(fm)));
        }
        _ => {}
    }

    let children: Vec<String> = node
        .children()
        .map(|child| ast_to_json(child, depth + 2))
        .collect();

    let mut parts = vec![format!("{}\"type\": \"{}\"", indent1, node_type)];
    parts.extend(props);

    if !children.is_empty() {
        parts.push(format!(
            "{}\"children\": [\n{}\n{}]",
            indent1,
            children.join(",\n"),
            indent1
        ));
    }

    format!("{}{{\n{}\n{}}}", indent, parts.join(",\n"), indent)
}

fn json_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('"');
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c < '\x20' => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

fn extract_plain_text<'a>(root: &'a AstNode<'a>) -> String {
    use comrak::arena_tree::NodeEdge;

    let mut text = String::new();
    let mut last_was_block = false;
    let mut first_cell_in_row = true;

    for edge in root.traverse() {
        let node = match edge {
            NodeEdge::Start(n) => n,
            NodeEdge::End(n) => {
                match &n.data.borrow().value {
                    // A link's target is only useful after its label.
                    NodeValue::Link(link) if !link.url.is_empty() => {
                        // An autolink's label already is the URL.
                        let label_is_url = text.ends_with(link.url.as_str())
                            || link
                                .url
                                .strip_prefix("mailto:")
                                .is_some_and(|a| text.ends_with(a));
                        if !label_is_url {
                            text.push_str(" (");
                            text.push_str(&link.url);
                            text.push(')');
                        }
                        last_was_block = false;
                    }
                    NodeValue::TableRow(_) => {
                        text.push('\n');
                        last_was_block = true;
                    }
                    NodeValue::Superscript => {
                        text.push('^');
                        last_was_block = false;
                    }
                    NodeValue::Highlight => {
                        text.push_str("==");
                        last_was_block = false;
                    }
                    NodeValue::WikiLink(link) if !link.url.is_empty() => {
                        text.push_str(" (");
                        text.push_str(&link.url);
                        text.push(')');
                        last_was_block = false;
                    }
                    _ => {}
                }
                continue;
            }
        };
        let data = node.data.borrow();
        match &data.value {
            NodeValue::Text(t) => {
                text.push_str(t);
                last_was_block = false;
            }
            NodeValue::Code(c) => {
                text.push_str(&c.literal);
                last_was_block = false;
            }
            NodeValue::CodeBlock(cb) => {
                if last_was_block {
                    text.push('\n');
                }
                text.push_str(&cb.literal);
                last_was_block = true;
            }
            NodeValue::TableRow(_) => {
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push('\n');
                }
                first_cell_in_row = true;
            }
            NodeValue::TableCell => {
                if !first_cell_in_row {
                    text.push_str(" | ");
                }
                first_cell_in_row = false;
                last_was_block = false;
            }
            NodeValue::Superscript => {
                text.push('^');
                last_was_block = false;
            }
            NodeValue::Highlight => {
                text.push_str("==");
                last_was_block = false;
            }
            NodeValue::Math(m) => {
                text.push_str(&m.literal);
                last_was_block = false;
            }
            NodeValue::HtmlInline(h) => {
                text.push_str(h);
                last_was_block = false;
            }
            NodeValue::HtmlBlock(hb) => {
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push_str("\n\n");
                }
                text.push_str(hb.literal.trim_end());
                last_was_block = true;
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                text.push('\n');
                last_was_block = false;
            }
            NodeValue::Paragraph => {
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push_str("\n\n");
                }
                last_was_block = true;
            }
            NodeValue::Heading(_) => {
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push_str("\n\n");
                }
                last_was_block = true;
            }
            _ => {}
        }
    }

    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{}\n", trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mermaid_temp_dir_is_process_scoped() {
        let dir = mermaid_temp_dir();
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        assert!(
            name.ends_with(&std::process::id().to_string()),
            "mermaid temp dir must carry the pid so concurrent exports cannot collide: {}",
            name
        );
        assert_ne!(
            dir,
            std::env::temp_dir().join("md-mermaid-export"),
            "the old shared directory must not be reused"
        );
    }

    #[test]
    fn remote_render_is_off_by_default() {
        use std::sync::atomic::Ordering;
        // Valid only while no unit test in this binary calls export_pdf, which
        // sets this process-global. Drop this test if one is ever added.
        assert!(
            !ALLOW_REMOTE_RENDER.load(Ordering::Relaxed),
            "diagram source must never be uploaded unless --allow-remote-render set it"
        );
    }
}

#[cfg(test)]
mod output_path_tests {
    use super::*;

    #[test]
    fn test_default_output_path_swaps_extension() {
        assert_eq!(default_output_path(Some("README.md"), "pdf"), "README.pdf");
        assert_eq!(
            default_output_path(Some("docs/a.markdown"), "epub"),
            "docs/a.epub"
        );
        assert_eq!(default_output_path(None, "pdf"), "output.pdf");
    }

    #[test]
    fn test_default_output_path_only_touches_the_last_extension() {
        // `f.replace(".md", ".pdf")` was global: this produced
        // "notes.pdf.backup.pdf".
        assert_eq!(
            default_output_path(Some("notes.md.backup.md"), "pdf"),
            "notes.md.backup.pdf"
        );
    }

    #[test]
    fn test_default_output_path_leaves_directories_alone() {
        // A directory whose name contains ".markdown" was rewritten too.
        assert_eq!(
            default_output_path(Some("v1.markdown/post.md"), "pdf"),
            "v1.markdown/post.pdf"
        );
    }

    #[test]
    fn test_default_output_path_handles_no_extension() {
        assert_eq!(default_output_path(Some("LICENSE"), "pdf"), "LICENSE.pdf");
    }
}

#[cfg(test)]
mod chapter_tests {
    use super::*;

    #[test]
    fn test_splits_on_h1() {
        let ch = split_chapters("# One\n\na\n\n# Two\n\nb\n");
        assert_eq!(ch.len(), 2);
        assert_eq!(ch[0].0, "One");
        assert_eq!(ch[1].0, "Two");
    }

    #[test]
    fn test_falls_back_to_h2_when_only_one_h1() {
        let ch = split_chapters("# Title\n\nintro\n\n## Alpha\n\na\n\n## Beta\n\nb\n");
        let titles: Vec<&str> = ch.iter().map(|c| c.0.as_str()).collect();
        // The leading chunk holds the H1 and the intro but has no H2 of its
        // own, so its title is empty; export_epub substitutes the document
        // title for it, which is what reaches the reader's table of contents.
        assert_eq!(titles, vec!["", "Alpha", "Beta"]);
        assert!(ch[0].1.contains("# Title"));
    }

    #[test]
    fn test_heading_inside_code_fence_is_not_a_chapter() {
        let ch = split_chapters("# One\n\n```sh\n# not a heading\n# nor this\n```\n\n# Two\n");
        assert_eq!(ch.len(), 2, "fenced # lines must not split: {:?}", ch);
    }

    #[test]
    fn test_no_headings_is_a_single_chapter() {
        let ch = split_chapters("Just prose.\n\nMore prose.\n");
        assert_eq!(ch.len(), 1);
        assert!(ch[0].0.is_empty(), "untitled chapter");
    }

    #[test]
    fn test_content_before_first_heading_becomes_its_own_chapter() {
        let ch = split_chapters("Preamble text.\n\n# One\n\na\n\n# Two\n\nb\n");
        assert_eq!(ch.len(), 3);
        assert!(ch[0].0.is_empty());
        assert!(ch[0].1.contains("Preamble"));
    }

    #[test]
    fn test_empty_heading_does_not_start_a_chapter() {
        let ch = split_chapters("# \n\ntext\n");
        assert_eq!(ch.len(), 1);
    }
}

#[cfg(test)]
mod code_wrap_tests {
    use super::*;

    #[test]
    fn test_short_line_untouched() {
        assert_eq!(wrap_code_line("abc", 10), vec!["abc"]);
    }

    #[test]
    fn test_exact_length_untouched() {
        assert_eq!(wrap_code_line("abcde", 5), vec!["abcde"]);
    }

    #[test]
    fn test_long_line_wraps_without_loss() {
        let line = "a".repeat(25);
        let wrapped = wrap_code_line(&line, 10);
        assert_eq!(wrapped.len(), 3);
        assert_eq!(wrapped.concat(), line, "no character may be dropped");
    }

    #[test]
    fn test_multibyte_counts_characters_not_bytes() {
        let line = "\u{65e5}".repeat(12);
        let wrapped = wrap_code_line(&line, 5);
        assert_eq!(wrapped.concat(), line);
        assert!(wrapped.iter().all(|s| s.chars().count() <= 5));
    }
}
