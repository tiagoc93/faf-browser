use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::thread;

use clap::Parser;
use faf_browser::api::commands::{run, Cli};

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

fn faf_binary() -> std::path::PathBuf {
    let exe = std::env::current_exe().unwrap();
    let target_dir = exe.parent().unwrap().parent().unwrap();
    target_dir.join("faf-browser")
}

#[test]
fn test_stdin_mode() {
    let html =
        r#"<html><head><title>Stdin Test</title></head><body><h1>Hello</h1></body></html>"#;
    let port = start_server_with_html(html);

    let mut child = Command::new(faf_binary())
        .args([
            &format!("http://127.0.0.1:{}/", port),
            "--stdin",
            "--no-scripts",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn faf");

    {
        let stdin = child.stdin.take().unwrap();
        let mut stdin = std::io::BufWriter::new(stdin);
        stdin.write_all(b"document.title\n").unwrap();
        stdin
            .write_all(b"document.querySelector('h1').text\n")
            .unwrap();
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
    let html =
        r#"<html><head><title>REPL Test</title></head><body><h1>World</h1></body></html>"#;
    let port = start_server_with_html(html);

    let mut child = Command::new(faf_binary())
        .args([
            "repl",
            "--url",
            &format!("http://127.0.0.1:{}/", port),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn faf");

    {
        let stdin = child.stdin.take().unwrap();
        let mut stdin = std::io::BufWriter::new(stdin);
        stdin.write_all(b"document.title\n").unwrap();
        stdin
            .write_all(b"document.querySelector('h1').text\n")
            .unwrap();
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

    let mut child = Command::new(faf_binary())
        .args([
            "repl",
            "--url",
            &format!("http://127.0.0.1:{}/", port),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn faf");

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
    assert!(
        result.is_ok(),
        "click subcommand should succeed: {:?}",
        result
    );
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
    assert!(
        result.is_ok(),
        "JS click bridge should succeed: {:?}",
        result
    );
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
    let mut child = Command::new(faf_binary())
        .args([
            &format!("http://127.0.0.1:{}/", port),
            "--stdin",
            "--no-scripts",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn faf");

    {
        let stdin = child.stdin.take().unwrap();
        let mut stdin = std::io::BufWriter::new(stdin);
        stdin
            .write_all(b"document.querySelector('#btn').click()\n")
            .unwrap();
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

fn start_form_server() -> u16 {
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
            let body = r#"<html><body>
                <form id="login" action="/login" method="POST">
                    <input id="email" name="email" value="default@example.com">
                    <select id="country" name="country">
                        <option value="US">USA</option>
                        <option value="BR">Brasil</option>
                    </select>
                    <input id="agree" name="agree" type="checkbox">
                    <button type="submit">Submit</button>
                </form>
            </body></html>"#;
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
async fn test_fill_input() {
    let port = start_form_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--no-scripts",
        "--js",
        r#"var el = document.querySelector('#email'); el.value = 'user@test.com'; if (el.value !== 'user@test.com') throw new Error('value mismatch: ' + el.value); 'ok'"#,
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "fill input should succeed: {:?}", result);
}

#[tokio::test]
async fn test_select_option() {
    let port = start_form_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--no-scripts",
        "--js",
        r#"var el = document.querySelector('#country'); el.value = 'BR'; if (el.value !== 'BR') throw new Error('value mismatch: ' + el.value); 'ok'"#,
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "select option should succeed: {:?}", result);
}

#[tokio::test]
async fn test_checkbox() {
    let port = start_form_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--no-scripts",
        "--js",
        r#"var el = document.querySelector('#agree'); el.checked = true; if (el.checked !== true) throw new Error('checked mismatch: ' + el.checked); 'ok'"#,
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "checkbox should succeed: {:?}", result);
}

fn start_basic_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for _ in 0..5 {
            let (mut stream, _) = match listener.accept() {
                Ok(c) => c,
                Err(_) => break,
            };
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let body =
                "<html><body><h1>Scroll Test</h1><div id='target'>Target</div></body></html>";
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
async fn test_scroll_to() {
    let port = start_basic_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--no-scripts",
        "--js",
        "window.scrollTo(0, 500); window.pageYOffset",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "scrollTo should succeed: {:?}", result);
}

#[tokio::test]
async fn test_scroll_by() {
    let port = start_basic_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--no-scripts",
        "--js",
        "window.scrollBy(0, 200); window.scrollY",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "scrollBy should succeed: {:?}", result);
}

use std::sync::atomic::{AtomicU64, Ordering};

fn start_watch_changing_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let counter = std::sync::Arc::new(AtomicU64::new(0));
    let c = counter.clone();
    thread::spawn(move || {
        for _ in 0..4 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let count = c.fetch_add(1, Ordering::SeqCst);
            let body = format!(
                "<html><body><h1>Valor: {}</h1></body></html>",
                count
            );
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
async fn test_watch_detect_change() {
    let port = start_watch_changing_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "watch",
        "h1",
        "--interval",
        "1",
        "--max-checks",
        "3",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "watch should succeed: {:?}", result);
}

#[tokio::test]
async fn test_watch_max_checks() {
    let port = start_watch_changing_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "watch",
        "h1",
        "--interval",
        "1",
        "--max-checks",
        "2",
    ]);
    let result = run(cli).await;
    assert!(
        result.is_ok(),
        "watch with max-checks should succeed: {:?}",
        result
    );
}

#[tokio::test]
async fn test_watch_json_output() {
    let port = start_watch_changing_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--json",
        "watch",
        "h1",
        "--interval",
        "1",
        "--max-checks",
        "2",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "watch --json should succeed: {:?}", result);
}

fn start_disappearing_element_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let counter = std::sync::Arc::new(AtomicU64::new(0));
    let c = counter.clone();
    thread::spawn(move || {
        for _ in 0..5 {
            let (mut stream, _) = match listener.accept() {
                Ok(c) => c,
                Err(_) => break,
            };
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let count = c.fetch_add(1, Ordering::SeqCst);
            let body = if count == 0 {
                r#"<html><body><button id="btn" onclick="window.navigated = true">Click</button></body></html>"#.to_string()
            } else {
                r#"<html><body><h1>Navigated</h1></body></html>"#.to_string()
            };
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
async fn test_click_disappearing_element() {
    let port = start_disappearing_element_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "click",
        "#btn",
    ]);
    let result = run(cli).await;
    assert!(
        result.is_ok(),
        "click on disappearing element should succeed: {:?}",
        result
    );
}

#[tokio::test]
async fn test_watch_interval_zero() {
    let port = start_watch_changing_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "watch",
        "h1",
        "--interval",
        "0",
        "--max-checks",
        "3",
    ]);
    let result = run(cli).await;
    assert!(
        result.is_ok(),
        "watch with interval 0 should succeed: {:?}",
        result
    );
}

#[tokio::test]
async fn test_scroll_clamp_negative() {
    let port = start_basic_server();
    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--no-scripts",
        "--js",
        "window.scrollTo(0, -9999); window.pageYOffset",
    ]);
    let result = run(cli).await;
    assert!(result.is_ok(), "scroll clamp should succeed: {:?}", result);
}
