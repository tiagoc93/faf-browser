use crate::css::selector::ElementMatch;
use crate::css::style::ComputedStyle;
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

fn find_computed_style(
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
