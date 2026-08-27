use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::cli::ServeArgs;
use crate::cli::ThemeName;
use crate::html;

fn syntax_theme() -> &'static str {
    crate::options::syntax_theme()
}

/// Owned so the eight `let theme = doc_theme();` sites keep passing `&theme`,
/// including those inside `move` closures and the axum handler.
fn doc_theme() -> ThemeName {
    crate::options::theme().clone()
}

/// Parse the `--host` value into an address to bind.
fn bind_ip(host: &str) -> Result<IpAddr, Box<dyn std::error::Error>> {
    if host.eq_ignore_ascii_case("localhost") {
        return Ok(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
    }
    match host.parse::<IpAddr>() {
        Ok(ip) => Ok(ip),
        Err(_) => {
            let msg = format!(
                "Invalid --host '{}': expected an IP address like 127.0.0.1 or 0.0.0.0",
                host
            );
            Err(msg.into())
        }
    }
}

/// The loopback host to print for a bind, in URL form.
fn loopback_host(ip: IpAddr) -> &'static str {
    match ip {
        IpAddr::V4(_) => "127.0.0.1",
        IpAddr::V6(_) => "[::1]",
    }
}

/// Format an IP for a URL. IPv6 needs brackets.
fn url_host(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(v4) => v4.to_string(),
        IpAddr::V6(v6) => format!("[{}]", v6),
    }
}

/// The URL to open in the browser. A wildcard bind is reached over loopback.
fn browser_url(addr: SocketAddr) -> String {
    let ip = addr.ip();
    let host = if ip.is_unspecified() {
        loopback_host(ip).to_string()
    } else {
        url_host(ip)
    };
    format!("http://{}:{}", host, addr.port())
}

/// Detect the LAN IP by probing a UDP socket (doesn't send traffic).
fn lan_ip() -> Option<IpAddr> {
    let sock = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect("8.8.8.8:80").ok()?;
    sock.local_addr()
        .ok()
        .map(|a| a.ip())
        .filter(|ip| !ip.is_loopback())
}

/// Print the server banner: source description, listening URLs, and hint.
fn print_serve_banner(addr: SocketAddr, source: &str) {
    let ip = addr.ip();
    let port = addr.port();

    eprintln!("  Serving {}", source);

    if ip.is_loopback() || ip.is_unspecified() {
        eprintln!("    Local:   http://{}:{}", loopback_host(ip), port);
    }

    if ip.is_loopback() {
        eprintln!("    (local only - use --host 0.0.0.0 to expose on your network)");
    } else {
        // Wildcard or a specific interface: reachable from off this machine.
        let network = if ip.is_unspecified() {
            lan_ip().map(url_host)
        } else {
            Some(url_host(ip))
        };
        if let Some(network) = network {
            eprintln!("    Network: http://{}:{}", network, port);
        }
        eprintln!("    Warning: unauthenticated - anyone who can reach this can edit these files");
    }

    eprintln!("  Press Ctrl+C to stop");
}

struct AppState {
    /// Single-file mode: one entry with key ""
    /// Multi-file mode: entries keyed by filename
    files: RwLock<HashMap<String, FileEntry>>,
    index_html: RwLock<Option<String>>,
    tx: broadcast::Sender<String>,
    custom_css: String,
    #[allow(dead_code)]
    multi: bool,
    #[allow(dead_code)]
    stdin_mode: bool,
    file_paths: HashMap<String, PathBuf>,
    dir_path: Option<PathBuf>,
    /// Directory that static assets are resolved against: images written
    /// by the drag-and-drop uploader, and anything else the markdown
    /// references relatively. `None` in stdin mode, where the document
    /// has no location on disk.
    base_dir: Option<PathBuf>,
    filenames: RwLock<Vec<String>>,
}

struct FileEntry {
    full_html: String,
    raw_html: String,
    markdown: String,
}

pub async fn start_server(args: &ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Raw HTML in the served markdown is dropped unless the user opted in.
    // Set before anything renders, so the watcher threads inherit it.
    crate::options::set_allow_raw_html(args.unsafe_html);

    // Fail on a bad --host before we read files or consume stdin. The bind
    // sites parse it again; this call exists only to move the error earlier.
    let _validated = bind_ip(&args.host)?;

    // Spawn shutdown handler: on signal, print message and exit.
    // This runs independently of axum's graceful shutdown so that
    // long-lived SSE connections and watcher threads cannot prevent exit.
    spawn_shutdown_handler();

    // Determine mode: stdin, single file, directory, or multi-file
    let files_arg = &args.files;

    let is_stdin = files_arg.is_empty() || (files_arg.len() == 1 && files_arg[0] == "-");

    if is_stdin {
        return serve_stdin(args).await;
    }

    // Check if it's a directory
    if files_arg.len() == 1 {
        let p = std::path::Path::new(&files_arg[0]);
        if p.is_dir() {
            return serve_directory(args, p).await;
        }
        // Single file
        return serve_single_file(args, &files_arg[0]).await;
    }

    // Multiple files
    serve_multi_files(args).await
}

async fn serve_stdin(args: &ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;

    if std::io::stdin().is_terminal() {
        return Err("No input on stdin. Pipe markdown or specify a file.".into());
    }

    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;

    let theme = doc_theme();
    let custom_css = crate::options::custom_css().to_string();
    let full = html::render_standalone(&buf, syntax_theme(), &theme, "stdin", &custom_css);
    let raw = html::render_fragment(&buf, syntax_theme());

    let (tx, _) = broadcast::channel::<String>(16);

    let mut files_map = HashMap::new();
    files_map.insert(
        String::new(),
        FileEntry {
            full_html: full,
            raw_html: raw,
            markdown: buf,
        },
    );

    let state = Arc::new(AppState {
        files: RwLock::new(files_map),
        index_html: RwLock::new(None),
        tx,
        custom_css,
        multi: false,
        stdin_mode: true,
        file_paths: HashMap::new(),
        dir_path: None,
        base_dir: None,
        filenames: RwLock::new(vec![]),
    });

    let app = Router::new()
        .route("/", get(serve_page_single))
        .route("/raw", get(serve_raw_single))
        .with_state(state);

    let addr = SocketAddr::new(bind_ip(&args.host)?, args.port.unwrap_or(0));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let bound = listener.local_addr()?;
    let url = browser_url(bound);

    print_serve_banner(bound, "from stdin (no live reload)");

    let _ = open::that(&url);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_single_file(args: &ServeArgs, file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(file)
        .canonicalize()
        .map_err(|e| format!("Cannot open '{}': {}", file, e))?;
    let markdown = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Cannot read '{}': {}", file, e))?;

    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "preview".to_string());

    let theme = doc_theme();
    let custom_css = crate::options::custom_css().to_string();
    let full = html::render_page(&markdown, syntax_theme(), &theme, &filename, &custom_css);
    let raw = html::render_fragment(&markdown, syntax_theme());

    let (tx, _) = broadcast::channel::<String>(16);

    let mut files_map = HashMap::new();
    files_map.insert(
        String::new(),
        FileEntry {
            full_html: full,
            raw_html: raw,
            markdown,
        },
    );

    let mut file_paths = HashMap::new();
    file_paths.insert(String::new(), file_path.clone());

    let state = Arc::new(AppState {
        files: RwLock::new(files_map),
        index_html: RwLock::new(None),
        tx,
        custom_css,
        multi: false,
        stdin_mode: false,
        file_paths,
        dir_path: None,
        base_dir: file_path.parent().map(|p| p.to_path_buf()),
        filenames: RwLock::new(vec![]),
    });

    // File watcher
    {
        let state = state.clone();
        let path = file_path.clone();
        let fname = filename.clone();

        std::thread::spawn(move || {
            use notify::{RecursiveMode, Watcher};

            let (ntx, nrx) = std::sync::mpsc::channel();
            let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
                if let Ok(event) = res
                    && event.kind.is_modify()
                {
                    let _ = ntx.send(());
                }
            })
            .expect("Failed to create file watcher");

            watcher
                .watch(&path, RecursiveMode::NonRecursive)
                .expect("Failed to watch file");

            let theme = doc_theme();
            let mut last = Instant::now();
            for _ in nrx {
                if last.elapsed() < Duration::from_millis(300) {
                    continue;
                }
                last = Instant::now();

                if let Ok(content) = std::fs::read_to_string(&path) {
                    let css = &state.custom_css;
                    let full = html::render_page(&content, syntax_theme(), &theme, &fname, css);
                    let raw = html::render_fragment(&content, syntax_theme());
                    let mut files = state.files.write().unwrap();
                    files.insert(
                        String::new(),
                        FileEntry {
                            full_html: full,
                            raw_html: raw,
                            markdown: content,
                        },
                    );
                    let _ = state.tx.send("reload".to_string());
                }
            }
        });
    }

    let app = Router::new()
        .route("/", get(serve_page_single))
        .route("/raw", get(serve_raw_single))
        .route("/source", get(get_source_single).put(put_source_single))
        .route("/upload", post(upload_handler))
        .route("/events", get(sse_handler))
        .fallback(static_fallback)
        .with_state(state);

    let addr = SocketAddr::new(bind_ip(&args.host)?, args.port.unwrap_or(0));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let bound = listener.local_addr()?;
    let url = browser_url(bound);

    print_serve_banner(bound, &filename);

    let _ = open::that(&url);

    axum::serve(listener, app).await?;

    Ok(())
}

fn scan_md_files(dir: &std::path::Path) -> Vec<(String, PathBuf)> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension()
                && (ext == "md" || ext == "markdown")
                && let Some(name) = path.file_name()
            {
                results.push((name.to_string_lossy().to_string(), path));
            }
        }
    }
    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}

async fn serve_directory(
    args: &ServeArgs,
    dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = dir.canonicalize()?;
    let md_files = scan_md_files(&dir);

    if md_files.is_empty() {
        return Err(format!("No .md files found in '{}'", dir.display()).into());
    }

    let theme = doc_theme();
    let custom_css = crate::options::custom_css().to_string();
    let filenames: Vec<String> = md_files.iter().map(|(n, _)| n.clone()).collect();
    let index = html::render_index_page(&filenames, &theme, true);

    let mut files_map = HashMap::new();
    let mut file_paths = HashMap::new();
    for (name, path) in &md_files {
        if let Ok(content) = std::fs::read_to_string(path) {
            let full = html::render_page_multi(
                &content,
                syntax_theme(),
                &theme,
                name,
                &filenames,
                name,
                &custom_css,
            );
            let raw = html::render_fragment(&content, syntax_theme());
            files_map.insert(
                name.clone(),
                FileEntry {
                    full_html: full,
                    raw_html: raw,
                    markdown: content,
                },
            );
            file_paths.insert(name.clone(), path.clone());
        }
    }

    let (tx, _) = broadcast::channel::<String>(16);

    let state = Arc::new(AppState {
        files: RwLock::new(files_map),
        index_html: RwLock::new(Some(index)),
        tx,
        custom_css,
        multi: true,
        stdin_mode: false,
        file_paths,
        dir_path: Some(dir.clone()),
        base_dir: Some(dir.clone()),
        filenames: RwLock::new(filenames.clone()),
    });

    // Watch the directory
    {
        let state = state.clone();
        let dir = dir.clone();

        std::thread::spawn(move || {
            use notify::{RecursiveMode, Watcher};

            let (ntx, nrx) = std::sync::mpsc::channel();
            let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
                if let Ok(event) = res
                    && event.kind.is_modify()
                {
                    for path in &event.paths {
                        if let Some(name) = path.file_name() {
                            let _ = ntx.send(name.to_string_lossy().to_string());
                        }
                    }
                }
            })
            .expect("Failed to create file watcher");

            watcher
                .watch(&dir, RecursiveMode::NonRecursive)
                .expect("Failed to watch directory");

            let theme = doc_theme();
            let mut last = Instant::now();
            for changed_file in nrx {
                if last.elapsed() < Duration::from_millis(300) {
                    continue;
                }
                last = Instant::now();

                let current_filenames = state.filenames.read().unwrap().clone();
                if !current_filenames.contains(&changed_file) {
                    continue;
                }

                let path = dir.join(&changed_file);
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let css = &state.custom_css;
                    let full = html::render_page_multi(
                        &content,
                        syntax_theme(),
                        &theme,
                        &changed_file,
                        &current_filenames,
                        &changed_file,
                        css,
                    );
                    let raw = html::render_fragment(&content, syntax_theme());
                    let mut files = state.files.write().unwrap();
                    files.insert(
                        changed_file.clone(),
                        FileEntry {
                            full_html: full,
                            raw_html: raw,
                            markdown: content,
                        },
                    );
                    let _ = state.tx.send(format!(r#"{{"file":"{}"}}"#, changed_file));
                }
            }
        });
    }

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/create", post(create_file))
        .route("/upload", post(upload_handler))
        .route("/events", get(sse_handler))
        .route("/{file}", get(serve_page_multi))
        .route("/{file}/raw", get(serve_raw_multi))
        .route(
            "/{file}/source",
            get(get_source_multi).put(put_source_multi),
        )
        .fallback(static_fallback)
        .with_state(state);

    let addr = SocketAddr::new(bind_ip(&args.host)?, args.port.unwrap_or(0));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let bound = listener.local_addr()?;
    let url = browser_url(bound);

    print_serve_banner(
        bound,
        &format!("{} files from {}", filenames.len(), dir.display()),
    );

    let _ = open::that(&url);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_multi_files(args: &ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    let theme = doc_theme();
    let custom_css = crate::options::custom_css().to_string();

    let mut entries: Vec<(String, PathBuf, String)> = Vec::new();
    for file in &args.files {
        let file_path = PathBuf::from(file)
            .canonicalize()
            .map_err(|e| format!("Cannot open '{}': {}", file, e))?;
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Cannot read '{}': {}", file, e))?;
        let name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file.clone());
        entries.push((name, file_path, content));
    }

    let filenames: Vec<String> = entries.iter().map(|(n, _, _)| n.clone()).collect();
    let paths: Vec<PathBuf> = entries.iter().map(|(_, p, _)| p.clone()).collect();

    let mut files_map = HashMap::new();
    for (name, _, content) in &entries {
        let full = html::render_page_multi(
            content,
            syntax_theme(),
            &theme,
            name,
            &filenames,
            name,
            &custom_css,
        );
        let raw = html::render_fragment(content, syntax_theme());
        files_map.insert(
            name.clone(),
            FileEntry {
                full_html: full,
                raw_html: raw,
                markdown: content.clone(),
            },
        );
    }

    let file_paths: HashMap<String, PathBuf> = entries
        .iter()
        .map(|(n, p, _)| (n.clone(), p.clone()))
        .collect();

    let index = html::render_index_page(&filenames, &theme, false);
    let (tx, _) = broadcast::channel::<String>(16);

    let state = Arc::new(AppState {
        files: RwLock::new(files_map),
        index_html: RwLock::new(Some(index)),
        tx,
        custom_css,
        multi: true,
        stdin_mode: false,
        file_paths,
        dir_path: None,
        base_dir: paths
            .first()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf()),
        filenames: RwLock::new(filenames.clone()),
    });

    // Watch each file
    {
        let state = state.clone();
        let paths = paths.clone();
        let filenames = filenames.clone();

        std::thread::spawn(move || {
            use notify::{RecursiveMode, Watcher};

            let (ntx, nrx) = std::sync::mpsc::channel();
            let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
                if let Ok(event) = res
                    && event.kind.is_modify()
                {
                    for path in &event.paths {
                        if let Some(name) = path.file_name() {
                            let _ = ntx.send(name.to_string_lossy().to_string());
                        }
                    }
                }
            })
            .expect("Failed to create file watcher");

            for path in &paths {
                watcher
                    .watch(path, RecursiveMode::NonRecursive)
                    .expect("Failed to watch file");
            }

            let theme = doc_theme();
            let mut last = Instant::now();
            let name_to_path: HashMap<String, PathBuf> = filenames
                .iter()
                .cloned()
                .zip(paths.iter().cloned())
                .collect();

            for changed_file in nrx {
                if last.elapsed() < Duration::from_millis(300) {
                    continue;
                }
                last = Instant::now();

                if let Some(path) = name_to_path.get(&changed_file)
                    && let Ok(content) = std::fs::read_to_string(path)
                {
                    let css = &state.custom_css;
                    let full = html::render_page_multi(
                        &content,
                        syntax_theme(),
                        &theme,
                        &changed_file,
                        &filenames,
                        &changed_file,
                        css,
                    );
                    let raw = html::render_fragment(&content, syntax_theme());
                    let mut files = state.files.write().unwrap();
                    files.insert(
                        changed_file.clone(),
                        FileEntry {
                            full_html: full,
                            raw_html: raw,
                            markdown: content,
                        },
                    );
                    let _ = state.tx.send(format!(r#"{{"file":"{}"}}"#, changed_file));
                }
            }
        });
    }

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/upload", post(upload_handler))
        .route("/events", get(sse_handler))
        .route("/{file}", get(serve_page_multi))
        .route("/{file}/raw", get(serve_raw_multi))
        .route(
            "/{file}/source",
            get(get_source_multi).put(put_source_multi),
        )
        .fallback(static_fallback)
        .with_state(state);

    let addr = SocketAddr::new(bind_ip(&args.host)?, args.port.unwrap_or(0));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let bound = listener.local_addr()?;
    let url = browser_url(bound);

    print_serve_banner(bound, &format!("{} files", filenames.len()));

    let _ = open::that(&url);

    axum::serve(listener, app).await?;

    Ok(())
}

// --- Content negotiation helpers ---

fn wants_markdown(headers: &axum::http::HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/markdown"))
        .unwrap_or(false)
}

fn markdown_response(markdown: &str) -> Response {
    let tokens = crate::estimate_tokens(markdown);
    Response::builder()
        .header(header::CONTENT_TYPE, "text/markdown; charset=utf-8")
        .header(header::VARY, "Accept")
        .header("X-Markdown-Tokens", tokens.to_string())
        .body(axum::body::Body::from(markdown.to_string()))
        .unwrap()
        .into_response()
}

// --- Static asset serving ---

/// Content-Type for the file extensions `mdx serve` hands out as static
/// assets. Anything not listed is refused: the server binds 0.0.0.0, so
/// it must not become a general-purpose file server for whatever
/// directory it was pointed at.
///
/// Covers every image type the drag-and-drop uploader accepts (png,
/// jpeg, gif, webp, svg) plus the other media a markdown document
/// normally references.
fn static_content_type(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "png" | "apng" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "avif" => "image/avif",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "tif" | "tiff" => "image/tiff",
        _ => return None,
    })
}

/// Percent-decode a URL path. Returns `None` if the decoded bytes are not
/// valid UTF-8.
///
/// Decoding happens *before* validation (see `resolve_static_path`), so
/// an encoded `%2e%2e%2f` cannot smuggle a `../` past the component
/// check the way it would if the checks ran on the raw path.
fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Some(hi) = (bytes[i + 1] as char).to_digit(16)
            && let Some(lo) = (bytes[i + 2] as char).to_digit(16)
        {
            out.push((hi * 16 + lo) as u8);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// Resolve a decoded, relative URL path against `base`, refusing anything
/// that escapes the base directory.
///
/// Defence in depth, in order:
///  1. Every component must be `Component::Normal`; `..`, a leading `/`
///     and Windows prefixes such as `C:` are rejected. Each segment is
///     re-parsed as a path on its own so that a segment like `c:` (and,
///     on Windows, `..\x`) cannot smuggle a prefix or a parent component
///     through in one piece.
///  2. The extension must be one this server publishes.
///  3. The candidate is canonicalized and must still live under the
///     canonical base directory. This is what stops a symlink inside the
///     served directory from pointing somewhere else; `tower_http`'s
///     `ServeDir` does step 1 but not this one.
///  4. It must be a regular file, not a directory.
fn resolve_static_path(base: &std::path::Path, rel: &str) -> Option<PathBuf> {
    if rel.contains('\0') {
        return None;
    }

    let mut candidate = base.to_path_buf();
    let mut pushed = false;
    for component in std::path::Path::new(rel).components() {
        match component {
            std::path::Component::Normal(seg) => {
                if !std::path::Path::new(seg)
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_)))
                {
                    return None;
                }
                candidate.push(seg);
                pushed = true;
            }
            std::path::Component::CurDir => {}
            _ => return None,
        }
    }
    if !pushed {
        return None;
    }

    let ext = candidate
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    static_content_type(&ext)?;

    // Both sides are canonicalized: on Windows that means both carry the
    // `\\?\` verbatim prefix, and `starts_with` compares whole path
    // components, so a sibling directory such as `notes-private` cannot
    // pass a containment check against `notes`.
    let canonical_base = std::fs::canonicalize(base).ok()?;
    let resolved = std::fs::canonicalize(&candidate).ok()?;
    if !resolved.starts_with(&canonical_base) || !resolved.is_file() {
        return None;
    }

    Some(resolved)
}

/// True when `name` is a single plain path segment.
///
/// A string check for '/', '\\' and ".." is not enough: on Windows a
/// drive-relative name like "C:evil.md" carries a prefix, and PathBuf::push
/// replaces the whole path rather than appending when it sees one.
fn is_plain_file_name(name: &str) -> bool {
    if name.is_empty() || name.contains('\0') || name.contains('\\') {
        return false;
    }
    // Rejected on every platform, not just Windows: the guard must not depend
    // on where the server runs. On Linux "C:evil.md" is a legal file name, so
    // the component walk below would accept it.
    let b = name.as_bytes();
    if b.len() >= 2 && b[1] == b':' && b[0].is_ascii_alphabetic() {
        return false;
    }
    let mut components = std::path::Path::new(name).components();
    matches!(components.next(), Some(std::path::Component::Normal(_)))
        && components.next().is_none()
}

fn static_not_found() -> Response {
    (StatusCode::NOT_FOUND, "Not found").into_response()
}

/// Serve `rel` (an already percent-decoded relative path) from the
/// directory this server was pointed at.
fn static_file_response(base: Option<&std::path::Path>, rel: &str) -> Response {
    let Some(base) = base else {
        return static_not_found();
    };
    let Some(path) = resolve_static_path(base, rel) else {
        return static_not_found();
    };

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    let content_type = static_content_type(&ext).unwrap_or("application/octet-stream");

    match std::fs::read(&path) {
        Ok(data) => Response::builder()
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CACHE_CONTROL, "no-cache")
            .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
            // SVG is an active document type: navigating directly to a served
            // .svg runs its scripts in this server's origin, which can PUT
            // /source. `nosniff` does not stop that. CSP is ignored on
            // subresources, so `<img>` rendering is unaffected.
            .header(
                header::CONTENT_SECURITY_POLICY,
                "default-src 'none'; sandbox",
            )
            .body(axum::body::Body::from(data))
            .unwrap()
            .into_response(),
        Err(_) => static_not_found(),
    }
}

/// Router fallback: serve a static asset from the document's directory.
///
/// Registered with `Router::fallback` rather than as a `/{*path}` route
/// for two reasons. matchit 0.8 treats `/{*path}` and the existing
/// `/{file}` route as a conflict, which axum surfaces as a startup panic.
/// And a fallback runs only after every declared route has failed to
/// match, so it cannot shadow `/raw`, `/source`, `/upload`, `/events` or
/// the markdown routes.
async fn static_fallback(State(state): State<Arc<AppState>>, uri: axum::http::Uri) -> Response {
    match percent_decode(uri.path().trim_start_matches('/')) {
        Some(rel) => static_file_response(state.base_dir.as_deref(), &rel),
        None => static_not_found(),
    }
}

// --- Route handlers ---

async fn serve_page_single(
    headers: axum::http::HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Response {
    let files = state.files.read().unwrap();
    if let Some(entry) = files.get("") {
        if wants_markdown(&headers) {
            return markdown_response(&entry.markdown);
        }
        Html(entry.full_html.clone()).into_response()
    } else {
        Html(String::new()).into_response()
    }
}

async fn serve_raw_single(State(state): State<Arc<AppState>>) -> Html<String> {
    let files = state.files.read().unwrap();
    Html(
        files
            .get("")
            .map(|f| f.raw_html.clone())
            .unwrap_or_default(),
    )
}

async fn serve_index(State(state): State<Arc<AppState>>) -> Html<String> {
    let index = state.index_html.read().unwrap();
    Html(index.clone().unwrap_or_default())
}

async fn serve_page_multi(
    headers: axum::http::HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(file): Path<String>,
) -> Response {
    // Resolve and release the lock before any filesystem work below.
    let rendered = {
        let files = state.files.read().unwrap();
        files.get(&file).map(|entry| {
            if wants_markdown(&headers) {
                markdown_response(&entry.markdown)
            } else {
                Html(entry.full_html.clone()).into_response()
            }
        })
    };

    match rendered {
        Some(response) => response,
        // Not a markdown document we know about. It may be an asset
        // sitting next to the markdown, e.g. a root-level `/logo.png`,
        // which matches this single-segment route and so never reaches
        // the router fallback. `file` was already percent-decoded by
        // axum's Path extractor, so it is validated here, not decoded
        // a second time.
        None => static_file_response(state.base_dir.as_deref(), &file),
    }
}

async fn serve_raw_multi(
    State(state): State<Arc<AppState>>,
    Path(file): Path<String>,
) -> Html<String> {
    let files = state.files.read().unwrap();
    Html(
        files
            .get(&file)
            .map(|f| f.raw_html.clone())
            .unwrap_or_default(),
    )
}

async fn sse_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .map(|r| Ok(Event::default().data(r.unwrap_or_else(|_| "reload".to_string()))));
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Spawn a background task that exits the process on shutdown signal.
/// This bypasses axum's graceful shutdown (which blocks on active SSE
/// connections) and also kills any watcher threads that would otherwise
/// keep the process alive.
fn spawn_shutdown_handler() {
    tokio::spawn(async {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};

            let mut sigint =
                signal(SignalKind::interrupt()).expect("Failed to install SIGINT handler");
            let mut sigterm =
                signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");

            tokio::select! {
                _ = sigint.recv() => {}
                _ = sigterm.recv() => {}
            }
        }

        #[cfg(windows)]
        {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        }

        eprintln!("\n  Stopped.");
        std::process::exit(0);
    });
}

// --- Source (editor) handlers ---

async fn get_source_single(State(state): State<Arc<AppState>>) -> String {
    let files = state.files.read().unwrap();
    files
        .get("")
        .map(|f| f.markdown.clone())
        .unwrap_or_default()
}

async fn put_source_single(State(state): State<Arc<AppState>>, body: String) -> StatusCode {
    let path = match state.file_paths.get("") {
        Some(p) => p.clone(),
        None => return StatusCode::NOT_FOUND,
    };
    match atomic_write(&path, &body) {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn get_source_multi(State(state): State<Arc<AppState>>, Path(file): Path<String>) -> String {
    let files = state.files.read().unwrap();
    files
        .get(&file)
        .map(|f| f.markdown.clone())
        .unwrap_or_default()
}

async fn put_source_multi(
    State(state): State<Arc<AppState>>,
    Path(file): Path<String>,
    body: String,
) -> StatusCode {
    // An unknown name in directory mode is joined to the served directory, so
    // it must be a plain file name. `create_file` already guards this shape;
    // this endpoint did not, which made PUT /%2e%2e%2f%2e%2e%2f.bashrc/source
    // an unauthenticated arbitrary file write -- reachable from the network,
    // since the server binds 0.0.0.0.
    let in_served_dir = || {
        if !is_plain_file_name(&file) {
            return None;
        }
        state.dir_path.as_ref().map(|d| d.join(&file))
    };

    let path = match state.file_paths.get(&file).cloned().or_else(in_served_dir) {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND,
    };
    match atomic_write(&path, &body) {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn create_file(State(state): State<Arc<AppState>>, body: String) -> (StatusCode, String) {
    let dir = match &state.dir_path {
        Some(d) => d,
        None => return (StatusCode::BAD_REQUEST, "Not in directory mode".to_string()),
    };

    let mut filename = body.trim().to_string();
    if filename.is_empty() {
        return (StatusCode::BAD_REQUEST, "Filename is empty".to_string());
    }
    if !filename.ends_with(".md") {
        filename.push_str(".md");
    }
    if !is_plain_file_name(&filename) || filename.len() > 255 {
        return (StatusCode::BAD_REQUEST, "Invalid filename".to_string());
    }

    // Check for duplicates
    {
        let names = state.filenames.read().unwrap();
        if names.contains(&filename) {
            return (StatusCode::CONFLICT, "File already exists".to_string());
        }
    }

    // Derive a title from the filename (strip .md)
    let title = filename.strip_suffix(".md").unwrap_or(&filename);
    let markdown = format!("# {title}\n");

    let file_path = dir.join(&filename);
    if let Err(e) = atomic_write(&file_path, &markdown) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create file: {e}"),
        );
    }

    // Update shared filenames list
    let updated_filenames = {
        let mut names = state.filenames.write().unwrap();
        names.push(filename.clone());
        names.sort();
        names.clone()
    };

    // Render the new file's page
    let theme = doc_theme();
    let full = html::render_page_multi(
        &markdown,
        syntax_theme(),
        &theme,
        &filename,
        &updated_filenames,
        &filename,
        &state.custom_css,
    );
    let raw = html::render_fragment(&markdown, syntax_theme());

    // Insert into files map
    {
        let mut files = state.files.write().unwrap();
        files.insert(
            filename.clone(),
            FileEntry {
                full_html: full,
                raw_html: raw,
                markdown,
            },
        );
    }

    // Rebuild index page
    {
        let new_index = html::render_index_page(&updated_filenames, &theme, true);
        let mut index = state.index_html.write().unwrap();
        *index = Some(new_index);
    }

    (StatusCode::OK, filename)
}

async fn upload_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> (StatusCode, String) {
    // Determine the base directory for assets. This is the same directory
    // the static-asset handler serves from, so the "assets/<name>" path
    // returned below is guaranteed to resolve.
    let base_dir = match state.base_dir {
        Some(ref dir) => dir.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                r#"{"error":"No file context"}"#.to_string(),
            );
        }
    };

    let assets_dir = base_dir.join("assets");
    if let Err(e) = std::fs::create_dir_all(&assets_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(r#"{{"error":"Failed to create assets dir: {}"}}"#, e),
        );
    }

    // Extract filename from Content-Disposition or Content-Type
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Handle multipart form data
    let (filename, file_data) = if content_type.starts_with("multipart/form-data") {
        // Simple multipart parser: find the file content between boundaries
        match parse_multipart(&body, content_type) {
            Some(result) => result,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    r#"{"error":"Failed to parse upload"}"#.to_string(),
                );
            }
        }
    } else {
        // Raw upload - determine extension from content type
        let ext = match content_type {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/gif" => "gif",
            "image/webp" => "webp",
            "image/svg+xml" => "svg",
            _ => "png",
        };
        (format!("image.{}", ext), body.to_vec())
    };

    // The name comes from the client (multipart Content-Disposition), so
    // strip any directory component before it is joined to assets_dir.
    let filename = sanitize_upload_filename(&filename);

    // Deduplicate filename
    let final_name = dedup_filename(&assets_dir, &filename);
    let file_path = assets_dir.join(&final_name);

    if let Err(e) = std::fs::write(&file_path, &file_data) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(r#"{{"error":"Failed to write file: {}"}}"#, e),
        );
    }

    let path = format!("assets/{}", final_name);
    (StatusCode::OK, format!(r#"{{"path":"{}"}}"#, path))
}

fn parse_multipart(body: &[u8], content_type: &str) -> Option<(String, Vec<u8>)> {
    // Extract boundary from content-type
    let boundary = content_type
        .split("boundary=")
        .nth(1)?
        .trim_matches('"')
        .to_string();

    let boundary_marker = format!("--{}", boundary);
    let body_str = String::from_utf8_lossy(body);

    // Find the filename in Content-Disposition header
    let filename = body_str
        .lines()
        .find(|line| line.contains("filename="))
        .and_then(|line| {
            line.split("filename=")
                .nth(1)
                .map(|s| s.trim_matches('"').trim_matches('\'').to_string())
        })
        .unwrap_or_else(|| "upload.png".to_string());

    // Find the file data: it's after the empty line following headers, before the next boundary
    let parts: Vec<&[u8]> = split_bytes(body, boundary_marker.as_bytes());
    for part in parts.iter().skip(1) {
        // skip preamble
        // Find \r\n\r\n or \n\n (end of headers)
        if let Some(header_end) = find_double_newline(part) {
            let data_start = header_end;
            let mut data = &part[data_start..];
            // Trim trailing \r\n before boundary
            if data.ends_with(b"\r\n") {
                data = &data[..data.len() - 2];
            } else if data.ends_with(b"\n") {
                data = &data[..data.len() - 1];
            }
            if !data.is_empty() {
                return Some((filename, data.to_vec()));
            }
        }
    }

    None
}

fn split_bytes<'a>(haystack: &'a [u8], needle: &[u8]) -> Vec<&'a [u8]> {
    let mut parts = Vec::new();
    let mut start = 0;
    let nlen = needle.len();

    while start <= haystack.len() {
        if let Some(pos) = haystack[start..].windows(nlen).position(|w| w == needle) {
            parts.push(&haystack[start..start + pos]);
            start = start + pos + nlen;
        } else {
            parts.push(&haystack[start..]);
            break;
        }
    }
    parts
}

fn find_double_newline(data: &[u8]) -> Option<usize> {
    // Look for \r\n\r\n
    if let Some(pos) = data.windows(4).position(|w| w == b"\r\n\r\n") {
        return Some(pos + 4);
    }
    // Look for \n\n
    if let Some(pos) = data.windows(2).position(|w| w == b"\n\n") {
        return Some(pos + 2);
    }
    None
}

/// Reduce a client-supplied upload name to a single, safe file name.
/// Any directory component (`/`, `\`, a drive prefix) is discarded, as
/// are characters that are illegal in Windows file names. An empty or
/// dot-only result falls back to "upload.png".
fn sanitize_upload_filename(name: &str) -> String {
    let name = match name.rfind(['/', '\\']) {
        Some(idx) => &name[idx + 1..],
        None => name,
    };
    let cleaned: String = name
        .trim()
        .trim_matches('.')
        .chars()
        .filter(|c| !matches!(c, ':' | '\0' | '<' | '>' | '"' | '|' | '?' | '*'))
        .collect();

    // Windows reserved device names are matched on the stem, so `CON.png`
    // opens the console instead of creating a file.
    let stem = cleaned.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved = matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    );

    if cleaned.is_empty() || reserved || cleaned.len() > 200 {
        "upload.png".to_string()
    } else {
        cleaned
    }
}

fn dedup_filename(dir: &std::path::Path, filename: &str) -> String {
    if !dir.join(filename).exists() {
        return filename.to_string();
    }

    let stem = std::path::Path::new(filename)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let ext = std::path::Path::new(filename)
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut counter = 1;
    loop {
        let new_name = if ext.is_empty() {
            format!("{}-{}", stem, counter)
        } else {
            format!("{}-{}.{}", stem, counter, ext)
        };
        if !dir.join(&new_name).exists() {
            return new_name;
        }
        counter += 1;
    }
}

fn atomic_write(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or(path);
    let tmp = dir.join(format!(".md-tmp-{}", std::process::id()));
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)
}

use std::io::IsTerminal;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_decode() {
        assert_eq!(
            percent_decode("assets/a%20b.png").unwrap(),
            "assets/a b.png"
        );
        assert_eq!(percent_decode("plain.png").unwrap(), "plain.png");
        assert_eq!(percent_decode("%2e%2e%2fetc").unwrap(), "../etc");
        assert_eq!(percent_decode("%2E%2E%2Fetc").unwrap(), "../etc");
        // A stray or truncated '%' is passed through, not swallowed.
        assert_eq!(percent_decode("100%.png").unwrap(), "100%.png");
        assert_eq!(percent_decode("trailing%2").unwrap(), "trailing%2");
        // Invalid UTF-8 is rejected outright.
        assert!(percent_decode("%ff%fe.png").is_none());
    }

    #[test]
    fn test_static_content_type_allowlist() {
        assert_eq!(static_content_type("png"), Some("image/png"));
        assert_eq!(static_content_type("jpeg"), Some("image/jpeg"));
        assert_eq!(static_content_type("svg"), Some("image/svg+xml"));
        assert_eq!(static_content_type("webp"), Some("image/webp"));
        assert_eq!(static_content_type("gif"), Some("image/gif"));
        // Not an asset type: never served.
        assert_eq!(static_content_type("txt"), None);
        assert_eq!(static_content_type("env"), None);
        assert_eq!(static_content_type("md"), None);
        assert_eq!(static_content_type(""), None);
    }

    #[test]
    fn test_resolve_static_path() {
        let dir = std::env::temp_dir().join("md-unit-static-resolve");
        let outside = std::env::temp_dir().join("md-unit-static-outside.png");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::write(dir.join("assets").join("ok.png"), b"x").unwrap();
        std::fs::write(dir.join("notes.txt"), b"x").unwrap();
        std::fs::write(&outside, b"x").unwrap();

        // Allowed.
        assert!(resolve_static_path(&dir, "assets/ok.png").is_some());
        assert!(resolve_static_path(&dir, "./assets/ok.png").is_some());

        // Escapes the base directory.
        assert!(resolve_static_path(&dir, "../md-unit-static-outside.png").is_none());
        assert!(resolve_static_path(&dir, "assets/../../md-unit-static-outside.png").is_none());
        assert!(resolve_static_path(&dir, "/etc/passwd").is_none());

        // Not an asset, not a file, not a path.
        assert!(resolve_static_path(&dir, "notes.txt").is_none());
        assert!(resolve_static_path(&dir, "assets").is_none());
        assert!(resolve_static_path(&dir, "").is_none());
        assert!(resolve_static_path(&dir, "missing.png").is_none());

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_file(&outside);
    }

    #[test]
    fn test_sanitize_upload_filename() {
        assert_eq!(sanitize_upload_filename("shot.png"), "shot.png");
        assert_eq!(sanitize_upload_filename("../../evil.png"), "evil.png");
        assert_eq!(sanitize_upload_filename("..\\..\\evil.png"), "evil.png");
        assert_eq!(sanitize_upload_filename("/etc/cron.d/evil.png"), "evil.png");
        assert_eq!(sanitize_upload_filename("a:b.png"), "ab.png");
        assert_eq!(sanitize_upload_filename(".."), "upload.png");
        assert_eq!(sanitize_upload_filename(""), "upload.png");
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn test_plain_file_names_accepted() {
        assert!(is_plain_file_name("notes.md"));
        assert!(is_plain_file_name("a b.md"));
        assert!(is_plain_file_name("café.md"));
    }

    #[test]
    fn test_separators_and_traversal_rejected() {
        for bad in ["a/b.md", "a\\b.md", "..", "../x.md", "", "/abs.md"] {
            assert!(!is_plain_file_name(bad), "{:?} must be rejected", bad);
        }
    }

    #[test]
    fn test_drive_relative_name_rejected() {
        // On Windows "C:evil.md" is a prefix component and PathBuf::push would
        // replace the served directory entirely. A string check for / \ and ..
        // does not catch it.
        assert!(!is_plain_file_name("C:evil.md"));
        assert!(!is_plain_file_name(r"\\?\C:\evil.md"));
    }

    #[test]
    fn test_nul_rejected() {
        assert!(!is_plain_file_name("a\0b.md"));
    }
}
