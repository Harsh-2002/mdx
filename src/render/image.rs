#[cfg(feature = "images")]
use std::io::Write;
#[cfg(feature = "images")]
use std::path::Path;

#[cfg(feature = "images")]
use crate::terminal::ImageProtocol;

#[cfg(feature = "images")]
use super::RenderContext;

/// Try to render an inline image using terminal image protocols.
/// Returns `true` if the image was rendered, `false` to fall back to text placeholder.
#[cfg(feature = "images")]
pub fn render_inline_image<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    url: &str,
) -> std::io::Result<bool> {
    // Skip remote URLs — only render local files
    if url.starts_with("http://") || url.starts_with("https://") {
        return Ok(false);
    }

    if ctx.term.image_protocol == ImageProtocol::None {
        return Ok(false);
    }

    // Resolve image path relative to base_dir
    let path = if let Some(ref base) = ctx.image_base_dir {
        base.join(url)
    } else {
        Path::new(url).to_path_buf()
    };

    if !path.exists() {
        return Ok(false);
    }

    // Load and resize image
    let img = match image::open(&path) {
        Ok(img) => img,
        Err(_) => return Ok(false),
    };

    // Resize to fit terminal width
    // Terminal cells are roughly 2:1 width:height ratio
    let max_width_px = (ctx.available_width() as u32) * 8; // rough pixel estimate
    let max_height_px = max_width_px / 2; // aspect ratio consideration

    let img = img.resize(
        max_width_px.min(800),
        max_height_px.min(600),
        image::imageops::FilterType::Lanczos3,
    );

    match ctx.term.image_protocol {
        ImageProtocol::ITerm2 => render_iterm2(w, ctx, &img),
        ImageProtocol::Kitty => render_kitty(w, ctx, &img),
        ImageProtocol::None => Ok(false),
    }
}

#[cfg(feature = "images")]
fn render_iterm2<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    img: &image::DynamicImage,
) -> std::io::Result<bool> {
    use base64::Engine;
    use std::io::Cursor;

    let mut png_data = Vec::new();
    let mut cursor = Cursor::new(&mut png_data);
    if img.write_to(&mut cursor, image::ImageFormat::Png).is_err() {
        return Ok(false);
    }

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);
    let width_cells = ctx.available_width();

    if ctx.needs_newline {
        writeln!(w)?;
    }
    ctx.write_indent(w)?;

    // iTerm2 inline image protocol
    write!(w, "\x1b]1337;File=inline=1;width={}:{}", width_cells, b64)?;
    // Use ST (String Terminator) - BEL (\x07) for iTerm2
    write!(w, "\x07")?;
    writeln!(w)?;

    ctx.needs_newline = true;
    Ok(true)
}

#[cfg(feature = "images")]
fn render_kitty<W: Write>(
    w: &mut W,
    ctx: &mut RenderContext<'_>,
    img: &image::DynamicImage,
) -> std::io::Result<bool> {
    use base64::Engine;
    use std::io::Cursor;

    let mut png_data = Vec::new();
    let mut cursor = Cursor::new(&mut png_data);
    if img.write_to(&mut cursor, image::ImageFormat::Png).is_err() {
        return Ok(false);
    }

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);

    if ctx.needs_newline {
        writeln!(w)?;
    }
    ctx.write_indent(w)?;

    // Kitty graphics protocol — chunked transmission
    let chunk_size = 4096;
    let chunks: Vec<&str> = b64
        .as_bytes()
        .chunks(chunk_size)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect();

    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        if i == 0 {
            // First chunk: include format and action
            write!(
                w,
                "\x1b_Gf=100,a=T,m={};{}\x1b\\",
                if is_last { 0 } else { 1 },
                chunk
            )?;
        } else {
            // Continuation chunks
            write!(w, "\x1b_Gm={};{}\x1b\\", if is_last { 0 } else { 1 }, chunk)?;
        }
    }
    writeln!(w)?;

    ctx.needs_newline = true;
    Ok(true)
}

/// Fallback when images feature is not enabled.
#[cfg(not(feature = "images"))]
pub fn render_inline_image<W: std::io::Write>(
    _w: &mut W,
    _ctx: &mut super::RenderContext<'_>,
    _url: &str,
) -> std::io::Result<bool> {
    Ok(false)
}
