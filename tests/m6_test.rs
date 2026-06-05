use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use clap::Parser;
use faf_browser::api::commands::{run, Cli};

fn start_server_with_html(html: &'static str) -> u16 {
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

#[tokio::test]
async fn test_dump_basic() {
    let html = r##"<!DOCTYPE html><html><head><title>Test</title></head><body><h1>Hello</h1></body></html>"##;
    let port = start_server_with_html(html);
    let output = "/tmp/faf_test_dump_basic_result.html";

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        output,
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "dump should succeed: {:?}", result);

    let saved = std::fs::read_to_string(output).unwrap();
    assert!(saved.contains("Hello"));
    assert!(saved.contains("<h1>"));
    let _ = std::fs::remove_file(output);
}

#[tokio::test]
async fn test_dump_resolves_relative_urls() {
    let html = r##"<!DOCTYPE html><html><body><a href="/about">About</a><img src="/img/photo.png"></body></html>"##;
    let port = start_server_with_html(html);
    let output = "/tmp/faf_test_dump_urls.html";

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        output,
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "dump should succeed: {:?}", result);

    let saved = std::fs::read_to_string(output).unwrap();
    let expected = format!("http://127.0.0.1:{}/about", port);
    assert!(
        saved.contains(&expected),
        "Expected absolute URL containing {}",
        expected
    );
    let _ = std::fs::remove_file(output);
}

#[tokio::test]
async fn test_dump_no_scripts() {
    let html = r##"<html><head><script>alert('xss')</script></head><body><button onclick="evil()">Click</button></body></html>"##;
    let port = start_server_with_html(html);
    let output = "/tmp/faf_test_dump_no_scripts.html";

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        output,
        "--no-scripts",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "dump should succeed: {:?}", result);

    let saved = std::fs::read_to_string(output).unwrap();
    assert!(!saved.contains("alert"));
    assert!(!saved.contains("onclick"));
    assert!(saved.contains("<button"));
    let _ = std::fs::remove_file(output);
}

#[tokio::test]
async fn test_dump_preserves_existing_styles() {
    let html = r##"<!DOCTYPE html><html><head><style>.red { color: red; }</style></head><body><h1 class="red">Red</h1></body></html>"##;
    let port = start_server_with_html(html);
    let output = "/tmp/faf_test_dump_styles.html";

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "dump",
        "--output",
        output,
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "dump should succeed: {:?}", result);

    let saved = std::fs::read_to_string(output).unwrap();
    assert!(saved.contains(".red { color: red; }"));
    assert!(saved.contains("<style>"));
    let _ = std::fs::remove_file(output);
}
