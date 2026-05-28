use crate::css::selector::ElementMatch;
use crate::css::style::ComputedStyle;
use crate::dom::HtmlDocument;
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Serialize)]
pub struct PageOutput {
    pub url: String,
    pub title: Option<String>,
    pub links: Vec<(String, String)>,
    pub images: Vec<(String, String)>,
    pub metadata: std::collections::HashMap<String, String>,
    pub text: String,
}

#[derive(Serialize)]
pub struct QueryOutput {
    pub selector: String,
    pub results: Vec<QueryItem>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct QueryItem {
    pub tag: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attributes: std::collections::HashMap<String, String>,
    pub text: String,
    pub html: String,
}

#[derive(Serialize)]
pub struct StyledQueryItem {
    pub tag: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attributes: std::collections::HashMap<String, String>,
    pub text: String,
    pub html: String,
    pub computed_style: ComputedStyle,
}

#[derive(Serialize)]
pub struct StyledQueryOutput {
    pub selector: String,
    pub results: Vec<StyledQueryItem>,
    pub count: usize,
}

/// Converte resultado de query CSS pra output serializável
pub fn query_to_output(selector: &str, results: Vec<crate::dom::QueryResult>) -> QueryOutput {
    let items: Vec<QueryItem> = results
        .into_iter()
        .map(|r| {
            let attrs: std::collections::HashMap<_, _> = r.attributes.into_iter().collect();
            QueryItem {
                tag: r.tag,
                id: r.id,
                classes: r.classes,
                attributes: attrs,
                text: r.text,
                html: r.html,
            }
        })
        .collect();

    let count = items.len();
    QueryOutput {
        selector: selector.to_string(),
        results: items,
        count,
    }
}

/// Converte resultado de query CSS com estilos computados pra output serializável
pub fn styled_query_to_output(
    selector: &str,
    results: Vec<crate::dom::QueryResult>,
    styles: &[(ElementMatch, ComputedStyle)],
) -> StyledQueryOutput {
    let items: Vec<StyledQueryItem> = results
        .into_iter()
        .map(|r| {
            let computed_style = find_computed_style(&r, styles);
            let attrs: std::collections::HashMap<_, _> = r.attributes.into_iter().collect();
            StyledQueryItem {
                tag: r.tag,
                id: r.id,
                classes: r.classes,
                attributes: attrs,
                text: r.text,
                html: r.html,
                computed_style,
            }
        })
        .collect();

    let count = items.len();
    StyledQueryOutput {
        selector: selector.to_string(),
        results: items,
        count,
    }
}

pub fn find_computed_style(
    result: &crate::dom::QueryResult,
    styles: &[(ElementMatch, ComputedStyle)],
) -> ComputedStyle {
    styles
        .iter()
        .find(|(em, _)| {
            em.tag == result.tag
                && em.id == result.id
                && em.classes == result.classes
                && em.text == result.text
        })
        .map(|(_, s)| s.clone())
        .unwrap_or_default()
}

/// Extrai dados estruturados de um HtmlDocument
pub fn extract_page(url: &str, doc: &HtmlDocument) -> PageOutput {
    PageOutput {
        url: url.to_string(),
        title: doc.title(),
        links: doc.links(),
        images: doc.images(),
        metadata: doc.metadata(),
        text: doc.visible_text(),
    }
}

/// Filtra campos específicos de um StyledQueryItem.
/// Se `fields` for None, retorna o item completo serializado.
pub fn filter_fields(fields: &Option<Vec<String>>, item: &StyledQueryItem) -> Value {
    if let Some(f) = fields {
        let mut map = Map::new();
        for field in f {
            let v = match field.as_str() {
                "tag" => Value::String(item.tag.clone()),
                "id" => item.id.clone().map(Value::String).unwrap_or(Value::Null),
                "classes" => Value::Array(
                    item.classes
                        .iter()
                        .map(|c| Value::String(c.clone()))
                        .collect(),
                ),
                "text" => Value::String(item.text.clone()),
                "html" => Value::String(item.html.clone()),
                "href" => item
                    .attributes
                    .get("href")
                    .map(|v| Value::String(v.clone()))
                    .unwrap_or(Value::Null),
                "src" => item
                    .attributes
                    .get("src")
                    .map(|v| Value::String(v.clone()))
                    .unwrap_or(Value::Null),
                "alt" => item
                    .attributes
                    .get("alt")
                    .map(|v| Value::String(v.clone()))
                    .unwrap_or(Value::Null),
                "color" => Value::String(item.computed_style.color.clone()),
                "bg" => Value::String(item.computed_style.background_color.clone()),
                "font-size" => Value::String(item.computed_style.font_size.clone()),
                "font-family" => Value::String(item.computed_style.font_family.clone()),
                "display" => Value::String(item.computed_style.display.clone()),
                other => item
                    .attributes
                    .get(other)
                    .map(|v| Value::String(v.clone()))
                    .unwrap_or(Value::Null),
            };
            map.insert(field.clone(), v);
        }
        Value::Object(map)
    } else {
        serde_json::to_value(item).unwrap_or(Value::Null)
    }
}

/// Converte resultados de query CSS com estilos para vetor de JSON filtrado.
pub fn styled_results_to_filtered_json(
    results: Vec<crate::dom::QueryResult>,
    styles: &[(ElementMatch, ComputedStyle)],
    fields: &Option<Vec<String>>,
) -> Vec<Value> {
    results
        .into_iter()
        .map(|r| {
            let computed_style = find_computed_style(&r, styles);
            let attrs: std::collections::HashMap<_, _> = r.attributes.into_iter().collect();
            let item = StyledQueryItem {
                tag: r.tag,
                id: r.id,
                classes: r.classes,
                attributes: attrs,
                text: r.text,
                html: r.html,
                computed_style,
            };
            filter_fields(fields, &item)
        })
        .collect()
}

/// Converte resultados de query para StyledQueryItems.
pub fn to_styled_items(
    results: Vec<crate::dom::QueryResult>,
    styles: &[(ElementMatch, ComputedStyle)],
) -> Vec<StyledQueryItem> {
    results
        .into_iter()
        .map(|r| {
            let computed_style = find_computed_style(&r, styles);
            let attrs: std::collections::HashMap<_, _> = r.attributes.into_iter().collect();
            StyledQueryItem {
                tag: r.tag,
                id: r.id,
                classes: r.classes,
                attributes: attrs,
                text: r.text,
                html: r.html,
                computed_style,
            }
        })
        .collect()
}

/// Formata itens estilizados como JSON Lines (um JSON por linha).
pub fn to_jsonl(items: &[StyledQueryItem], fields: &Option<Vec<String>>) -> String {
    items
        .iter()
        .map(|item| filter_fields(fields, item).to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Formata itens estilizados como CSV (cabeçalho + linhas).
pub fn to_csv(items: &[StyledQueryItem], fields: &[String]) -> String {
    let mut lines = vec![fields.join(",")];
    for item in items {
        let obj = filter_fields(&Some(fields.to_vec()), item);
        if let Value::Object(map) = obj {
            let row: Vec<String> = fields
                .iter()
                .map(|f| format_csv_value(map.get(f).unwrap_or(&Value::Null)))
                .collect();
            lines.push(row.join(","));
        }
    }
    lines.join("\n")
}

fn format_csv_value(val: &Value) -> String {
    match val {
        Value::String(s) => escape_csv(s),
        Value::Null => String::new(),
        Value::Array(arr) => escape_csv(
            &arr
                .iter()
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join(" "),
        ),
        other => escape_csv(&other.to_string()),
    }
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
