use anyhow::{Context as _, Result};
use rquickjs::{Context, Runtime};

pub struct JsRuntime {
    #[allow(dead_code)]
    runtime: Runtime,
    context: Context,
}

impl JsRuntime {
    pub fn new() -> Result<Self> {
        let runtime = Runtime::new()?;
        let context = Context::full(&runtime).context("failed to create full rquickjs context")?;
        let mut this = Self { runtime, context };
        this.init_console()?;
        Ok(this)
    }

    pub fn init_console(&mut self) -> Result<()> {
        self.context.with(|ctx| {
            let console = rquickjs::Object::new(ctx.clone())
                .context("failed to create console object")?;

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

            console.set("log", log).context("failed to set console.log")?;
            console.set("warn", warn).context("failed to set console.warn")?;
            console.set("error", error).context("failed to set console.error")?;
            ctx.globals()
                .set("console", console)
                .context("failed to set global console")?;
            Ok(())
        })
    }

    pub fn eval(&self, code: &str) -> Result<String> {
        self.context.with(|ctx| {
            let value: rquickjs::Coerced<String> = ctx.eval(code).context("JS eval failed")?;
            Ok(value.0)
        })
    }

    pub fn eval_json(&self, code: &str) -> Result<serde_json::Value> {
        self.context.with(|ctx| {
            let value: rquickjs::Value = ctx.eval(code).context("JS eval failed")?;
            ctx.globals()
                .set("__faf_eval_tmp", value)
                .context("failed to set temp global")?;
            let json_str: String = ctx
                .eval("JSON.stringify(__faf_eval_tmp)")
                .context("JSON.stringify failed")?;
            let json: serde_json::Value =
                serde_json::from_str(&json_str).context("failed to parse JSON string")?;
            Ok(json)
        })
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
}
