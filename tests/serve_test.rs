use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::time::Duration;

// Serialize all serve tests to avoid resource contention
static SERVE_LOCK: Mutex<()> = Mutex::new(());

// ── Helpers ──────────────────────────────────────────────────────────

struct ServeProcess {
    child: Child,
    port: u16,
    // Keep draining stderr so the server doesn't get SIGPIPE/broken pipe
    // when it writes to stderr after we've read the port line.
    _stderr_drain: Option<JoinHandle<()>>,
}

impl ServeProcess {
    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{}", self.port, path)
    }
}

impl Drop for ServeProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn start_serve(args: &[&str]) -> ServeProcess {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mdx"))
        .arg("serve")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start mdx serve");

    let stderr = child.stderr.take().unwrap();
    let mut reader = BufReader::new(stderr);
    let mut port = 0u16;
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                // Lines look like: "  Serving foo.md at http://127.0.0.1:PORT"
                if let Some(idx) = line.rfind(':')
                    && let Ok(p) = line[idx + 1..].trim().parse::<u16>()
                {
                    port = p;
                    break;
                }
            }
        }
    }

    assert!(port > 0, "Failed to detect serve port");

    // Keep reading stderr in background so the pipe stays open and the
    // server doesn't panic on broken pipe when it writes more output.
    let drain = std::thread::spawn(move || {
        let mut buf = String::new();
        while reader.read_line(&mut buf).unwrap_or(0) > 0 {
            buf.clear();
        }
    });

    // Give server a moment to be fully ready
    std::thread::sleep(Duration::from_millis(500));

    ServeProcess {
        child,
        port,
        _stderr_drain: Some(drain),
    }
}

fn http_agent() -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build();
    config.into()
}

fn http_get_with_accept(url: &str, accept: &str) -> (u16, String, Option<String>) {
    let agent = http_agent();
    for attempt in 0..3 {
        match agent.get(url).header("Accept", accept).call() {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let ct = resp
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.to_string());
                let body = resp.into_body().read_to_string().unwrap_or_default();
                return (status, body, ct);
            }
            Err(ureq::Error::StatusCode(code)) => {
                return (code, String::new(), None);
            }
            Err(_) if attempt < 2 => {
                std::thread::sleep(Duration::from_millis(300));
            }
            Err(_) => return (0, String::new(), None),
        }
    }
    (0, String::new(), None)
}

/// Returns (status, body, headers_map) where headers_map has lowercased keys.
fn http_get_with_headers(
    url: &str,
    accept: &str,
) -> (u16, String, std::collections::HashMap<String, String>) {
    let agent = http_agent();
    for attempt in 0..3 {
        match agent.get(url).header("Accept", accept).call() {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let mut headers = std::collections::HashMap::new();
                for name in resp.headers().keys() {
                    if let Some(val) = resp.headers().get(name).and_then(|v| v.to_str().ok()) {
                        headers.insert(name.as_str().to_lowercase(), val.to_string());
                    }
                }
                let body = resp.into_body().read_to_string().unwrap_or_default();
                return (status, body, headers);
            }
            Err(ureq::Error::StatusCode(code)) => {
                return (code, String::new(), std::collections::HashMap::new());
            }
            Err(_) if attempt < 2 => {
                std::thread::sleep(Duration::from_millis(300));
            }
            Err(_) => return (0, String::new(), std::collections::HashMap::new()),
        }
    }
    (0, String::new(), std::collections::HashMap::new())
}

fn http_get(url: &str) -> (u16, String) {
    let agent = http_agent();
    for attempt in 0..3 {
        match agent.get(url).call() {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let body = resp.into_body().read_to_string().unwrap_or_default();
                return (status, body);
            }
            Err(ureq::Error::StatusCode(code)) => {
                let status = code;
                let body = String::new();
                return (status, body);
            }
            Err(_) if attempt < 2 => {
                std::thread::sleep(Duration::from_millis(300));
            }
            Err(_) => return (0, String::new()),
        }
    }
    (0, String::new())
}

fn http_put(url: &str, body: &str) -> u16 {
    let agent = http_agent();
    for attempt in 0..3 {
        match agent.put(url).send(body) {
            Ok(resp) => return resp.status().as_u16(),
            Err(ureq::Error::StatusCode(code)) => return code,
            Err(_) if attempt < 2 => {
                std::thread::sleep(Duration::from_millis(300));
            }
            Err(_) => return 0,
        }
    }
    0
}

fn http_post(url: &str, body: &str) -> (u16, String) {
    let agent = http_agent();
    for attempt in 0..3 {
        match agent.post(url).send(body) {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let resp_body = resp.into_body().read_to_string().unwrap_or_default();
                return (status, resp_body);
            }
            Err(ureq::Error::StatusCode(code)) => {
                return (code, String::new());
            }
            Err(_) if attempt < 2 => {
                std::thread::sleep(Duration::from_millis(300));
            }
            Err(_) => return (0, String::new()),
        }
    }
    (0, String::new())
}

/// A real 68-byte 1x1 transparent PNG. The bytes are not valid UTF-8, so
/// a byte-exact round trip proves the server did not mangle them.
const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0B, 0x49, 0x44, 0x41, 0x54, 0x78, 0xDA, 0x63, 0x60, 0x00, 0x02, 0x00,
    0x00, 0x05, 0x00, 0x01, 0xE9, 0xFA, 0xDC, 0xD8, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44,
    0xAE, 0x42, 0x60, 0x82,
];

/// Returns (status, body_bytes, content_type).
fn http_get_bytes(url: &str) -> (u16, Vec<u8>, Option<String>) {
    let agent = http_agent();
    for attempt in 0..3 {
        match agent.get(url).call() {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let ct = resp
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.to_string());
                let body = resp.into_body().read_to_vec().unwrap_or_default();
                return (status, body, ct);
            }
            Err(ureq::Error::StatusCode(code)) => return (code, Vec::new(), None),
            Err(_) if attempt < 2 => {
                std::thread::sleep(Duration::from_millis(300));
            }
            Err(_) => return (0, Vec::new(), None),
        }
    }
    (0, Vec::new(), None)
}

/// POST a one-part multipart/form-data body, the shape the editor's
/// drag-and-drop uploader sends.
fn http_post_multipart_png(url: &str, filename: &str, data: &[u8]) -> (u16, String) {
    let boundary = "----mdxtestboundary";
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n",
            filename
        )
        .as_bytes(),
    );
    body.extend_from_slice(b"Content-Type: image/png\r\n\r\n");
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let agent = http_agent();
    let content_type = format!("multipart/form-data; boundary={}", boundary);
    match agent
        .post(url)
        .header("content-type", content_type.as_str())
        .send(&body[..])
    {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let text = resp.into_body().read_to_string().unwrap_or_default();
            (status, text)
        }
        Err(ureq::Error::StatusCode(code)) => (code, String::new()),
        Err(_) => (0, String::new()),
    }
}

fn write_tmp(name: &str, content: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let unique_name = format!("md-test-{}-{}", id, name);
    let path = std::env::temp_dir().join(unique_name);
    std::fs::write(&path, content).unwrap();
    path
}

/// Start `mdx serve` and capture the stderr banner (everything up to and
/// including the "Press Ctrl+C" line) alongside the running process.
fn start_serve_with_banner(args: &[&str]) -> (ServeProcess, Vec<String>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mdx"))
        .arg("serve")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start mdx serve");

    let stderr = child.stderr.take().unwrap();
    let mut reader = BufReader::new(stderr);
    let mut banner = Vec::new();
    let mut port = 0u16;
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                banner.push(line.trim_end().to_string());
                if port == 0
                    && let Some(idx) = line.rfind(':')
                    && let Ok(p) = line[idx + 1..].trim().parse::<u16>()
                {
                    port = p;
                }
                if line.contains("Press Ctrl+C") {
                    break;
                }
            }
        }
    }

    assert!(port > 0, "Failed to detect serve port");

    let drain = std::thread::spawn(move || {
        let mut buf = String::new();
        while reader.read_line(&mut buf).unwrap_or(0) > 0 {
            buf.clear();
        }
    });

    std::thread::sleep(Duration::from_millis(500));

    let srv = ServeProcess {
        child,
        port,
        _stderr_drain: Some(drain),
    };
    (srv, banner)
}

/// This machine's non-loopback IPv4, if it has one. Probes a UDP socket, which
/// picks a route without sending traffic; None when there is no default route
/// (offline CI), so the caller can skip.
fn lan_ipv4() -> Option<std::net::IpAddr> {
    let sock = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect("8.8.8.8:80").ok()?;
    let ip = sock.local_addr().ok()?.ip();
    if ip.is_loopback() { None } else { Some(ip) }
}

// ── Single file serve ────────────────────────────────────────────────

#[test]
fn test_serve_single_file_page() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-test.md", "# Hello Serve\n\nContent here.");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (status, body) = http_get(&srv.url("/"));
    assert_eq!(status, 200);
    assert!(
        body.contains("<!DOCTYPE html>"),
        "Should return full HTML page"
    );
    assert!(
        body.contains("Hello Serve"),
        "Should contain rendered heading"
    );
    assert!(
        body.contains("Content here"),
        "Should contain rendered content"
    );

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_single_file_raw() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-raw.md", "# Raw Test\n\nParagraph.");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (status, body) = http_get(&srv.url("/raw"));
    assert_eq!(status, 200);
    assert!(body.contains("<h1>"), "Raw should contain HTML fragment");
    assert!(body.contains("Raw Test"), "Raw should contain heading text");
    assert!(!body.contains("<!DOCTYPE"), "Raw should NOT be a full page");

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_single_file_source_get() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-source.md", "# Source Test\n\nOriginal content.");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (status, body) = http_get(&srv.url("/source"));
    assert_eq!(status, 200);
    assert_eq!(body.trim(), "# Source Test\n\nOriginal content.");

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_single_file_source_put() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-put.md", "# Before\n\nOld content.");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let status = http_put(&srv.url("/source"), "# After\n\nNew content.");
    assert_eq!(status, 200);

    // Verify file was actually written
    std::thread::sleep(Duration::from_millis(100));
    let on_disk = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(on_disk, "# After\n\nNew content.");

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_editor_ui_elements() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-editor-ui.md", "# Editor Test");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (_, body) = http_get(&srv.url("/"));
    assert!(
        body.contains("editor-toggle"),
        "Should have editor toggle button"
    );
    assert!(body.contains("editor-pane"), "Should have editor pane");
    assert!(
        body.contains("editor-textarea"),
        "Should have editor textarea"
    );
    assert!(
        body.contains("editor-toolbar"),
        "Should have editor toolbar"
    );
    assert!(body.contains("editor-format-bar"), "Should have format bar");
    assert!(body.contains("line-numbers"), "Should have line numbers");
    assert!(body.contains("fmt-btn"), "Should have format buttons");
    assert!(
        body.contains("has-editor"),
        "Body should have has-editor class"
    );

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_format_buttons() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-fmt.md", "# Format Test");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (_, body) = http_get(&srv.url("/"));
    assert!(
        body.contains(r#"data-fmt="bold""#),
        "Should have bold button"
    );
    assert!(
        body.contains(r#"data-fmt="italic""#),
        "Should have italic button"
    );
    assert!(
        body.contains(r#"data-fmt="heading""#),
        "Should have heading button"
    );
    assert!(
        body.contains(r#"data-fmt="strikethrough""#),
        "Should have strikethrough button"
    );
    assert!(
        body.contains(r#"data-fmt="code""#),
        "Should have code button"
    );
    assert!(
        body.contains(r#"data-fmt="link""#),
        "Should have link button"
    );
    assert!(
        body.contains(r#"data-fmt="list""#),
        "Should have list button"
    );
    assert!(
        body.contains(r#"data-fmt="quote""#),
        "Should have quote button"
    );

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_sse_endpoint() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-sse.md", "# SSE Test");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    // SSE endpoint should respond — use a short timeout since the connection stays open.
    // We just verify the endpoint exists and doesn't 404.
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(1)))
        .build()
        .into();
    match agent.get(&srv.url("/events")).call() {
        Ok(_) => {} // Connected — endpoint exists
        Err(ureq::Error::StatusCode(code)) => {
            assert_ne!(code, 404, "SSE endpoint should exist");
        }
        Err(_) => {} // Timeout is expected for SSE
    }

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_custom_port() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-port.md", "# Port Test");
    let srv = start_serve(&["--port", "0", tmp.to_str().unwrap()]);

    let (status, _) = http_get(&srv.url("/"));
    assert_eq!(status, 200);

    let _ = std::fs::remove_file(&tmp);
}

// ── Directory serve ──────────────────────────────────────────────────

#[test]
fn test_serve_directory_index() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-dir-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("alpha.md"), "# Alpha").unwrap();
    std::fs::write(dir.join("beta.md"), "# Beta").unwrap();

    let srv = start_serve(&[dir.to_str().unwrap()]);

    let (status, body) = http_get(&srv.url("/"));
    assert_eq!(status, 200);
    assert!(body.contains("alpha.md"), "Index should list alpha.md");
    assert!(body.contains("beta.md"), "Index should list beta.md");
    assert!(body.contains("file-grid"), "Should have file grid");
    assert!(
        body.contains(r#"id="new-note-card""#),
        "Directory mode should have new note card"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_serve_directory_file_page() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-dir-page");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("readme.md"),
        "# Directory Readme\n\nHello from dir.",
    )
    .unwrap();

    let srv = start_serve(&[dir.to_str().unwrap()]);

    let (status, body) = http_get(&srv.url("/readme.md"));
    assert_eq!(status, 200);
    assert!(
        body.contains("Directory Readme"),
        "Should render file content"
    );
    assert!(body.contains("Hello from dir"), "Should render paragraph");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_serve_directory_file_raw() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-dir-raw");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("doc.md"), "# Doc Raw").unwrap();

    let srv = start_serve(&[dir.to_str().unwrap()]);

    let (status, body) = http_get(&srv.url("/doc.md/raw"));
    assert_eq!(status, 200);
    assert!(
        body.contains("Doc Raw"),
        "Raw endpoint should return HTML fragment"
    );
    assert!(!body.contains("<!DOCTYPE"), "Raw should not be a full page");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_serve_directory_source_get() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-dir-source-get");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("notes.md"), "# My Notes\n\nSome notes.").unwrap();

    let srv = start_serve(&[dir.to_str().unwrap()]);

    let (status, body) = http_get(&srv.url("/notes.md/source"));
    assert_eq!(status, 200);
    assert_eq!(body.trim(), "# My Notes\n\nSome notes.");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_serve_directory_source_put() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-dir-source-put");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("edit.md"), "# Original").unwrap();

    let srv = start_serve(&[dir.to_str().unwrap()]);

    let status = http_put(&srv.url("/edit.md/source"), "# Updated\n\nNew text.");
    assert_eq!(status, 200);

    std::thread::sleep(Duration::from_millis(100));
    let on_disk = std::fs::read_to_string(dir.join("edit.md")).unwrap();
    assert_eq!(on_disk, "# Updated\n\nNew text.");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_serve_directory_create_file() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-dir-create");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("existing.md"), "# Existing").unwrap();

    let srv = start_serve(&[dir.to_str().unwrap()]);

    let (status, body) = http_post(&srv.url("/create"), "new-note");
    assert_eq!(status, 200);
    assert_eq!(body.trim(), "new-note.md");

    // File should exist on disk
    let created = std::fs::read_to_string(dir.join("new-note.md")).unwrap();
    assert!(
        created.contains("# new-note"),
        "Created file should have title heading"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_serve_directory_create_file_with_md_extension() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-dir-create-ext");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("seed.md"), "# Seed").unwrap();

    let srv = start_serve(&[dir.to_str().unwrap()]);

    let (status, body) = http_post(&srv.url("/create"), "readme.md");
    assert_eq!(status, 200);
    assert_eq!(body.trim(), "readme.md");
    assert!(dir.join("readme.md").exists(), "File should be created");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_serve_directory_create_duplicate() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-dir-create-dup");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("exists.md"), "# Exists").unwrap();

    let srv = start_serve(&[dir.to_str().unwrap()]);

    let (status, _) = http_post(&srv.url("/create"), "exists");
    assert_eq!(status, 409, "Creating duplicate should return 409 Conflict");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_serve_directory_create_invalid_filename() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-dir-create-invalid");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("seed.md"), "# Seed").unwrap();

    let srv = start_serve(&[dir.to_str().unwrap()]);

    // Path traversal
    let (status, _) = http_post(&srv.url("/create"), "../escape");
    assert_eq!(status, 400, "Path traversal should be rejected");

    // Slashes
    let (status, _) = http_post(&srv.url("/create"), "sub/dir");
    assert_eq!(status, 400, "Slashes should be rejected");

    // Empty
    let (status, _) = http_post(&srv.url("/create"), "");
    assert_eq!(status, 400, "Empty name should be rejected");

    // Too long
    let long_name = "a".repeat(256);
    let (status, _) = http_post(&srv.url("/create"), &long_name);
    assert_eq!(status, 400, "Too-long name should be rejected");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_serve_directory_create_then_serve() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-dir-create-serve");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("seed.md"), "# Seed").unwrap();

    let srv = start_serve(&[dir.to_str().unwrap()]);

    // Create a new file
    let (status, body) = http_post(&srv.url("/create"), "fresh-note");
    assert_eq!(status, 200);
    assert_eq!(body.trim(), "fresh-note.md");

    // The new file should be serveable
    let (status, body) = http_get(&srv.url("/fresh-note.md"));
    assert_eq!(status, 200);
    assert!(
        body.contains("fresh-note"),
        "New file page should contain its title"
    );

    // The index should now list the new file
    let (status, body) = http_get(&srv.url("/"));
    assert_eq!(status, 200);
    assert!(
        body.contains("fresh-note.md"),
        "Index should list the newly created file"
    );

    // The new file's source should be editable
    let status = http_put(
        &srv.url("/fresh-note.md/source"),
        "# Fresh Note\n\nEdited content.",
    );
    assert_eq!(status, 200);

    std::thread::sleep(Duration::from_millis(100));
    let on_disk = std::fs::read_to_string(dir.join("fresh-note.md")).unwrap();
    assert_eq!(on_disk, "# Fresh Note\n\nEdited content.");

    let _ = std::fs::remove_dir_all(&dir);
}

// ── Multi-file serve ─────────────────────────────────────────────────

#[test]
fn test_serve_multi_file_no_create() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp1 = write_tmp("nc1.md", "# File A");
    let tmp2 = write_tmp("nc2.md", "# File B");

    let srv = start_serve(&[tmp1.to_str().unwrap(), tmp2.to_str().unwrap()]);

    // Index should NOT have new-note-card element in multi-file mode
    // (CSS contains the class name, so check for the actual HTML element id)
    let (_, body) = http_get(&srv.url("/"));
    assert!(
        !body.contains(r#"id="new-note-card""#),
        "Multi-file mode should NOT have new note card element"
    );

    let _ = std::fs::remove_file(&tmp1);
    let _ = std::fs::remove_file(&tmp2);
}

#[test]
fn test_serve_multi_file_index() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp1 = write_tmp("a.md", "# File A");
    let tmp2 = write_tmp("b.md", "# File B");
    let name1 = tmp1.file_name().unwrap().to_str().unwrap().to_string();
    let name2 = tmp2.file_name().unwrap().to_str().unwrap().to_string();

    let srv = start_serve(&[tmp1.to_str().unwrap(), tmp2.to_str().unwrap()]);

    let (status, body) = http_get(&srv.url("/"));
    assert_eq!(status, 200);
    assert!(body.contains(&name1), "Index should list file A");
    assert!(body.contains(&name2), "Index should list file B");

    let _ = std::fs::remove_file(&tmp1);
    let _ = std::fs::remove_file(&tmp2);
}

#[test]
fn test_serve_multi_file_page() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp1 = write_tmp("p1.md", "# Page One\n\nFirst file.");
    let tmp2 = write_tmp("p2.md", "# Page Two\n\nSecond file.");
    let name1 = tmp1.file_name().unwrap().to_str().unwrap().to_string();
    let name2 = tmp2.file_name().unwrap().to_str().unwrap().to_string();

    let srv = start_serve(&[tmp1.to_str().unwrap(), tmp2.to_str().unwrap()]);

    let (status, body) = http_get(&srv.url(&format!("/{}", name1)));
    assert_eq!(status, 200);
    assert!(body.contains("Page One"), "Should render first file");

    let (status, body) = http_get(&srv.url(&format!("/{}", name2)));
    assert_eq!(status, 200);
    assert!(body.contains("Page Two"), "Should render second file");

    let _ = std::fs::remove_file(&tmp1);
    let _ = std::fs::remove_file(&tmp2);
}

#[test]
fn test_serve_multi_file_sidebar_navigation() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp1 = write_tmp("nav1.md", "# Nav One");
    let tmp2 = write_tmp("nav2.md", "# Nav Two");
    let name1 = tmp1.file_name().unwrap().to_str().unwrap().to_string();

    let srv = start_serve(&[tmp1.to_str().unwrap(), tmp2.to_str().unwrap()]);

    let (_, body) = http_get(&srv.url(&format!("/{}", name1)));
    assert!(
        body.contains("back-to-index"),
        "Should have back-to-index link"
    );
    assert!(
        body.contains("file-nav"),
        "Should have file navigation panel"
    );
    assert!(
        body.contains("file-search"),
        "Should have file search input"
    );
    assert!(body.contains("sidebar-tab"), "Should have sidebar tabs");

    let _ = std::fs::remove_file(&tmp1);
    let _ = std::fs::remove_file(&tmp2);
}

// ── HTML output structure ────────────────────────────────────────────

#[test]
fn test_serve_html_has_mermaid() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-mermaid.md", "# Mermaid Test");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (_, body) = http_get(&srv.url("/"));
    assert!(
        body.contains("mermaid") || body.contains("Mermaid"),
        "Should include mermaid script: body len={}",
        body.len()
    );

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_html_has_katex() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-katex.md", "# KaTeX Test");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (_, body) = http_get(&srv.url("/"));
    assert!(
        body.contains("katex") || body.contains("KaTeX"),
        "Should include KaTeX: body len={}",
        body.len()
    );

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_html_has_theme_toggle() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-theme.md", "# Theme Test");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (_, body) = http_get(&srv.url("/"));
    assert!(
        body.contains("theme-toggle"),
        "Should have theme toggle button"
    );
    assert!(
        body.contains("data-theme"),
        "Should have data-theme attribute"
    );

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_html_has_toc_sidebar() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-toc.md", "# Heading\n\n## Sub");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (_, body) = http_get(&srv.url("/"));
    assert!(body.contains("toc-panel"), "Should have ToC panel");
    assert!(
        body.contains("sidebar-toggle"),
        "Should have sidebar toggle"
    );
    assert!(body.contains("progress-bar"), "Should have progress bar");
    assert!(
        body.contains("back-to-top"),
        "Should have back-to-top button"
    );

    let _ = std::fs::remove_file(&tmp);
}

// ── Content negotiation (MFA) ────────────────────────────────────────

#[test]
fn test_serve_content_negotiation_markdown() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-cn-md.md", "# Negotiation Test\n\nMarkdown body.");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (status, body, ct) = http_get_with_accept(&srv.url("/"), "text/markdown");
    assert_eq!(status, 200);
    assert!(
        ct.as_deref().unwrap_or("").contains("text/markdown"),
        "Content-Type should be text/markdown, got: {:?}",
        ct
    );
    assert!(
        !body.contains("<!DOCTYPE"),
        "Markdown response should not contain HTML doctype"
    );
    assert!(
        body.contains("# Negotiation Test"),
        "Should return raw markdown content"
    );

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_content_negotiation_html_default() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-cn-html.md", "# HTML Default\n\nHTML body.");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (status, body, ct) = http_get_with_accept(&srv.url("/"), "text/html");
    assert_eq!(status, 200);
    assert!(
        ct.as_deref().unwrap_or("").contains("text/html"),
        "Content-Type should be text/html, got: {:?}",
        ct
    );
    assert!(
        body.contains("<!DOCTYPE html>"),
        "Should return full HTML page"
    );

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_markdown_has_token_header() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp(
        "md-serve-cn-tokens.md",
        "# Token Test\n\nSome content for tokens.",
    );
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (_status, _body, headers) = http_get_with_headers(&srv.url("/"), "text/markdown");
    let tokens_header = headers
        .get("x-markdown-tokens")
        .and_then(|v| v.parse::<u64>().ok());
    assert!(
        tokens_header.is_some(),
        "X-Markdown-Tokens header should be present"
    );
    assert!(
        tokens_header.unwrap() > 0,
        "Token count should be greater than 0"
    );

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_markdown_has_vary_header() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-cn-vary.md", "# Vary Test\n\nContent.");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (_status, _body, headers) = http_get_with_headers(&srv.url("/"), "text/markdown");
    let vary = headers.get("vary").map(|s| s.as_str()).unwrap_or("");
    assert!(
        vary.contains("Accept"),
        "Vary header should contain Accept, got: {}",
        vary
    );

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_multi_content_negotiation() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-cn-multi");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("agent.md"), "# Agent Page\n\nMulti-file markdown.").unwrap();

    let srv = start_serve(&[dir.to_str().unwrap()]);

    let (status, body, ct) = http_get_with_accept(&srv.url("/agent.md"), "text/markdown");
    assert_eq!(status, 200);
    assert!(
        ct.as_deref().unwrap_or("").contains("text/markdown"),
        "Content-Type should be text/markdown, got: {:?}",
        ct
    );
    assert!(body.contains("# Agent Page"), "Should return raw markdown");
    assert!(!body.contains("<!DOCTYPE"), "Should not contain HTML");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_serve_put_source_rejects_traversal() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let root = std::env::temp_dir().join("md-serve-put-traversal");
    let _ = std::fs::remove_dir_all(&root);
    let served = root.join("served");
    std::fs::create_dir_all(&served).unwrap();
    std::fs::write(served.join("index.md"), "# Put\n").unwrap();

    let srv = start_serve(&[served.to_str().unwrap()]);

    // Percent-encoded and raw forms both decode to a traversal before the
    // handler sees them.
    for probe in ["/%2e%2e%2fescaped.md/source", "/..%2fescaped.md/source"] {
        let agent = http_agent();
        let _ = agent.put(&srv.url(probe)).send("PWNED");
    }

    assert!(
        !root.join("escaped.md").exists(),
        "PUT /{{file}}/source must not write outside the served directory"
    );
    // The legitimate path must still work.
    let code = http_put(&srv.url("/index.md/source"), "# Edited\n");
    assert_eq!(code, 200, "normal source write must still succeed");
    assert_eq!(
        std::fs::read_to_string(served.join("index.md")).unwrap(),
        "# Edited\n"
    );

    let _ = std::fs::remove_dir_all(&root);
}

// ── Raw HTML sanitising ──────────────────────────────────────────────

const XSS_DOC: &str =
    "# Untrusted\n\n<script>alert('pwned')</script>\n\nInline <img src=x onerror=alert(1)> tail.\n";

#[test]
fn test_serve_strips_raw_html_by_default() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-xss.md", XSS_DOC);
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    let (status, body) = http_get(&srv.url("/"));
    assert_eq!(status, 200);
    assert!(body.contains("Untrusted"), "Document should still render");
    assert!(
        !body.contains("alert('pwned')"),
        "A <script> in markdown must not reach the served page"
    );
    assert!(
        !body.contains("onerror=alert(1)"),
        "Inline raw HTML must not reach the served page"
    );

    // /raw has no page chrome, so it is the cleanest place to check the shape.
    let (status, raw) = http_get(&srv.url("/raw"));
    assert_eq!(status, 200);
    assert!(
        raw.contains("<!-- raw HTML omitted -->"),
        "Raw HTML should be replaced by comrak's omitted marker, got: {}",
        raw
    );
    assert!(
        raw.contains("Inline") && raw.contains("tail."),
        "Text around the stripped tag should survive, got: {}",
        raw
    );

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_unsafe_html_renders_raw_html() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-unsafe-html.md", XSS_DOC);
    let srv = start_serve(&["--unsafe-html", tmp.to_str().unwrap()]);

    let (status, raw) = http_get(&srv.url("/raw"));
    assert_eq!(status, 200);
    // GFM's tagfilter stays on, so --unsafe-html renders raw HTML the way
    // GitHub does: ordinary tags pass, script is neutralised.
    assert!(
        raw.contains("<img src=x onerror=alert(1)>"),
        "--unsafe-html should pass ordinary raw HTML through, got: {}",
        raw
    );
    assert!(
        raw.contains("&lt;script>"),
        "tagfilter should neutralise script even with --unsafe-html, got: {}",
        raw
    );
    assert!(
        !raw.contains("<!-- raw HTML omitted -->"),
        "--unsafe-html should omit nothing, got: {}",
        raw
    );

    let _ = std::fs::remove_file(&tmp);
}

// ── Bind address ─────────────────────────────────────────────────────

#[test]
fn test_serve_default_bind_is_loopback_only() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-bind.md", "# Bind Test");
    let srv = start_serve(&[tmp.to_str().unwrap()]);

    // Positive control: the server is up, on loopback.
    let (status, _) = http_get(&srv.url("/"));
    assert_eq!(status, 200, "Server should be reachable on 127.0.0.1");

    // Same server, same port, addressed by this machine's LAN IP.
    match lan_ipv4() {
        Some(ip) => {
            let addr = std::net::SocketAddr::new(ip, srv.port);
            let err = std::net::TcpStream::connect_timeout(&addr, Duration::from_secs(2))
                .err()
                .map(|e| e.kind());
            assert!(
                err.is_some(),
                "Default bind must not accept connections at {}",
                addr
            );
            // A timeout means the packet was dropped by a firewall, which says
            // nothing about what the process bound. Windows CI does this. Skip
            // rather than claim a pass, and skip rather than fail the build on
            // a property of the network.
            if err == Some(std::io::ErrorKind::TimedOut) {
                eprintln!(
                    "  Inconclusive: {} timed out (firewall DROP), not asserting",
                    addr
                );
            }
        }
        None => eprintln!("  Skipping LAN check: no non-loopback IPv4 on this machine"),
    }

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_banner_hides_network_url_by_default() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-banner.md", "# Banner Test");
    let (_srv, banner) = start_serve_with_banner(&[tmp.to_str().unwrap()]);
    let text = banner.join("\n");

    assert!(
        text.contains("Local:   http://127.0.0.1:"),
        "Banner should advertise the loopback URL, got: {}",
        text
    );
    assert!(
        !text.contains("Network:"),
        "A loopback bind must not advertise a LAN URL, got: {}",
        text
    );

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_host_wildcard_warns_about_exposure() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = write_tmp("md-serve-host.md", "# Host Test");
    let (srv, banner) = start_serve_with_banner(&["--host", "0.0.0.0", tmp.to_str().unwrap()]);
    let text = banner.join("\n");

    assert!(
        text.contains("Warning:"),
        "--host 0.0.0.0 must warn that the server is unauthenticated, got: {}",
        text
    );

    // And it is still reachable over loopback.
    let (status, _) = http_get(&srv.url("/"));
    assert_eq!(status, 200);

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_serve_rejects_invalid_host() {
    let out = Command::new(env!("CARGO_BIN_EXE_mdx"))
        .args(["serve", "--host", "not-an-ip"])
        .arg(write_tmp("md-serve-badhost.md", "# x").to_str().unwrap())
        .output()
        .expect("Failed to run mdx serve");
    assert!(!out.status.success(), "Invalid --host should exit non-zero");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Invalid --host"),
        "Should explain the bad host, got: {}",
        stderr
    );
}

// ── Static assets ────────────────────────────────────────────────────

#[test]
fn test_serve_static_asset_single_file_mode() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-static-single");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    let note = dir.join("note.md");
    std::fs::write(&note, "# Shot\n\n![](assets/shot.png)\n").unwrap();
    std::fs::write(dir.join("assets").join("shot.png"), TINY_PNG).unwrap();

    let srv = start_serve(&[note.to_str().unwrap()]);

    let (status, body, ct) = http_get_bytes(&srv.url("/assets/shot.png"));
    assert_eq!(status, 200, "asset next to the served file should resolve");
    assert_eq!(
        ct.as_deref().unwrap_or(""),
        "image/png",
        "Content-Type should be image/png, got: {:?}",
        ct
    );
    assert_eq!(body, TINY_PNG, "image bytes should round-trip unchanged");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_serve_static_asset_directory_mode() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-static-dir");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(dir.join("index.md"), "# Dir\n\n![](logo.png)\n").unwrap();
    std::fs::write(dir.join("logo.png"), TINY_PNG).unwrap();
    std::fs::write(dir.join("assets").join("photo.jpg"), TINY_PNG).unwrap();
    std::fs::write(dir.join("secrets.txt"), "TOP-SECRET-MARKER").unwrap();

    let srv = start_serve(&[dir.to_str().unwrap()]);

    // A root-level image also matches the /{file} markdown route, so this
    // asserts the markdown route falls through to the static handler.
    let (status, body, ct) = http_get_bytes(&srv.url("/logo.png"));
    assert_eq!(status, 200, "root-level image should be served");
    assert_eq!(ct.as_deref().unwrap_or(""), "image/png");
    assert_eq!(body, TINY_PNG, "image bytes should round-trip unchanged");

    // Nested asset: the path the drag-and-drop uploader produces.
    let (status, _, ct) = http_get_bytes(&srv.url("/assets/photo.jpg"));
    assert_eq!(status, 200, "nested asset should be served");
    assert_eq!(ct.as_deref().unwrap_or(""), "image/jpeg");

    // The markdown route still wins for markdown.
    let (status, page) = http_get(&srv.url("/index.md"));
    assert_eq!(status, 200);
    assert!(
        page.contains("<!DOCTYPE html>"),
        "markdown route must not be shadowed by the static handler"
    );

    // Non-asset files in the served directory are not published.
    let (status, body, _) = http_get_bytes(&srv.url("/secrets.txt"));
    assert_ne!(status, 200, "non-asset extensions must not be served");
    assert!(
        !String::from_utf8_lossy(&body).contains("TOP-SECRET-MARKER"),
        "non-asset file content must not leak"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_serve_static_rejects_path_traversal() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let root = std::env::temp_dir().join("md-serve-static-traversal");
    let _ = std::fs::remove_dir_all(&root);
    let served = root.join("served");
    std::fs::create_dir_all(served.join("assets")).unwrap();
    std::fs::write(served.join("index.md"), "# Traversal\n").unwrap();
    // The target lives OUTSIDE the served directory, and uses an
    // allowlisted extension so only the containment check can stop it.
    std::fs::write(root.join("secret.png"), "TOP-SECRET-MARKER").unwrap();

    let srv = start_serve(&[served.to_str().unwrap()]);

    // Percent-encoded so no HTTP client normalises the `..` away before
    // the request reaches the server. The first form is a single raw path
    // segment, so it is routed through /{file} and must still be refused
    // after axum percent-decodes it.
    let attempts = [
        "/%2e%2e%2fsecret.png",
        "/assets/%2e%2e%2f%2e%2e%2fsecret.png",
        "/assets/..%2f..%2fsecret.png",
        "/%2e%2e%5csecret.png",
    ];
    for attempt in attempts {
        let (status, body, _) = http_get_bytes(&srv.url(attempt));
        assert_ne!(status, 200, "traversal {} should not succeed", attempt);
        assert!(
            !String::from_utf8_lossy(&body).contains("TOP-SECRET-MARKER"),
            "traversal {} leaked a file outside the served directory",
            attempt
        );
    }

    let _ = std::fs::remove_dir_all(&root);
}

#[cfg(unix)]
#[test]
fn test_serve_static_rejects_symlink_escape() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let root = std::env::temp_dir().join("md-serve-static-symlink");
    let _ = std::fs::remove_dir_all(&root);
    let served = root.join("served");
    std::fs::create_dir_all(&served).unwrap();
    std::fs::write(served.join("index.md"), "# Symlink\n").unwrap();
    std::fs::write(root.join("outside.png"), "OUTSIDE-MARKER").unwrap();
    std::os::unix::fs::symlink(root.join("outside.png"), served.join("leak.png")).unwrap();

    let srv = start_serve(&[served.to_str().unwrap()]);

    let (status, body, _) = http_get_bytes(&srv.url("/leak.png"));
    assert_ne!(
        status, 200,
        "a symlink pointing outside the served directory must not be served"
    );
    assert!(
        !String::from_utf8_lossy(&body).contains("OUTSIDE-MARKER"),
        "symlink escape leaked a file outside the served directory"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_serve_upload_then_fetch_round_trip() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("md-serve-static-upload");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let note = dir.join("note.md");
    std::fs::write(&note, "# Upload\n").unwrap();

    let srv = start_serve(&[note.to_str().unwrap()]);

    let (status, json) = http_post_multipart_png(&srv.url("/upload"), "shot.png", TINY_PNG);
    assert_eq!(status, 200, "upload should succeed, got body: {}", json);
    assert!(
        json.contains("\"path\":\"assets/"),
        "upload should return an assets/ path, got: {}",
        json
    );

    // Fetch exactly the URL the editor inserts as ![](<path>).
    let rel = json
        .split("\"path\":\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .expect("upload response should contain a path")
        .to_string();

    let (status, body, ct) = http_get_bytes(&srv.url(&format!("/{}", rel)));
    assert_eq!(status, 200, "uploaded image at /{} should be served", rel);
    assert_eq!(ct.as_deref().unwrap_or(""), "image/png");
    assert_eq!(body, TINY_PNG, "served bytes should match the upload");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_serve_upload_rejects_traversal_filename() {
    let _guard = SERVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let root = std::env::temp_dir().join("md-serve-static-upload-evil");
    let _ = std::fs::remove_dir_all(&root);
    let served = root.join("served");
    std::fs::create_dir_all(&served).unwrap();
    let note = served.join("note.md");
    std::fs::write(&note, "# Upload\n").unwrap();

    let srv = start_serve(&[note.to_str().unwrap()]);

    let (status, json) = http_post_multipart_png(&srv.url("/upload"), "../../evil.png", TINY_PNG);
    assert_eq!(status, 200, "upload should still succeed, got: {}", json);
    assert!(
        !root.join("evil.png").exists() && !served.join("evil.png").exists(),
        "upload must not write outside the assets directory"
    );
    assert!(
        served.join("assets").join("evil.png").exists(),
        "upload should be stored under assets/ with the path stripped"
    );

    let _ = std::fs::remove_dir_all(&root);
}
