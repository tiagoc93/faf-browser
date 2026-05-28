use scraper;

/// Navegador/parser HTML usando scraper (html5ever)
pub struct HtmlDocument {
    inner: scraper::Html,
}

/// Resultado de uma query CSS
pub struct QueryResult {
    pub tag: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attributes: Vec<(String, String)>,
    pub text: String,
    pub inner_html: String,
    pub html: String,
}

impl HtmlDocument {
    /// Parseia HTML string
    pub fn parse(html: &str) -> Self {
        Self {
            inner: scraper::Html::parse_document(html),
        }
    }

    /// Título da página
    pub fn title(&self) -> Option<String> {
        let selector = scraper::Selector::parse("title").ok()?;
        self.inner
            .select(&selector)
            .next()
            .and_then(|el| el.text().collect::<String>().into())
            .map(|s: String| s.trim().to_string())
    }

    /// Extrai todos os links (href) da página
    pub fn links(&self) -> Vec<(String, String)> {
        let selector = scraper::Selector::parse("a[href]").unwrap();
        self.inner
            .select(&selector)
            .filter_map(|el| {
                let href = el.value().attr("href")?;
                let text: String = el.text().collect();
                Some((text.trim().to_string(), href.to_string()))
            })
            .collect()
    }

    /// Extrai todas as imagens
    pub fn images(&self) -> Vec<(String, String)> {
        let selector = scraper::Selector::parse("img[src]").unwrap();
        self.inner
            .select(&selector)
            .filter_map(|el| {
                let src = el.value().attr("src")?;
                let alt = el.value().attr("alt").unwrap_or("");
                Some((alt.to_string(), src.to_string()))
            })
            .collect()
    }

    /// Extrai metadados (Open Graph, meta tags)
    pub fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut meta = std::collections::HashMap::new();

        // Meta tags com name ou property
        let selector = scraper::Selector::parse("meta[name], meta[property]").unwrap();
        for el in self.inner.select(&selector) {
            let name = el
                .value()
                .attr("name")
                .or_else(|| el.value().attr("property"))
                .unwrap_or("");
            let content = el.value().attr("content").unwrap_or("");
            meta.insert(name.to_string(), content.to_string());
        }

        meta
    }

    /// Executa query CSS e retorna resultados
    pub fn query(&self, selector_str: &str) -> anyhow::Result<Vec<QueryResult>> {
        let selector = scraper::Selector::parse(selector_str)
            .map_err(|e| anyhow::anyhow!("Seletor CSS inválido '{}': {:?}", selector_str, e))?;

        let results: Vec<QueryResult> = self
            .inner
            .select(&selector)
            .map(|el| {
                let text: String = el.text().collect();
                // Acessar HTML interno via fragment
                let inner_html: String = el.inner_html();
                let html: String = format!(
                    "<{}{}{}>{}</{}>",
                    el.value().name(),
                    el.value()
                        .attr("id")
                        .map(|id| format!(" id=\"{}\"", id.replace('"', "&quot;")))
                        .unwrap_or_default(),
                    el.value()
                        .attr("class")
                        .map(|c| format!(" class=\"{}\"", c.replace('"', "&quot;")))
                        .unwrap_or_default(),
                    inner_html,
                    el.value().name(),
                );

                QueryResult {
                    tag: el.value().name().to_string(),
                    id: el.value().attr("id").map(|s| s.to_string()),
                    classes: el
                        .value()
                        .attr("class")
                        .map(|c| c.split_whitespace().map(|s| s.to_string()).collect())
                        .unwrap_or_default(),
                    attributes: el
                        .value()
                        .attrs()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                    text: text.trim().to_string(),
                    inner_html,
                    html,
                }
            })
            .collect();

        Ok(results)
    }

    /// Coleta todo o texto visível da página
    pub fn visible_text(&self) -> String {
        let selector = scraper::Selector::parse("body").unwrap();
        self.inner
            .select(&selector)
            .map(|el| {
                let text: String = el.text().collect();
                text
            })
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Retorna o HTML fonte
    pub fn html(&self) -> String {
        self.inner.html()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_title() {
        let doc = HtmlDocument::parse("<html><head><title>Teste</title></head><body></body></html>");
        assert_eq!(doc.title(), Some("Teste".to_string()));
    }

    #[test]
    fn test_query_by_tag() {
        let doc = HtmlDocument::parse("<div><h1>Título</h1><p>Parágrafo</p></div>");
        let results = doc.query("h1").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "Título".to_string());

        let results = doc.query("p").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "Parágrafo".to_string());
    }

    #[test]
    fn test_query_by_class() {
        let html = r#"<div class="container"><p class="highlight">Texto</p><p>Normal</p></div>"#;
        let doc = HtmlDocument::parse(html);
        let results = doc.query(".highlight").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].classes.contains(&"highlight".to_string()));
    }

    #[test]
    fn test_query_by_id() {
        let html = r#"<div id="main"><p>Conteúdo</p></div>"#;
        let doc = HtmlDocument::parse(html);
        let results = doc.query("#main").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, Some("main".to_string()));
    }

    #[test]
    fn test_links() {
        let html = r#"<a href="https://example.com">Exemplo</a><a href="/about">Sobre</a>"#;
        let doc = HtmlDocument::parse(html);
        let links = doc.links();
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].1, "https://example.com");
    }

    #[test]
    fn test_images() {
        let html = r#"<img src="foto.jpg" alt="Foto"><img src="icon.png">"#;
        let doc = HtmlDocument::parse(html);
        let imgs = doc.images();
        assert_eq!(imgs.len(), 2);
        assert_eq!(imgs[0].1, "foto.jpg");
    }

    #[test]
    fn test_metadata() {
        let html = r#"<meta name="description" content="Teste"><meta property="og:title" content="OG Title">"#;
        let doc = HtmlDocument::parse(html);
        let meta = doc.metadata();
        assert!(meta.contains_key("description") || meta.contains_key("og:title"));
    }

    #[test]
    fn test_invalid_selector() {
        let doc = HtmlDocument::parse("<p>Oi</p>");
        let result = doc.query("]][[");
        assert!(result.is_err());
    }
}
