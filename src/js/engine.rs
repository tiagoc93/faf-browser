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
            let json: serde_json::Value = serde_json::from_str(&json_str)
                .context("failed to parse JSON string")?;
            Ok(json)
        })
    }

    pub fn context(&self) -> &Context {
        &self.context
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
}
