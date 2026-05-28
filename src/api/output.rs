use crate::dom::HtmlDocument;
use serde::Serialize;

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
