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
        Ok(Self { runtime, context })
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
}
