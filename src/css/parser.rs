use crate::http::client::HttpClient;
use anyhow::Result;
use cssparser::{Parser, ParserInput, ToCss, Token};
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub selectors: String,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: String,
    pub value: String,
}

/// Parse a CSS stylesheet into a structured representation.
///
/// Uses `cssparser` for tokenizing declaration blocks and manual scanning
/// for top-level rules (selectors + `{…}` blocks) so that selectors are
/// reconstructed exactly as written.
pub fn parse_css(css_text: &str) -> Result<Stylesheet> {
    let mut rules = Vec::new();
    let mut media_css = String::new(); // M8.5: collect inner CSS from @media rules
    let chars: Vec<char> = css_text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Skip whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }

        // Skip CSS comments: /* ... */
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2; // skip */
            continue;
        }

        // M8.5: Process @media rules instead of skipping them
        if chars[i] == '@' {
            let result = extract_media_rules(&chars, i, css_text)?;
            i = result.0;
            if let Some(inner_css) = result.1 {
                // Append inner CSS to be parsed with the rest
                media_css.push_str(&inner_css);
                media_css.push('\n');
            }
            continue;
        }

        // Read selector string until '{'
        let selector_start = i;
        let mut brace_depth = 0;
        while i < chars.len() && (chars[i] != '{' || brace_depth > 0) {
            if chars[i] == '\'' || chars[i] == '"' {
                let quote = chars[i];
                i += 1;
                while i < chars.len() && chars[i] != quote {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if i < chars.len() {
                    i += 1;
                }
                continue;
            }
            if chars[i] == '{' {
                brace_depth += 1;
            } else if chars[i] == '}' && brace_depth > 0 {
                brace_depth -= 1;
            }
            i += 1;
        }
        if i >= chars.len() {
            break; // No opening brace found
        }
        let selectors = css_text[selector_start..i].trim().to_string();
        i += 1; // skip '{'

        // Read declarations until matching '}'
        let decl_start = i;
        brace_depth = 1;
        while i < chars.len() && brace_depth > 0 {
            if chars[i] == '\'' || chars[i] == '"' {
                let quote = chars[i];
                i += 1;
                while i < chars.len() && chars[i] != quote {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if i < chars.len() {
                    i += 1;
                }
                continue;
            }
            if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
                // Skip comment inside declaration block
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
            if chars[i] == '{' {
                brace_depth += 1;
            } else if chars[i] == '}' {
                brace_depth -= 1;
            }
            if brace_depth > 0 {
                i += 1;
            }
        }
        let decl_text = css_text[decl_start..i].trim();
        i += 1; // skip '}'

        let declarations = parse_declarations(decl_text)?;
        rules.push(Rule {
            selectors,
            declarations,
        });
    }

    // M8.5: Parse CSS extracted from @media rules (desktop-first: min-width queries)
    if !media_css.is_empty() {
        let media_sheet = parse_css(&media_css)?;
        rules.extend(media_sheet.rules);
    }

    Ok(Stylesheet { rules })
}

/// M8.5: Extract inner CSS from @media rules that match our viewport.
/// Returns (new_index, optional_inner_css).
/// For now, extracts all @media screen rules (assumes desktop viewport >= 768px).
fn extract_media_rules(chars: &[char], mut i: usize, css_text: &str) -> Result<(usize, Option<String>)> {
    let start = i;
    // Skip "@media" token
    while i < chars.len() && chars[i] != '{' && chars[i] != ';' {
        i += 1;
    }
    let at_rule = css_text[start..i].trim().to_string();
    
    if i < chars.len() && chars[i] == '{' {
        // Find matching closing brace
        let mut depth = 1;
        let body_start = i + 1;
        i += 1;
        while i < chars.len() && depth > 0 {
            if chars[i] == '{' {
                depth += 1;
            } else if chars[i] == '}' {
                depth -= 1;
            }
            i += 1;
        }
        let body_end = i - 1; // position of closing '}'
        
        // Extract rules: only @media (screen), skip @keyframes, @font-face, @import
        let at_lower = at_rule.to_lowercase();
        if !at_lower.starts_with("@media") {
            return Ok((i, None)); // skip non-media at-rules
        }
        let is_print = at_rule.contains("print");
        if !is_print {
            let inner = css_text[body_start..body_end].to_string();
            return Ok((i, Some(inner)));
        }
        return Ok((i, None));
    }
    
    // No body (like @import), skip to ;
    while i < chars.len() && chars[i] != ';' {
        i += 1;
    }
    if i < chars.len() { i += 1; }
    Ok((i, None))
}

fn parse_declarations(decl_text: &str) -> Result<Vec<Declaration>> {
    let mut input = ParserInput::new(decl_text);
    let mut parser = Parser::new(&mut input);
    let mut declarations = Vec::new();

    while !parser.is_exhausted() {
        // Get property name
        let property = match parser.next() {
            Ok(Token::Ident(name)) => name.to_string(),
            Ok(Token::Semicolon) => continue,
            Ok(Token::WhiteSpace(_)) => continue,
            Ok(_) => {
                // Unexpected token, skip until semicolon
                skip_until_semicolon(&mut parser);
                continue;
            }
            Err(_) => break,
        };

        // Expect colon
        match parser.next() {
            Ok(Token::Colon) => {}
            Ok(Token::Semicolon) => continue,
            Err(_) => break,
            Ok(_) => {
                skip_until_semicolon(&mut parser);
                continue;
            }
        }

        // Collect value tokens until semicolon or end of block
        let mut value_parts = Vec::new();
        while let Ok(token) = parser.next() {
            if let Token::Semicolon = token {
                break;
            }
            value_parts.push(token.to_css_string());
        }

        let value = value_parts.join(" ").trim().to_string();
        if !property.is_empty() && !value.is_empty() {
            declarations.push(Declaration { property, value });
        }
    }

    Ok(declarations)
}

fn skip_until_semicolon(parser: &mut Parser) {
    while let Ok(token) = parser.next() {
        if let Token::Semicolon = token {
            break;
        }
    }
}

/// Extrai CSS de tags `<style>` e `<link rel="stylesheet">` da página.
/// Retorna o CSS concatenado de todas as fontes.
pub async fn extract_page_stylesheets(
    doc: &scraper::Html,
    base_url: &Url,
    client: &HttpClient,
) -> Vec<String> {
    let mut css_sources = Vec::new();

    // 1. Extrair conteúdo de tags <style>
    if let Ok(style_selector) = scraper::Selector::parse("style") {
        for el in doc.select(&style_selector) {
            let css_text: String = el.text().collect();
            if !css_text.trim().is_empty() {
                css_sources.push(css_text);
            }
        }
    }

    // 2. Extrair href de tags <link rel="stylesheet">
    if let Ok(link_selector) = scraper::Selector::parse("link[rel~=stylesheet]") {
        for el in doc.select(&link_selector) {
            if let Some(href) = el.value().attr("href") {
                let resolved_url = match base_url.join(href) {
                    Ok(u) => u.to_string(),
                    Err(err) => {
                        log::warn!("Falha ao resolver URL do stylesheet '{}': {}", href, err);
                        continue;
                    }
                };
                match client.get(&resolved_url).await {
                    Ok(resp) => {
                        let css_text = resp.body;
                        if !css_text.trim().is_empty() {
                            css_sources.push(css_text);
                        }
                    }
                    Err(err) => {
                        log::warn!("Falha ao baixar CSS '{}': {}", resolved_url, err);
                    }
                }
            }
        }
    }

    css_sources
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rule() {
        let css = "h1 { color: red; }";
        let sheet = parse_css(css).unwrap();
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selectors, "h1");
        assert_eq!(sheet.rules[0].declarations.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].property, "color");
        assert_eq!(sheet.rules[0].declarations[0].value, "red");
    }

    #[test]
    fn test_parse_class_selector() {
        let css = ".class { font-size: 14px; }";
        let sheet = parse_css(css).unwrap();
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selectors, ".class");
        assert_eq!(sheet.rules[0].declarations.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].property, "font-size");
        assert_eq!(sheet.rules[0].declarations[0].value, "14px");
    }

    #[test]
    fn test_parse_id_selector() {
        let css = "#id { margin: 0; }";
        let sheet = parse_css(css).unwrap();
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selectors, "#id");
        assert_eq!(sheet.rules[0].declarations.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].property, "margin");
        assert_eq!(sheet.rules[0].declarations[0].value, "0");
    }

    #[test]
    fn test_parse_descendant_selector() {
        let css = "div span { display: block; }";
        let sheet = parse_css(css).unwrap();
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selectors, "div span");
        assert_eq!(sheet.rules[0].declarations.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].property, "display");
        assert_eq!(sheet.rules[0].declarations[0].value, "block");
    }

    #[test]
    fn test_parse_multiple_declarations() {
        let css = "p { color: blue; font-size: 12px; margin: 0; }";
        let sheet = parse_css(css).unwrap();
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].declarations.len(), 3);
        assert_eq!(sheet.rules[0].declarations[0].property, "color");
        assert_eq!(sheet.rules[0].declarations[0].value, "blue");
        assert_eq!(sheet.rules[0].declarations[1].property, "font-size");
        assert_eq!(sheet.rules[0].declarations[1].value, "12px");
        assert_eq!(sheet.rules[0].declarations[2].property, "margin");
        assert_eq!(sheet.rules[0].declarations[2].value, "0");
    }

    #[test]
    fn test_parse_comments() {
        let css = "/* header styles */ h1 { color: red; } /* end */";
        let sheet = parse_css(css).unwrap();
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selectors, "h1");
    }

    #[test]
    fn test_ignore_at_rules_media() {
        let css = "@media screen { h1 { color: red; } } p { color: blue; }";
        let sheet = parse_css(css).unwrap();
        // M8.5: @media inner rules are now extracted, so we get 2 rules (h1 + p)
        assert_eq!(sheet.rules.len(), 2);
        // h1 comes from media, p is direct
        assert!(sheet.rules.iter().any(|r| r.selectors.contains("h1")));
        assert!(sheet.rules.iter().any(|r| r.selectors.contains("p")));
    }

    #[test]
    fn test_ignore_import() {
        let css = "@import url('style.css'); h1 { color: red; }";
        let sheet = parse_css(css).unwrap();
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selectors, "h1");
    }

    #[test]
    fn test_ignore_keyframes() {
        let css =
            "@keyframes fade { from { opacity: 0; } to { opacity: 1; } } div { opacity: 0.5; }";
        let sheet = parse_css(css).unwrap();
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selectors, "div");
        assert_eq!(sheet.rules[0].declarations[0].property, "opacity");
        assert_eq!(sheet.rules[0].declarations[0].value, "0.5");
    }

    #[test]
    fn test_ignore_font_face() {
        let css = "@font-face { font-family: 'Custom'; src: url('font.woff'); } body { font-family: sans-serif; }";
        let sheet = parse_css(css).unwrap();
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selectors, "body");
        assert_eq!(sheet.rules[0].declarations[0].property, "font-family");
    }

    #[test]
    fn test_multiple_rules() {
        let css = "h1 { color: red; } h2 { color: blue; }";
        let sheet = parse_css(css).unwrap();
        assert_eq!(sheet.rules.len(), 2);
        assert_eq!(sheet.rules[0].selectors, "h1");
        assert_eq!(sheet.rules[1].selectors, "h2");
    }

    #[test]
    fn test_empty_stylesheet() {
        let css = "";
        let sheet = parse_css(css).unwrap();
        assert!(sheet.rules.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let css = "   \n\t  ";
        let sheet = parse_css(css).unwrap();
        assert!(sheet.rules.is_empty());
    }

    #[test]
    fn test_comma_separated_selectors() {
        let css = "h1, h2, h3 { color: black; }";
        let sheet = parse_css(css).unwrap();
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selectors, "h1, h2, h3");
        assert_eq!(sheet.rules[0].declarations.len(), 1);
    }

    #[test]
    fn test_comment_inside_block() {
        let css = "p { /* text color */ color: green; }";
        let sheet = parse_css(css).unwrap();
        assert_eq!(sheet.rules[0].declarations.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].value, "green");
    }

    #[test]
    fn test_no_trailing_semicolon() {
        let css = "p { color: red }";
        let sheet = parse_css(css).unwrap();
        assert_eq!(sheet.rules[0].declarations.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].value, "red");
    }
}
