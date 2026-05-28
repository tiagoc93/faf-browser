use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::thread;

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
