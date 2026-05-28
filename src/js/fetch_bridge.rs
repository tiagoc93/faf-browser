use anyhow::{Context as _, Result};
use rquickjs::{Context, Function};
use serde_json::json;
use std::collections::HashMap;

/// Inicializa a API `fetch` no contexto QuickJS usando o HttpClient do Rust.
pub fn init_fetch(ctx: &Context, client: &crate::http::client::HttpClient) -> Result<()> {
    let client = client.clone();

    ctx.with(|ctx| {
        let inner_client = client.inner_client();
        let _fetch = Function::new(
            ctx.clone(),
            move |url: String, options_json: Option<String>| -> String {
                let options = options_json
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());

                let method = options
                    .as_ref()
                    .and_then(|v| {
                        v.get("method")
                            .and_then(|m| m.as_str())
                            .map(|s| s.to_uppercase())
                    })
                    .unwrap_or_else(|| "GET".to_string());

                let headers = options.as_ref().and_then(|v| v.get("headers").cloned());
                let body = options
                    .as_ref()
                    .and_then(|v| v.get("body").and_then(|b| b.as_str()).map(|s| s.to_string()));

                let req_method = match method.as_str() {
                    "GET" => reqwest::Method::GET,
                    "POST" => reqwest::Method::POST,
                    "PUT" => reqwest::Method::PUT,
                    "DELETE" => reqwest::Method::DELETE,
                    "PATCH" => reqwest::Method::PATCH,
                    "HEAD" => reqwest::Method::HEAD,
                    "OPTIONS" => reqwest::Method::OPTIONS,
                    _ => reqwest::Method::GET,
                };

                let response =
                    crate::js::engine::TIMER_RT.block_on(async {
                        let mut req = inner_client.request(req_method, &url);

                        if let Some(headers_obj) = headers
                            .as_ref()
                            .and_then(|v| v.as_object())
                        {
                                for (key, value) in headers_obj {
                                    if let Some(val_str) = value.as_str() {
                                        req = req.header(key, val_str);
                                    }
                                }
                            }
                        }

                        if let Some(body_str) = body {
                            req = req.body(body_str);
                        }

                        req.send().await
                    });

                match response {
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        let mut headers_map = HashMap::new();
                        for (key, value) in resp.headers() {
                            if let Ok(val_str) = value.to_str() {
                                headers_map.insert(key.to_string(), val_str.to_string());
                            }
                        }

                        let body = crate::js::engine::TIMER_RT.block_on(async { resp.text().await });

                        match body {
                            Ok(body_text) => json!({
                                "status": status,
                                "headers": headers_map,
                                "body": body_text,
                            })
                            .to_string(),
                            Err(e) => json!({
                                "status": status,
                                "headers": headers_map,
                                "error": format!("Failed to read body: {}", e),
                            })
                            .to_string(),
                        }
                    }
                    Err(e) => json!({
                        "status": 0,
                        "error": e.to_string(),
                    })
                    .to_string(),
                }
            },
        )
        .context("failed to create _faf_fetch function")?;

        ctx.globals()
            .set("_faf_fetch", _fetch)
            .context("failed to set global _faf_fetch")?;

        // Injeta o wrapper JS que expõe fetch como função síncrona
        // (sem Promise — mais prático para navegador headless)
        ctx.eval::<(), _>(r#"
            (function() {
                globalThis.fetch = function(url, options) {
                    var optionsStr = options ? JSON.stringify(options) : null;
                    var resultStr = globalThis._faf_fetch(String(url), optionsStr);
                    var result = JSON.parse(resultStr);
                    if (result.error) {
                        throw new Error(result.error);
                    }
                    return {
                        status: result.status,
                        headers: result.headers || {},
                        ok: result.status >= 200 && result.status < 300,
                        text: function() { return result.body; },
                        json: function() {
                            return JSON.parse(result.body);
                        }
                    };
                };
            })();
        "#)
        .context("failed to inject fetch wrapper")?;

        Ok(())
    })
}
