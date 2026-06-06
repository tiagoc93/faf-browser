use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use clap::Parser;
use faf_browser::api::commands::{run, Cli};

fn start_server(html: &'static str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            html.len(),
            html
        );
        let _ = stream.write_all(response.as_bytes());
    });
    port
}

fn output_path(name: &str) -> String {
    format!("/tmp/faf_test_m7_{}_{}.out", name, std::process::id())
}

// ── T090: Markdown ──────────────────────────────────────────────

#[tokio::test]
async fn test_markdown_h1() {
    let html = "<html><body><h1>Hello World</h1></body></html>";
    let port = start_server(html);
    let output = output_path("md_h1");

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        &output,
        "--format",
        "markdown",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "dump markdown should succeed: {:?}", result);

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(saved.contains("# Hello World"), "got: {}", saved);
    let _ = std::fs::remove_file(&output);
}

#[tokio::test]
async fn test_markdown_link() {
    let html = r##"<html><body><a href="https://example.com">Example</a></body></html>"##;
    let port = start_server(html);
    let output = output_path("md_link");

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        &output,
        "--format",
        "markdown",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "dump markdown should succeed: {:?}", result);

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(
        saved.contains("[Example](https://example.com)"),
        "got: {}",
        saved
    );
    let _ = std::fs::remove_file(&output);
}

#[tokio::test]
async fn test_markdown_list() {
    let html = "<html><body><ul><li>Item A</li><li>Item B</li></ul></body></html>";
    let port = start_server(html);
    let output = output_path("md_list");

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        &output,
        "--format",
        "markdown",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "dump markdown should succeed: {:?}", result);

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(saved.contains("- Item A"), "got: {}", saved);
    assert!(saved.contains("- Item B"), "got: {}", saved);
    let _ = std::fs::remove_file(&output);
}

#[tokio::test]
async fn test_markdown_skips_script() {
    let html = "<html><body><script>alert(1)</script><p>Safe content</p></body></html>";
    let port = start_server(html);
    let output = output_path("md_noscript");

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        &output,
        "--format",
        "markdown",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "dump markdown should succeed: {:?}", result);

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(!saved.contains("alert"), "script content leaked: {}", saved);
    assert!(saved.contains("Safe content"), "got: {}", saved);
    let _ = std::fs::remove_file(&output);
}

#[tokio::test]
async fn test_markdown_image() {
    let html = r##"<html><body><img src="photo.png" alt="A photo"></body></html>"##;
    let port = start_server(html);
    let output = output_path("md_img");

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        &output,
        "--format",
        "markdown",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "dump markdown should succeed: {:?}", result);

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(saved.contains("![A photo]("), "got: {}", saved);
    assert!(saved.contains("photo.png"), "got: {}", saved);
    let _ = std::fs::remove_file(&output);
}

// ── T091: Readability ───────────────────────────────────────────

#[tokio::test]
async fn test_readability_removes_nav() {
    let html = "<html><body><nav><a href=\"/\">Home</a></nav><article><h1>Title</h1><p>Content text that should be extracted.</p></article></body></html>";
    let port = start_server(html);
    let output = output_path("rb_nav");

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        &output,
        "--readability",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "readability should succeed: {:?}", result);

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(!saved.contains("Home"), "nav should not appear: {}", saved);
    assert!(saved.contains("Title"), "got: {}", saved);
    let _ = std::fs::remove_file(&output);
}

#[tokio::test]
async fn test_readability_preserves_article() {
    let html = "<html><body><article><h1>Main</h1><p>Body one.</p><p>Body two.</p><p>Body three.</p><p>Body four.</p></article></body></html>";
    let port = start_server(html);
    let output = output_path("rb_art");

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        &output,
        "--readability",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "readability should succeed: {:?}", result);

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(saved.contains("Main"), "got: {}", saved);
    let _ = std::fs::remove_file(&output);
}

// ── T092: Structured Data ───────────────────────────────────────

#[tokio::test]
async fn test_structured_data_json_ld() {
    let html = r##"<html><head><script type="application/ld+json">{"@type": "Product", "name": "Widget"}</script></head></html>"##;
    let port = start_server(html);
    let output = output_path("sd_ld");

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        &output,
        "--structured-data",
    ]);
    let result = run(cli).await;
    assert!(
        result.is_ok(),
        "structured data should succeed: {:?}",
        result
    );

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(saved.contains("Product"), "got: {}", saved);
    assert!(saved.contains("Widget"), "got: {}", saved);
    let _ = std::fs::remove_file(&output);
}

#[tokio::test]
async fn test_structured_data_open_graph() {
    let html =
        r##"<html><head><meta property="og:title" content="OG Title"></head></html>"##;
    let port = start_server(html);
    let output = output_path("sd_og");

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        &output,
        "--structured-data",
    ]);
    let result = run(cli).await;
    assert!(
        result.is_ok(),
        "structured data should succeed: {:?}",
        result
    );

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(saved.contains("OG Title"), "got: {}", saved);
    let _ = std::fs::remove_file(&output);
}

#[tokio::test]
async fn test_structured_data_meta() {
    let html =
        r##"<html><head><meta name="description" content="Meta desc"></head></html>"##;
    let port = start_server(html);
    let output = output_path("sd_meta");

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        &output,
        "--structured-data",
    ]);
    let result = run(cli).await;
    assert!(
        result.is_ok(),
        "structured data should succeed: {:?}",
        result
    );

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(saved.contains("Meta desc"), "got: {}", saved);
    let _ = std::fs::remove_file(&output);
}

// ── T093: Text ──────────────────────────────────────────────────

#[tokio::test]
async fn test_text_output_paragraphs() {
    let html = "<html><body><p>First paragraph.</p><p>Second paragraph.</p></body></html>";
    let port = start_server(html);
    let output = output_path("txt_par");

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        &output,
        "--format",
        "text",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "text dump should succeed: {:?}", result);

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(saved.contains("First paragraph"), "got: {}", saved);
    assert!(saved.contains("Second paragraph"), "got: {}", saved);
    assert!(!saved.contains("<p>"), "HTML tag leaked: {}", saved);
    let _ = std::fs::remove_file(&output);
}

#[tokio::test]
async fn test_text_output_skips_nav() {
    let html = "<html><body><nav>Menu link</nav><p>Content.</p></body></html>";
    let port = start_server(html);
    let output = output_path("txt_nav");

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        &output,
        "--format",
        "text",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "text dump should succeed: {:?}", result);

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(!saved.contains("Menu link"), "nav leaked: {}", saved);
    assert!(saved.contains("Content"), "got: {}", saved);
    let _ = std::fs::remove_file(&output);
}

// ── T096: Inline Images ─────────────────────────────────────────

fn minimal_png_bytes() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41,
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
        0x00, 0x00, 0x03, 0x00, 0x01, 0x1A, 0x72, 0x5C,
        0xD4, 0x74, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
        0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

fn start_server_with_image() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for _ in 0..3 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap();
            let request = String::from_utf8_lossy(&buf[..n]);

            if request.contains("GET /img/photo.png") {
                let png = minimal_png_bytes();
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    png.len()
                );
                stream.write_all(header.as_bytes()).unwrap();
                stream.write_all(&png).unwrap();
            } else {
                let html = "<html><body><img src=\"/img/photo.png\" alt=\"Test\"></body></html>";
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    html.len(),
                    html
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
        }
    });
    port
}

#[test]
fn test_inline_images_replaces_src_with_data_uri() {
    use std::process::Command;

    let port = start_server_with_image();
    let output = format!("/tmp/faf_test_inline_img_{}.html", std::process::id());

    let exe = std::env::current_exe().unwrap();
    let target_dir = exe.parent().unwrap().parent().unwrap();
    let faf_bin = target_dir.join("faf-browser");

    let child = Command::new(&faf_bin)
        .args([
            "dump",
            "--url",
            &format!("http://127.0.0.1:{}/", port),
            "--output",
            &output,
            "--inline-images",
            "--no-inline-css",
            "--no-scripts",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn faf");

    let out = child.wait_with_output().unwrap();

    assert!(
        out.status.success(),
        "dump with inline-images should succeed. stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let saved = std::fs::read_to_string(&output).unwrap();
    assert!(
        saved.contains("data:image/png;base64,"),
        "expected data URI, got first 500 chars: {}",
        &saved[..saved.len().min(500)]
    );
    assert!(
        !saved.contains("/img/photo.png"),
        "original src should be replaced"
    );
    let _ = std::fs::remove_file(&output);
}
