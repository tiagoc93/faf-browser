use scraper::Html;

pub fn extract_main_content(html: &str) -> String {
    let document = Html::parse_document(html);
    let body_selector = match scraper::Selector::parse("body") {
        Ok(s) => s,
        Err(_) => return html.to_string(),
    };

    let body = match document.select(&body_selector).next() {
        Some(b) => b,
        None => return html.to_string(),
    };

    let mut best_score: f64 = 0.0;
    let mut best_html: String = html.to_string();

    find_best_content(body, &mut best_score, &mut best_html);

    best_html
}

fn find_best_content(
    element: scraper::ElementRef,
    best_score: &mut f64,
    best_html: &mut String,
) {
    for child in element.children() {
        if let Some(el) = scraper::ElementRef::wrap(child) {
            let tag = el.value().name().to_lowercase();
            if tag == "script" || tag == "style" || tag == "noscript" || tag == "svg" || tag == "canvas" {
                continue;
            }

            let score = score_content(el);
            if score > *best_score {
                *best_score = score;
                *best_html = el.inner_html();
            }

            find_best_content(el, best_score, best_html);
        }
    }
}

fn score_content(element: scraper::ElementRef) -> f64 {
    let stats = count_stats(element);
    let tag = element.value().name().to_lowercase();

    if tag == "nav" || tag == "footer" || tag == "header" || tag == "aside" {
        return 0.0;
    }

    if is_noise_element(element) {
        return 0.0;
    }

    let text_len = stats.text_len as f64;
    let tag_count = stats.tag_count as f64 + 1.0;
    let link_count = stats.link_count as f64;
    let para_count = stats.paragraph_count as f64;
    let heading_count = stats.heading_count as f64;

    let density = text_len / tag_count;
    let link_ratio = link_count / tag_count;

    let mut score = text_len * (1.0 + (density / 10.0).min(1.5));

    if link_ratio > 0.3 {
        score *= 0.3;
    }

    if heading_count > 0.0 {
        score *= 1.5;
    }
    if para_count > 3.0 {
        score *= 1.3;
    }

    if tag == "article" || tag == "main" {
        score *= 2.0;
    }

    if tag == "body" || tag == "html" {
        score *= 0.65;
    }

    if let Some(id) = element.value().id() {
        let id_lower = id.to_lowercase();
        if id_lower.contains("main") || id_lower.contains("content") || id_lower.contains("article") {
            score *= 1.8;
        }
    }

    if let Some(class_list) = element.value().attr("class") {
        let classes: Vec<&str> = class_list.split_whitespace().collect();
        let content_classes: Vec<&str> = classes
            .iter()
            .filter(|c| {
                let cl = c.to_lowercase();
                cl.contains("main") || cl.contains("content") || cl.contains("article") || cl.contains("post")
                    || cl.contains("entry") || cl.contains("body")
            })
            .copied()
            .collect();
        if !content_classes.is_empty() {
            score *= 1.4;
        }
    }

    score
}

struct ContentStats {
    text_len: usize,
    tag_count: usize,
    link_count: usize,
    paragraph_count: usize,
    heading_count: usize,
}

fn count_stats(element: scraper::ElementRef) -> ContentStats {
    let mut stats = ContentStats {
        text_len: 0,
        tag_count: 0,
        link_count: 0,
        paragraph_count: 0,
        heading_count: 0,
    };
    count_stats_recursive(element, &mut stats);
    stats
}

fn count_stats_recursive(element: scraper::ElementRef, stats: &mut ContentStats) {
    for child in element.children() {
        match child.value() {
            scraper::Node::Element(_el) => {
                let Some(el) = scraper::ElementRef::wrap(child) else { continue };
                let tag = el.value().name().to_lowercase();
                stats.tag_count += 1;

                match tag.as_str() {
                    "a" => stats.link_count += 1,
                    "p" => stats.paragraph_count += 1,
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => stats.heading_count += 1,
                    "script" | "style" | "noscript" | "svg" | "canvas" => continue,
                    _ => {}
                }

                if !is_noise_element(el) {
                    count_stats_recursive(el, stats);
                }
            }
            scraper::Node::Text(text) => {
                stats.text_len += text.text.trim().len();
            }
            _ => {}
        }
    }
}

fn is_noise_element(element: scraper::ElementRef) -> bool {
    let noise_patterns = [
        "nav", "menu", "sidebar", "footer", "header", "ad", "advertisement",
        "banner", "widget", "comment", "related", "social", "share", "cookie",
        "popup", "modal", "overlay", "newsletter", "subscribe",
        "alert", "warning", "disclaimer", "notice",
    ];

    if let Some(id) = element.value().id() {
        let id_lower = id.to_lowercase();
        for p in &noise_patterns {
            if id_lower.contains(p) {
                return true;
            }
        }
    }

    if let Some(class_list) = element.value().attr("class") {
        let class_lower = class_list.to_lowercase();
        for p in &noise_patterns {
            if class_lower.contains(p) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_removes_nav() {
        let html = "<html><body><nav><a href=\"/\">Home</a></nav><article><h1>Article Title</h1><p>Content here.</p></article></body></html>";
        let result = extract_main_content(html);
        assert!(!result.contains("Home"));
        assert!(result.contains("Article Title"));
        assert!(result.contains("Content here"));
    }

    #[test]
    fn test_preserves_article() {
        let html = "<html><body><article><h1>Main Article</h1><p>This is the main content of the page with enough text to score well.</p><p>Second paragraph with more content.</p><p>Third paragraph.</p><p>Fourth paragraph.</p></article></body></html>";
        let result = extract_main_content(html);
        assert!(result.contains("Main Article"));
        assert!(result.contains("main content"));
    }

    #[test]
    fn test_text_density_wins() {
        let html = "<html><body><div class=\"nav\"><a>Link</a><a>Link</a></div><div class=\"content\"><h2>Real Content</h2><p>This is real paragraph text.</p><p>More text here.</p><p>Even more text.</p><p>Lots of text content.</p></div></body></html>";
        let result = extract_main_content(html);
        assert!(result.contains("Real Content"));
    }
}
