use clap::Parser;
use faf_browser::api::commands::{Cli, run};
use faf_browser::dom::HtmlDocument;
use faf_browser::js::JsRuntime;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

// ---------------------------------------------------------------------------
// 1. JS básico
// ---------------------------------------------------------------------------
#[test]
fn test_js_basic() {
    let rt = JsRuntime::new().unwrap();
    let result = rt.eval("1 + 2").unwrap();
    assert_eq!(result, "3");
}

// ---------------------------------------------------------------------------
// 2. DOM bridge
// ---------------------------------------------------------------------------
#[test]
fn test_dom_bridge() {
    let html = r#"<div id="main"><h1>Hello</h1></div>"#;
    let doc = HtmlDocument::parse(html);
    let mut rt = JsRuntime::new().unwrap();
    rt.set_dom(&doc).unwrap();

    let json = rt.eval_json("document.getElementById('main')").unwrap();
    assert!(json.is_object());
    let obj = json.as_object().unwrap();
    assert_eq!(obj.get("tag").unwrap(), "div");
    assert_eq!(obj.get("id").unwrap(), "main");
}

// ---------------------------------------------------------------------------
// 3. Script inline
// ---------------------------------------------------------------------------
#[test]
fn test_script_inline() {
    let html =
        r#"<html><head><script>document.title = 'JS OK'</script></head><body></body></html>"#;
    let doc = HtmlDocument::parse(html);
    let mut rt = JsRuntime::new().unwrap();
    rt.set_dom(&doc).unwrap();

    let base_url = url::Url::parse("https://example.com").unwrap();
    let config = faf_browser::utils::config::Config::default();
    let client = faf_browser::http::client::HttpClient::new(config).unwrap();

    rt.execute_page_scripts(&doc, &base_url, &client).unwrap();

    let title = rt.eval("document.title").unwrap();
    assert_eq!(title, "JS OK");
}

// ---------------------------------------------------------------------------
// 4. setTimeout
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_settimeout() {
    let mut rt = JsRuntime::new().unwrap();
    rt.init_timers().unwrap();
    rt.eval("setTimeout(() => { globalThis.timeoutResult = 42; }, 50)")
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    let result = rt.eval("globalThis.timeoutResult").unwrap();
    assert_eq!(result, "42");
}

// ---------------------------------------------------------------------------
// 5. Fetch
// ---------------------------------------------------------------------------
#[test]
fn test_fetch() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 1024];
        let _ = stream.read(&mut buf);
        let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: 13\r\n\r\n{\"hello\":123}";
        stream.write_all(response.as_bytes()).unwrap();
    });

    let config = faf_browser::utils::config::Config::default();
    let client = faf_browser::http::client::HttpClient::new(config).unwrap();
    let mut rt = JsRuntime::with_client(client).unwrap();
    rt.init_fetch().unwrap();

    let result = rt
        .eval(&format!(
            r#"var r = fetch('http://127.0.0.1:{}/test'); JSON.stringify({{status: r.status, body: r.text()}});"#,
            port
        ))
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["status"], 200);
    assert_eq!(parsed["body"], r#"{"hello":123}"#);
}

// ---------------------------------------------------------------------------
// 6. Timeout de segurança
// ---------------------------------------------------------------------------
#[test]
fn test_timeout_security() {
    let rt = JsRuntime::new().unwrap();
    let result = rt.eval_with_timeout("while(true){}", 1);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("timed out") || err.contains("timeout"),
        "expected timeout error, got: {}",
        err
    );
}

// ---------------------------------------------------------------------------
// 7. Error handling
// ---------------------------------------------------------------------------
#[test]
fn test_error_handling() {
    let rt = JsRuntime::new().unwrap();
    let result = rt.eval("undefined.x");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("TypeError") || err.contains("Error"),
        "expected TypeError or Error in: {}",
        err
    );
}

// ---------------------------------------------------------------------------
// 8. CLI --js
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_cli_js() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 1024];
        let _ = stream.read(&mut buf);
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\nContent-Length: 52\r\n\r\n<html><head><title>Test</title></head><body></body></html>";
        stream.write_all(response.as_bytes()).unwrap();
    });

    let cli = Cli::parse_from([
        "faf",
        &format!("http://127.0.0.1:{}/", port),
        "--js",
        "document.title = 'CLI JS OK'; document.title",
    ]);

    let result = run(cli).await;
    assert!(result.is_ok(), "CLI run failed: {:?}", result);
}

// ---------------------------------------------------------------------------
// 9. console.log
// ---------------------------------------------------------------------------
#[test]
fn test_console_log() {
    let rt = JsRuntime::new().unwrap();
    // Não deve panicar
    rt.eval("console.log('hello', 'world')").unwrap();
    rt.eval("console.warn('warning')").unwrap();
    rt.eval("console.error('error')").unwrap();
}
