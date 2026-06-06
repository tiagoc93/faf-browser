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
