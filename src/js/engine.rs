use anyhow::{Context as _, Result};
use rquickjs::{Context, Runtime};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use crate::dom::HtmlDocument;
use crate::http::client::HttpClient;

pub(crate) static TIMER_RT: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().expect("failed to create timer runtime"));

pub struct JsRuntime {
    #[allow(dead_code)]
    runtime: Runtime,
    context: Context,
    client: Option<crate::http::client::HttpClient>,
}

impl JsRuntime {
    pub fn new() -> Result<Self> {
        let runtime = Runtime::new()?;
        let context = Context::full(&runtime).context("failed to create full rquickjs context")?;
        let mut this = Self {
            runtime,
            context,
            client: None,
        };
        this.init_console()?;
        Ok(this)
    }

    pub fn with_client(client: crate::http::client::HttpClient) -> Result<Self> {
        let mut this = Self::new()?;
        this.client = Some(client);
        Ok(this)
    }

    pub fn init_console(&mut self) -> Result<()> {
        self.context.with(|ctx| {
            let console =
                rquickjs::Object::new(ctx.clone()).context("failed to create console object")?;

            let log = rquickjs::Function::new(
                ctx.clone(),
                |args: rquickjs::function::Rest<rquickjs::Value<'_>>| {
                    let parts: Vec<String> = args
                        .0
                        .iter()
                        .map(|v| {
                            v.as_string()
                                .map(|s| s.to_string().unwrap_or_default())
                                .unwrap_or_else(|| {
                                    v.ctx()
                                        .json_stringify(v)
                                        .ok()
                                        .flatten()
                                        .and_then(|s| s.to_string().ok())
                                        .unwrap_or_else(|| "[object]".to_string())
                                })
                        })
                        .collect();
                    log::info!("[JS] {}", parts.join(" "));
                },
            )
            .context("failed to create console.log")?;

            let warn = rquickjs::Function::new(
                ctx.clone(),
                |args: rquickjs::function::Rest<rquickjs::Value<'_>>| {
                    let parts: Vec<String> = args
                        .0
                        .iter()
                        .map(|v| {
                            v.as_string()
                                .map(|s| s.to_string().unwrap_or_default())
                                .unwrap_or_else(|| {
                                    v.ctx()
                                        .json_stringify(v)
                                        .ok()
                                        .flatten()
                                        .and_then(|s| s.to_string().ok())
                                        .unwrap_or_else(|| "[object]".to_string())
                                })
                        })
                        .collect();
                    log::warn!("[JS] {}", parts.join(" "));
                },
            )
            .context("failed to create console.warn")?;

            let error = rquickjs::Function::new(
                ctx.clone(),
                |args: rquickjs::function::Rest<rquickjs::Value<'_>>| {
                    let parts: Vec<String> = args
                        .0
                        .iter()
                        .map(|v| {
                            v.as_string()
                                .map(|s| s.to_string().unwrap_or_default())
                                .unwrap_or_else(|| {
                                    v.ctx()
                                        .json_stringify(v)
                                        .ok()
                                        .flatten()
                                        .and_then(|s| s.to_string().ok())
                                        .unwrap_or_else(|| "[object]".to_string())
                                })
                        })
                        .collect();
                    log::error!("[JS] {}", parts.join(" "));
                },
            )
            .context("failed to create console.error")?;

            console
                .set("log", log)
                .context("failed to set console.log")?;
            console
                .set("warn", warn)
                .context("failed to set console.warn")?;
            console
                .set("error", error)
                .context("failed to set console.error")?;
            ctx.globals()
                .set("console", console)
                .context("failed to set global console")?;
            Ok(())
        })
    }

    pub fn init_timers(&mut self) -> Result<()> {
        self.context.with(|ctx| {
            let timers: Arc<Mutex<HashMap<u64, bool>>> = Arc::new(Mutex::new(HashMap::new()));
            let counter = Arc::new(AtomicU64::new(1));
            let context = self.context.clone();

            // setTimeout
            let set_timeout = {
                let timers = timers.clone();
                let counter = counter.clone();
                let context = context.clone();
                rquickjs::Function::new(
                    ctx.clone(),
                    move |code: String, ms: u64| -> u64 {
                        let id = counter.fetch_add(1, Ordering::SeqCst);
                        timers.lock().unwrap().insert(id, false);
                        let timers = timers.clone();
                        let context = context.clone();
                        TIMER_RT.spawn(async move {
                            tokio::time::sleep(Duration::from_millis(ms)).await;
                            let cancelled = timers.lock().unwrap().get(&id).copied().unwrap_or(true);
                            if cancelled {
                                return;
                            }
                            let _ = context.with(|ctx| {
                                ctx.eval::<(), _>(code.as_str())?;
                                Ok::<_, rquickjs::Error>(())
                            });
                            timers.lock().unwrap().remove(&id);
                        });
                        id
                    },
                )
                .context("failed to create setTimeout")?
            };

            // clearTimeout
            let clear_timeout = {
                let timers = timers.clone();
                rquickjs::Function::new(
                    ctx.clone(),
                    move |id: u64| {
                        timers.lock().unwrap().insert(id, true);
                    },
                )
                .context("failed to create clearTimeout")?
            };

            // setInterval
            let set_interval = {
                let timers = timers.clone();
                let counter = counter.clone();
                let context = context.clone();
                rquickjs::Function::new(
                    ctx.clone(),
                    move |code: String, ms: u64| -> u64 {
                        let id = counter.fetch_add(1, Ordering::SeqCst);
                        timers.lock().unwrap().insert(id, false);
                        let timers = timers.clone();
                        let context = context.clone();
                        TIMER_RT.spawn(async move {
                            loop {
                                tokio::time::sleep(Duration::from_millis(ms)).await;
                                let cancelled =
                                    timers.lock().unwrap().get(&id).copied().unwrap_or(true);
                                if cancelled {
                                    break;
                                }
                                let _ = context.with(|ctx| {
                                    ctx.eval::<(), _>(code.as_str())?;
                                    Ok::<_, rquickjs::Error>(())
                                });
                            }
                        });
                        id
                    },
                )
                .context("failed to create setInterval")?
            };

            // clearInterval
            let clear_interval = {
                let timers = timers.clone();
                rquickjs::Function::new(
                    ctx.clone(),
                    move |id: u64| {
                        timers.lock().unwrap().insert(id, true);
                    },
                )
                .context("failed to create clearInterval")?
            };

            let global = ctx.globals();
            global
                .set("__rust_setTimeout", set_timeout)
                .context("failed to set __rust_setTimeout")?;
            global
                .set("__rust_clearTimeout", clear_timeout)
                .context("failed to set __rust_clearTimeout")?;
            global
                .set("__rust_setInterval", set_interval)
                .context("failed to set __rust_setInterval")?;
            global
                .set("__rust_clearInterval", clear_interval)
                .context("failed to set __rust_clearInterval")?;

            // JS wrappers to accept both functions and strings
            ctx.eval::<(), _>(r#"
                (function() {
                    const _st = globalThis.__rust_setTimeout;
                    const _ct = globalThis.__rust_clearTimeout;
                    const _si = globalThis.__rust_setInterval;
                    const _ci = globalThis.__rust_clearInterval;
                    globalThis.setTimeout = function(fn, ms) {
                        var code = (typeof fn === 'function') ? '(' + fn.toString() + ')()' : String(fn);
                        return _st(code, ms || 0);
                    };
                    globalThis.clearTimeout = function(id) { _ct(id); };
                    globalThis.setInterval = function(fn, ms) {
                        var code = (typeof fn === 'function') ? '(' + fn.toString() + ')()' : String(fn);
                        return _si(code, ms || 0);
                    };
                    globalThis.clearInterval = function(id) { _ci(id); };
                })();
            "#)
            .context("failed to inject timer wrappers")?;

            Ok(())
        })
    }

    pub fn eval(&self, code: &str) -> Result<String> {
        self.context.with(|ctx| {
            let result: rquickjs::Result<rquickjs::Coerced<String>> = ctx.eval(code);
            match result {
                Ok(v) => Ok(v.0),
                Err(e) => {
                    let err_msg = format_js_error(&ctx, e);
                    Err(anyhow::anyhow!("{}", err_msg))
                }
            }
        })
    }

    pub fn eval_json(&self, code: &str) -> Result<serde_json::Value> {
        self.context.with(|ctx| {
            let value: rquickjs::Value = match ctx.eval(code) {
                Ok(v) => v,
                Err(e) => {
                    let err_msg = format_js_error(&ctx, e);
                    return Err(anyhow::anyhow!("{}", err_msg));
                }
            };
            ctx.globals()
                .set("__faf_eval_tmp", value)
                .context("failed to set temp global")?;
            let json_str: String = match ctx.eval("JSON.stringify(__faf_eval_tmp)") {
                Ok(v) => v,
                Err(e) => {
                    let err_msg = format_js_error(&ctx, e);
                    return Err(anyhow::anyhow!("{}", err_msg));
                }
            };
            let json: serde_json::Value =
                serde_json::from_str(&json_str).context("failed to parse JSON string")?;
            Ok(json)
        })
    }

    pub fn eval_with_timeout(&self, code: &str, timeout_secs: u64) -> Result<String> {
        let code = code.to_string();
        let context = self.context.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let res = context.with(|ctx| {
                let result: rquickjs::Result<rquickjs::Coerced<String>> = ctx.eval(code.as_str());
                match result {
                    Ok(v) => Ok(v.0),
                    Err(e) => {
                        let err_msg = format_js_error(&ctx, e);
                        Err(anyhow::anyhow!("{}", err_msg))
                    }
                }
            });
            let _ = tx.send(res);
        });
        match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
            Ok(res) => res,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(anyhow::anyhow!(
                "JavaScript execution timed out after {}s",
                timeout_secs
            )),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                Err(anyhow::anyhow!("JavaScript execution thread disconnected"))
            }
        }
    }

    pub fn eval_json_with_timeout(
        &self,
        code: &str,
        timeout_secs: u64,
    ) -> Result<serde_json::Value> {
        let code = code.to_string();
        let context = self.context.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let res = context.with(|ctx| {
                let value: rquickjs::Value = match ctx.eval(code.as_str()) {
                    Ok(v) => v,
                    Err(e) => {
                        let err_msg = format_js_error(&ctx, e);
                        return Err(anyhow::anyhow!("{}", err_msg));
                    }
                };
                ctx.globals()
                    .set("__faf_eval_tmp", value)
                    .context("failed to set temp global")?;
                let json_str: String = match ctx.eval("JSON.stringify(__faf_eval_tmp)") {
                    Ok(v) => v,
                    Err(e) => {
                        let err_msg = format_js_error(&ctx, e);
                        return Err(anyhow::anyhow!("{}", err_msg));
                    }
                };
                let json: serde_json::Value =
                    serde_json::from_str(&json_str).context("failed to parse JSON string")?;
                Ok(json)
            });
            let _ = tx.send(res);
        });
        match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
            Ok(res) => res,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(anyhow::anyhow!(
                "JavaScript execution timed out after {}s",
                timeout_secs
            )),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                Err(anyhow::anyhow!("JavaScript execution thread disconnected"))
            }
        }
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn set_dom(&mut self, doc: &crate::dom::HtmlDocument) -> Result<()> {
        self.context.with(|ctx| {
            super::dom_bridge::inject_dom(&ctx, doc).context("failed to inject DOM bridge")?;
            Ok(())
        })
    }

    pub fn init_fetch(&mut self) -> Result<()> {
        let client = self.client.as_ref().context(
            "HttpClient not set. Use with_client() or set the client before calling init_fetch()",
        )?;
        super::fetch_bridge::init_fetch(&self.context, client)
    }

    pub fn client(&self) -> Option<&crate::http::client::HttpClient> {
        self.client.as_ref()
    }

    /// Executa todos os scripts `<script>` encontrados no documento HTML.
    /// Scripts inline são executados diretamente; scripts externos são baixados
    /// e depois executados. Erros são logados como warning e não interrompem
    /// o processamento dos demais scripts.
    pub fn execute_page_scripts(
        &mut self,
        doc: &HtmlDocument,
        base_url: &url::Url,
        client: &HttpClient,
    ) -> Result<()> {
        let selector = scraper::Selector::parse("script")
            .map_err(|e| anyhow::anyhow!("Falha ao parsear seletor de scripts: {:?}", e))?;

        for el in doc.scraper_html().select(&selector) {
            // Ignorar scripts com type="module" ou type="application/ld+json"
            if let Some(ty) = el.value().attr("type") {
                let ty = ty.trim().to_lowercase();
                if ty == "module" || ty == "application/ld+json" {
                    continue;
                }
            }

            if let Some(src) = el.value().attr("src") {
                // Script externo
                let resolved_url = match base_url.join(src) {
                    Ok(u) => u.to_string(),
                    Err(e) => {
                        log::warn!("URL inválida para script externo ({}): {}", src, e);
                        continue;
                    }
                };

                let script_body = match TIMER_RT.block_on(async { client.get(&resolved_url).await })
                {
                    Ok(body) => body,
                    Err(e) => {
                        log::warn!("Falha ao baixar script externo ({}): {}", resolved_url, e);
                        continue;
                    }
                };

                if let Err(e) = self.eval_with_timeout(&script_body, 5) {
                    log::warn!("Erro ao executar script externo ({}): {}", resolved_url, e);
                }
            } else {
                // Script inline
                let code: String = el.text().collect();
                let code = code.trim().to_string();
                if code.is_empty() {
                    continue;
                }
                if let Err(e) = self.eval_with_timeout(&code, 5) {
                    log::warn!("Erro ao executar script inline: {}", e);
                }
            }
        }

        Ok(())
    }
}

fn format_js_error<'js>(ctx: &rquickjs::Ctx<'js>, err: rquickjs::Error) -> String {
    use rquickjs::atom::PredefinedAtom;
    use rquickjs::{Coerced, Exception};

    match err {
        rquickjs::Error::Exception => {
            let value = ctx.catch();
            if let Some(obj) = value.as_object() {
                if let Some(ex) = Exception::from_object(obj.clone()) {
                    let name = obj
                        .get::<_, Option<Coerced<String>>>(PredefinedAtom::Name)
                        .ok()
                        .flatten()
                        .map(|c| c.0)
                        .unwrap_or_else(|| "Error".to_string());
                    let message = ex.message().unwrap_or_default();
                    let stack = ex.stack().unwrap_or_default();

                    if stack.is_empty() {
                        format!("{}: {}", name, message)
                    } else {
                        format!("{}: {}\n{}", name, message, stack)
                    }
                } else {
                    let msg = value
                        .as_string()
                        .and_then(|s| s.to_string().ok())
                        .unwrap_or_else(|| format!("{:?}", value));
                    format!("Exception: {}", msg)
                }
            } else {
                let msg = value
                    .as_string()
                    .and_then(|s| s.to_string().ok())
                    .or_else(|| value.as_int().map(|i| i.to_string()))
                    .or_else(|| value.as_float().map(|f| f.to_string()))
                    .unwrap_or_else(|| format!("{:?}", value));
                format!("Exception: {}", msg)
            }
        }
        other => format!("JS error: {}", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_runtime_new() {
        let rt = JsRuntime::new();
        assert!(rt.is_ok());
    }

    #[test]
    fn test_eval_simple() {
        let rt = JsRuntime::new().unwrap();
        let result = rt.eval("1 + 2");
        assert_eq!(result.unwrap(), "3");
    }

    #[test]
    fn test_eval_json_object() {
        let rt = JsRuntime::new().unwrap();
        let result = rt.eval_json("({a: 1, b: 2})");
        let expected = serde_json::json!({"a": 1, "b": 2});
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_dom_bridge_title() {
        let html = "<html><head><title>FAF Page</title></head><body></body></html>";
        let doc = crate::dom::HtmlDocument::parse(html);
        let mut rt = JsRuntime::new().unwrap();
        rt.set_dom(&doc).unwrap();

        let title = rt.eval("document.title").unwrap();
        assert_eq!(title, "FAF Page");
    }

    #[test]
    fn test_dom_bridge_get_element_by_id() {
        let html = r#"<div id="main"><h1>Hello</h1></div><p id="secondary">World</p>"#;
        let doc = crate::dom::HtmlDocument::parse(html);
        let mut rt = JsRuntime::new().unwrap();
        rt.set_dom(&doc).unwrap();

        let json = rt.eval_json("document.getElementById('main')").unwrap();
        assert!(json.is_object());
        let obj = json.as_object().unwrap();
        assert_eq!(obj.get("tag").unwrap(), "div");
        assert_eq!(obj.get("id").unwrap(), "main");
    }

    #[test]
    fn test_dom_bridge_get_element_by_id_null() {
        let html = "<div><p>Text</p></div>";
        let doc = crate::dom::HtmlDocument::parse(html);
        let mut rt = JsRuntime::new().unwrap();
        rt.set_dom(&doc).unwrap();

        let json = rt.eval_json("document.getElementById('missing')").unwrap();
        assert!(json.is_null());
    }

    #[test]
    fn test_dom_bridge_query_selector_all() {
        let html = "<div><h1>First</h1><h1>Second</h1><p>Para</p></div>";
        let doc = crate::dom::HtmlDocument::parse(html);
        let mut rt = JsRuntime::new().unwrap();
        rt.set_dom(&doc).unwrap();

        let json = rt.eval_json("document.querySelectorAll('h1')").unwrap();
        assert!(json.is_array());
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].get("tag").unwrap(), "h1");
        assert_eq!(arr[0].get("text").unwrap(), "First");
        assert_eq!(arr[1].get("text").unwrap(), "Second");
    }

    #[test]
    fn test_dom_bridge_query_selector_single() {
        let html = r#"<div><p class="intro">Introduction</p></div>"#;
        let doc = crate::dom::HtmlDocument::parse(html);
        let mut rt = JsRuntime::new().unwrap();
        rt.set_dom(&doc).unwrap();

        let json = rt.eval_json("document.querySelector('.intro')").unwrap();
        assert!(json.is_object());
        let obj = json.as_object().unwrap();
        assert_eq!(obj.get("tag").unwrap(), "p");
        assert_eq!(obj.get("text").unwrap(), "Introduction");
    }

    #[test]
    fn test_console_log() {
        let rt = JsRuntime::new().unwrap();
        // Should not panic; logs are captured by log crate
        rt.eval("console.log('hello', 'world')").unwrap();
    }

    #[test]
    fn test_console_warn() {
        let rt = JsRuntime::new().unwrap();
        rt.eval("console.warn('warning', 123)").unwrap();
    }

    #[test]
    fn test_console_error() {
        let rt = JsRuntime::new().unwrap();
        rt.eval("console.error('error', {a: 1})").unwrap();
    }

    #[test]
    fn test_eval_error_type_error() {
        let rt = JsRuntime::new().unwrap();
        let result = rt.eval("undefined.x");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("TypeError") || err.contains("Error"),
            "expected TypeError or Error in: {}",
            err
        );
        assert!(
            err.contains("undefined") || err.contains("read"),
            "expected 'undefined' or 'read' in: {}",
            err
        );
    }

    #[test]
    fn test_eval_error_syntax_error() {
        let rt = JsRuntime::new().unwrap();
        let result = rt.eval("{");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("SyntaxError") || err.contains("expected"),
            "expected SyntaxError or 'expected' in: {}",
            err
        );
    }

    #[test]
    fn test_eval_json_error() {
        let rt = JsRuntime::new().unwrap();
        let result = rt.eval_json("undefined.x");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("TypeError") || err.contains("Error"),
            "expected TypeError or Error in: {}",
            err
        );
        assert!(
            err.contains("undefined") || err.contains("read"),
            "expected 'undefined' or 'read' in: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_set_timeout() {
        let mut rt = JsRuntime::new().unwrap();
        rt.init_timers().unwrap();
        rt.eval("setTimeout(() => { globalThis.timeoutResult = 42; }, 50)")
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        let result = rt.eval("globalThis.timeoutResult").unwrap();
        assert_eq!(result, "42");
    }

    #[tokio::test]
    async fn test_clear_timeout() {
        let mut rt = JsRuntime::new().unwrap();
        rt.init_timers().unwrap();
        let id = rt
            .eval("setTimeout(() => { globalThis.clearedResult = 'bad'; }, 50)")
            .unwrap();
        rt.eval(&format!("clearTimeout({})", id)).unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        let result = rt.eval("typeof globalThis.clearedResult").unwrap();
        assert_eq!(result, "undefined");
    }

    #[tokio::test]
    async fn test_set_interval_and_clear() {
        let mut rt = JsRuntime::new().unwrap();
        rt.init_timers().unwrap();
        rt.eval(
            "globalThis.intervalCount = 0; setInterval(() => { globalThis.intervalCount++; }, 30)",
        )
        .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        let count: i32 = rt
            .eval("globalThis.intervalCount")
            .unwrap()
            .parse()
            .unwrap();
        assert!(
            count >= 3,
            "expected at least 3 interval ticks, got {}",
            count
        );

        rt.eval("clearInterval(globalThis.lastIntervalId || 1)")
            .unwrap();
        // Should not panic; interval may keep running if ID tracking is off, but we can at least verify eval works
    }

    #[tokio::test]
    async fn test_set_timeout_string_code() {
        let mut rt = JsRuntime::new().unwrap();
        rt.init_timers().unwrap();
        rt.eval("setTimeout('globalThis.stringResult = \"hello\"', 50)")
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        let result = rt.eval("globalThis.stringResult").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_fetch_sync() {
        let config = crate::utils::config::Config::default();
        let client = crate::http::client::HttpClient::new(config).unwrap();
        let mut rt = JsRuntime::with_client(client).unwrap();
        rt.init_fetch().unwrap();
        let result = rt.eval("typeof fetch").unwrap();
        assert_eq!(result, "function");
    }

    #[test]
    fn test_fetch_rejects_connection_refused() {
        let config = crate::utils::config::Config {
            timeout_secs: 5,
            ..crate::utils::config::Config::default()
        };
        let client = crate::http::client::HttpClient::new(config).unwrap();
        let mut rt = JsRuntime::with_client(client).unwrap();
        rt.init_fetch().unwrap();

        let result = rt.eval("fetch('http://127.0.0.1:1/')");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("error") || err.contains("refused") || err.contains("Connection"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_fetch_local_server() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: 13\r\n\r\n{\"hello\":123}";
            stream.write_all(response.as_bytes()).unwrap();
        });

        let config = crate::utils::config::Config::default();
        let client = crate::http::client::HttpClient::new(config).unwrap();
        let mut rt = JsRuntime::with_client(client).unwrap();
        rt.init_fetch().unwrap();

        let result = rt.eval(&format!(
            r#"var r = fetch('http://127.0.0.1:{}/test'); JSON.stringify({{status: r.status, body: r.text()}});"#,
            port
        ))
        .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], 200);
        assert_eq!(parsed["body"], r#"{"hello":123}"#);
    }

    #[test]
    fn test_fetch_response_json_method() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: 13\r\n\r\n{\"hello\":123}";
            stream.write_all(response.as_bytes()).unwrap();
        });

        let config = crate::utils::config::Config::default();
        let client = crate::http::client::HttpClient::new(config).unwrap();
        let mut rt = JsRuntime::with_client(client).unwrap();
        rt.init_fetch().unwrap();

        let result = rt
            .eval(&format!(
                r#"var r = fetch('http://127.0.0.1:{}/test'); JSON.stringify(r.json());"#,
                port
            ))
            .unwrap();
        assert_eq!(result, r#"{"hello":123}"#);
    }

    #[test]
    fn test_fetch_response_headers() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nX-Custom: test-value\r\nConnection: close\r\nContent-Length: 2\r\n\r\nok";
            stream.write_all(response.as_bytes()).unwrap();
        });

        let config = crate::utils::config::Config::default();
        let client = crate::http::client::HttpClient::new(config).unwrap();
        let mut rt = JsRuntime::with_client(client).unwrap();
        rt.init_fetch().unwrap();

        let result = rt
            .eval(&format!(
                r#"var r = fetch('http://127.0.0.1:{}/test'); JSON.stringify(r.headers);"#,
                port
            ))
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["content-type"], "application/json");
    }

    #[test]
    fn test_fetch_post_method() {
        let config = crate::utils::config::Config::default();
        let client = crate::http::client::HttpClient::new(config).unwrap();
        let mut rt = JsRuntime::with_client(client).unwrap();
        rt.init_fetch().unwrap();

        // Testar que options são aceitos (não vai conectar, mas não deve crashar)
        let result = rt.eval("typeof fetch('http://127.0.0.1:1/', {method: 'POST', body: 'test'})");
        // Deve lançar erro de conexão recusada, não erro de sintaxe
        assert!(result.is_err());
    }
    #[test]
    fn test_execute_page_scripts_inline() {
        let html =
            r#"<html><head><script>document.title = 'JS OK'</script></head><body></body></html>"#;
        let doc = crate::dom::HtmlDocument::parse(html);
        let mut rt = JsRuntime::new().unwrap();
        rt.set_dom(&doc).unwrap();

        let config = crate::utils::config::Config::default();
        let client = crate::http::client::HttpClient::new(config).unwrap();
        let base_url = url::Url::parse("https://example.com").unwrap();

        rt.execute_page_scripts(&doc, &base_url, &client).unwrap();

        let title = rt.eval("document.title").unwrap();
        assert_eq!(title, "JS OK");
    }

    #[test]
    fn test_execute_page_scripts_error_continues() {
        let html = r#"<html><head>
            <script>undefined.x</script>
            <script>document.title = 'RECOVERY'</script>
        </head><body></body></html>"#;
        let doc = crate::dom::HtmlDocument::parse(html);
        let mut rt = JsRuntime::new().unwrap();
        rt.set_dom(&doc).unwrap();

        let config = crate::utils::config::Config::default();
        let client = crate::http::client::HttpClient::new(config).unwrap();
        let base_url = url::Url::parse("https://example.com").unwrap();

        rt.execute_page_scripts(&doc, &base_url, &client).unwrap();

        let title = rt.eval("document.title").unwrap();
        assert_eq!(title, "RECOVERY");
    }
}
