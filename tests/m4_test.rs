use clap::Parser;
use faf_browser::api::commands::{Cli, run};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

// ---------------------------------------------------------------------------
// T032 — WaitForSelector
// ---------------------------------------------------------------------------

/// Servidor local que responde com HTML contendo o elemento buscado.
fn start_server_with_html(html: &'static str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 1024];
        let _ = stream.read(&mut buf);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
            html.len(),
            html
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    port
}

#[tokio::test]
async fn test_wait_for_selector_found() {
    let html = r#"<html><head><title>Wait Test</title></head><body><div id="app" class="loaded">Ready</div></body></html>"#;
    let port = start_server_with_html(html);

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "wait",
        "#app",
        "--timeout",
        "2",
        "--interval",
        "100",
    ]);

    let result = run(cli).await;
    assert!(result.is_ok(), "wait should find element immediately: {:?}", result);
}

#[tokio::test]
async fn test_wait_for_selector_found_json() {
    let html = r#"<html><head><title>Wait Test</title></head><body><div id="app" class="loaded">Ready</div></body></html>"#;
    let port = start_server_with_html(html);

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--json",
        "wait",
        ".loaded",
        "--timeout",
        "2",
        "--interval",
        "100",
    ]);

    let result = run(cli).await;
    assert!(result.is_ok(), "wait --json should find element: {:?}", result);
}

#[tokio::test]
async fn test_wait_for_selector_timeout() {
    let html = r#"<html><head><title>Wait Test</title></head><body><div id="other">Not it</div></body></html>"#;
    let port = start_server_with_html(html);

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "wait",
        "#missing",
        "--timeout",
        "1",
        "--interval",
        "200",
    ]);

    let result = run(cli).await;
    assert!(result.is_err(), "wait should timeout when element is missing");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("não encontrado") || err.contains("not found"),
        "expected timeout error, got: {}",
        err
    );
}

#[tokio::test]
async fn test_wait_for_selector_invalid_selector() {
    let html = r#"<html><head><title>Wait Test</title></head><body></body></html>"#;
    let port = start_server_with_html(html);

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "wait",
        "[[[bad",
        "--timeout",
        "1",
        "--interval",
        "200",
    ]);

    let result = run(cli).await;
    assert!(result.is_err(), "wait should fail on invalid selector");
}
