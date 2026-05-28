use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::thread;

use clap::Parser;
use faf_browser::api::commands::{Cli, run};

// ---------------------------------------------------------------------------
// T033 — REPL e stdin
// ---------------------------------------------------------------------------

/// Servidor local que responde com HTML contendo um título e elementos.
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

#[test]
fn test_stdin_mode() {
    let html = r#"<html><head><title>Stdin Test</title></head><body><h1>Hello</h1></body></html>"#;
    let port = start_server_with_html(html);

    let mut child = Command::new("cargo")
        .args([
            "run",
            "--",
            &format!("http://127.0.0.1:{}/", port),
            "--stdin",
            "--no-scripts",
        ])
        .current_dir("/home/hermes/faf-browser")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn cargo run");

    {
        let stdin = child.stdin.take().unwrap();
        let mut stdin = std::io::BufWriter::new(stdin);
        stdin.write_all(b"document.title\n").unwrap();
        stdin.write_all(b"document.querySelector('h1').text\n").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "stdin mode should exit successfully. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("Stdin Test"),
        "expected document.title in stdout, got: {}",
        stdout
    );
    assert!(
        stdout.contains("Hello"),
        "expected h1 text in stdout, got: {}",
        stdout
    );
}

#[test]
fn test_repl_mode() {
    let html = r#"<html><head><title>REPL Test</title></head><body><h1>World</h1></body></html>"#;
    let port = start_server_with_html(html);

    let mut child = Command::new("cargo")
        .args([
            "run",
            "--",
            "repl",
            "--url",
            &format!("http://127.0.0.1:{}/", port),
        ])
        .current_dir("/home/hermes/faf-browser")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn cargo run");

    {
        let stdin = child.stdin.take().unwrap();
        let mut stdin = std::io::BufWriter::new(stdin);
        stdin.write_all(b"document.title\n").unwrap();
        stdin.write_all(b"document.querySelector('h1').text\n").unwrap();
        stdin.write_all(b".exit\n").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "repl mode should exit successfully. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("REPL Test"),
        "expected document.title in stdout, got: {}",
        stdout
    );
    assert!(
        stdout.contains("World"),
        "expected h1 text in stdout, got: {}",
        stdout
    );
    assert!(
        stdout.contains("FAF REPL"),
        "expected REPL banner, got: {}",
        stdout
    );
}

#[test]
fn test_repl_json_toggle() {
    let html = r#"<html><head><title>JSON Toggle</title></head><body></body></html>"#;
    let port = start_server_with_html(html);

    let mut child = Command::new("cargo")
        .args([
            "run",
            "--",
            "repl",
            "--url",
            &format!("http://127.0.0.1:{}/", port),
        ])
        .current_dir("/home/hermes/faf-browser")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn cargo run");

    {
        let stdin = child.stdin.take().unwrap();
        let mut stdin = std::io::BufWriter::new(stdin);
        stdin.write_all(b".json\n").unwrap();
        stdin.write_all(b"document.title\n").unwrap();
        stdin.write_all(b".exit\n").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "repl json toggle should exit successfully. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("JSON Toggle"),
        "expected JSON stringified title, got: {}",
        stdout
    );
}

// ---------------------------------------------------------------------------
// T039 — Click via dispatchEvent
// ---------------------------------------------------------------------------

/// Servidor com botão que tem onclick inline.
fn start_click_button_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for _ in 0..5 {
            let (mut stream, _) = match listener.accept() {
                Ok(c) => c,
                Err(_) => break,
            };
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let body = r#"<html><head><title>Click Test</title></head><body><button id="btn" onclick="this.clicked = true">Clique</button></body></html>"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

#[tokio::test]
async fn test_click_subcommand_success() {
    let port = start_click_button_server();

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "click",
        "#btn",
    ]);

    let result = run(cli).await;
    assert!(result.is_ok(), "click subcommand should succeed: {:?}", result);
}

#[tokio::test]
async fn test_click_subcommand_not_found() {
    let port = start_click_button_server();

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "click",
        ".missing",
    ]);

    let result = run(cli).await;
    assert!(result.is_err(), "click on missing element should fail");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("não encontrado") || err.contains("not found"),
        "expected 'not found' error, got: {}",
        err
    );
}

#[tokio::test]
async fn test_click_js_bridge() {
    let port = start_click_button_server();

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--no-scripts",
        "--js",
        r#"var btn = document.querySelector('#btn'); btn.click(); btn.clicked"#,
    ]);

    let result = run(cli).await;
    assert!(result.is_ok(), "JS click bridge should succeed: {:?}", result);
}

#[tokio::test]
async fn test_click_json_output() {
    let port = start_click_button_server();

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--json",
        "click",
        "#btn",
    ]);

    let result = run(cli).await;
    assert!(result.is_ok(), "click --json should succeed: {:?}", result);
}

#[test]
fn test_click_via_stdin() {
    let port = start_click_button_server();

    let mut child = Command::new("cargo")
        .args([
            "run",
            "--",
            &format!("http://127.0.0.1:{}/", port),
            "--stdin",
            "--no-scripts",
        ])
        .current_dir("/home/hermes/faf-browser")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn cargo run");

    {
        let stdin = child.stdin.take().unwrap();
        let mut stdin = std::io::BufWriter::new(stdin);
        stdin.write_all(b"document.querySelector('#btn').click()\n").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "click via stdin should exit successfully. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}
