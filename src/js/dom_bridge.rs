use anyhow::Result;
use rquickjs::{Ctx, Function, Object};
use serde_json::json;

use crate::dom::HtmlDocument;

/// Converte um QueryResult do DOM para serde_json::Value.
fn query_result_to_json(result: &crate::dom::QueryResult) -> serde_json::Value {
    let mut attrs = serde_json::Map::new();
    for (k, v) in &result.attributes {
        attrs.insert(k.clone(), serde_json::Value::String(v.clone()));
    }

    json!({
        "tag": result.tag,
        "id": result.id,
        "classes": result.classes,
        "text": result.text,
        "attributes": attrs,
        "innerText": result.text,
        "textContent": result.text,
        "innerHTML": result.inner_html,
    })
}

/// Injeta o objeto global `document` no contexto JS, expondo funções DOM.
pub fn inject_dom(ctx: &Ctx<'_>, doc: &HtmlDocument) -> Result<()> {
    let document = Object::new(ctx.clone())?;

    // document.title
    let title = doc.title().unwrap_or_default();
    document.set("title", title)?;

    // Capturamos o HTML como string para reparsar dentro das closures.
    // HtmlDocument não é Send/Sync, então não pode ser compartilhado diretamente
    // com as closures do rquickjs. Reparsear é a estratégia mais segura.
    let html_string = doc.html();

    // _getElementById — função "raw" que retorna JSON string
    let html_for_id = html_string.clone();
    let _gebi = Function::new(ctx.clone(), move |id: String| -> Option<String> {
        let doc = HtmlDocument::parse(&html_for_id);
        let selector = format!("#{}", id);
        let results = doc.query(&selector).ok()?;
        if results.is_empty() {
            return None;
        }
        serde_json::to_string(&query_result_to_json(&results[0])).ok()
    })?;
    document.set("_getElementById", _gebi)?;

    // _querySelector — função "raw" que retorna JSON string
    let html_for_qs = html_string.clone();
    let _qs = Function::new(ctx.clone(), move |selector: String| -> Option<String> {
        let doc = HtmlDocument::parse(&html_for_qs);
        let results = doc.query(&selector).ok()?;
        if results.is_empty() {
            return None;
        }
        serde_json::to_string(&query_result_to_json(&results[0])).ok()
    })?;
    document.set("_querySelector", _qs)?;

    // _querySelectorAll — função "raw" que retorna JSON string (array)
    let html_for_qsa = html_string;
    let _qsa = Function::new(ctx.clone(), move |selector: String| -> String {
        let doc = HtmlDocument::parse(&html_for_qsa);
        let results = match doc.query(&selector) {
            Ok(r) => r,
            Err(_) => return "[]".to_string(),
        };
        let arr: Vec<serde_json::Value> = results.iter().map(query_result_to_json).collect();
        serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string())
    })?;
    document.set("_querySelectorAll", _qsa)?;

    // Publica o objeto document no global scope
    ctx.globals().set("document", document)?;

    // Injeta wrappers JS que convertem as strings JSON em objetos reais
    let _: () = ctx.eval(
        r#"
        (function() {
            var _gebi = document._getElementById;
            var _qs   = document._querySelector;
            var _qsa  = document._querySelectorAll;

            document.getElementById = function(id) {
                var r = _gebi(id);
                return r ? JSON.parse(r) : null;
            };

            document.querySelector = function(sel) {
                var r = _qs(sel);
                return r ? JSON.parse(r) : null;
            };

            document.querySelectorAll = function(sel) {
                var r = _qsa(sel);
                return JSON.parse(r);
            };
        })();
        "#,
    )?;

    Ok(())
}
