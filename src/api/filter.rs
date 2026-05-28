use crate::dom::QueryResult;
use regex::Regex;

pub struct QueryFilter {
    pub field: String,
    pub operator: String,
    pub value: String,
}

impl QueryFilter {
    /// Parseia string de filtro no formato "campo~=valor", "campo==valor", etc.
    /// Se não tem operador, assume `text~=valor`.
    pub fn parse(input: &str) -> Result<Self, String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err("Filtro vazio".to_string());
        }

        let operators = ["!~=", "!^=", "!$=", "~=", "==", "!=", "^=", "$=", "="];
        for op in &operators {
            if let Some(pos) = trimmed.find(op) {
                let field = trimmed[..pos].trim().to_string();
                let value = trimmed[pos + op.len()..].trim().to_string();
                if field.is_empty() {
                    return Err(format!("Campo vazio no filtro '{}'", input));
                }
                return Ok(Self {
                    field,
                    operator: op.to_string(),
                    value,
                });
            }
        }

        // Sem operador: assume text~=
        Ok(Self {
            field: "text".to_string(),
            operator: "~=".to_string(),
            value: trimmed.to_string(),
        })
    }

    /// Obtém o valor do campo a partir do QueryResult.
    fn get_field_value(&self, result: &QueryResult) -> Option<String> {
        match self.field.as_str() {
            "text" => Some(result.text.clone()),
            "tag" => Some(result.tag.clone()),
            "id" => result.id.clone(),
            "class" => Some(result.classes.join(" ")),
            _ => result
                .attributes
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(&self.field))
                .map(|(_, v)| v.clone()),
        }
    }

    /// Verifica se um QueryResult passa neste filtro.
    pub fn matches(&self, result: &QueryResult) -> bool {
        let Some(haystack) = self.get_field_value(result) else {
            return false;
        };

        match self.operator.as_str() {
            "~=" => haystack.to_lowercase().contains(&self.value.to_lowercase()),
            "!~=" => !haystack.to_lowercase().contains(&self.value.to_lowercase()),
            "==" => haystack == self.value,
            "!=" => haystack != self.value,
            "^=" => haystack.starts_with(&self.value),
            "!^=" => !haystack.starts_with(&self.value),
            "$=" => haystack.ends_with(&self.value),
            "!$=" => !haystack.ends_with(&self.value),
            "=" => {
                // Se parece regex, tenta usar regex; senão, substring
                if looks_like_regex(&self.value) {
                    match Regex::new(&self.value) {
                        Ok(re) => re.is_match(&haystack),
                        Err(_) => haystack.contains(&self.value),
                    }
                } else {
                    haystack.contains(&self.value)
                }
            }
            _ => false,
        }
    }
}

/// Aplica múltiplos filtros (AND). Retorna só os resultados que passam todos.
pub fn apply_filters(results: Vec<QueryResult>, filters: &[QueryFilter]) -> Vec<QueryResult> {
    if filters.is_empty() {
        return results;
    }
    results
        .into_iter()
        .filter(|r| filters.iter().all(|f| f.matches(r)))
        .collect()
}

fn looks_like_regex(s: &str) -> bool {
    s.contains('.') || s.contains('*') || s.contains('+') || s.contains('[') || s.contains('(')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(
        tag: &str,
        id: Option<&str>,
        classes: &[&str],
        text: &str,
        attrs: &[(&str, &str)],
    ) -> QueryResult {
        QueryResult {
            tag: tag.to_string(),
            id: id.map(|s| s.to_string()),
            classes: classes.iter().map(|s| s.to_string()).collect(),
            attributes: attrs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            text: text.to_string(),
            inner_html: text.to_string(),
            html: format!("<{}>{}</{}>", tag, text, tag),
        }
    }

    #[test]
    fn test_parse_basic() {
        let f = QueryFilter::parse("text~=hello").unwrap();
        assert_eq!(f.field, "text");
        assert_eq!(f.operator, "~=");
        assert_eq!(f.value, "hello");
    }

    #[test]
    fn test_parse_no_operator() {
        let f = QueryFilter::parse("hello").unwrap();
        assert_eq!(f.field, "text");
        assert_eq!(f.operator, "~=");
        assert_eq!(f.value, "hello");
    }

    #[test]
    fn test_parse_all_operators() {
        let ops = ["!~=", "!^=", "!$=", "~=", "==", "!=", "^=", "$=", "="];
        for op in &ops {
            let f = QueryFilter::parse(&format!("href{}https", op)).unwrap();
            assert_eq!(f.operator, *op);
        }
    }

    #[test]
    fn test_matches_not_contains() {
        let r_cat = make_result("a", None, &[], "Link", &[("href", "category/books")]);
        let r_cata = make_result("a", None, &[], "Link", &[("href", "catalogue/books")]);
        assert!(!QueryFilter::parse("href!~=category").unwrap().matches(&r_cat));
        assert!(QueryFilter::parse("href!~=category").unwrap().matches(&r_cata));
    }

    #[test]
    fn test_matches_not_starts_with() {
        let r = make_result("a", None, &[], "Link", &[("href", "/relative/path")]);
        assert!(QueryFilter::parse("href!^=https").unwrap().matches(&r));
        assert!(!QueryFilter::parse("href!^=/relative").unwrap().matches(&r));
    }

    #[test]
    fn test_matches_not_ends_with() {
        let r = make_result("img", None, &[], "Image", &[("src", "image.png")]);
        assert!(QueryFilter::parse("src!$=.svg").unwrap().matches(&r));
        assert!(!QueryFilter::parse("src!$=.png").unwrap().matches(&r));
    }

    #[test]
    fn test_apply_filters_with_negative() {
        let r1 = make_result("a", None, &[], "Books", &[("href", "/catalogue/books")]);
        let r2 = make_result("a", None, &[], "Travel", &[("href", "/category/travel")]);
        let results = vec![r1, r2];
        let filters = vec![
            QueryFilter::parse("href~=books").unwrap(),
            QueryFilter::parse("href!~=category").unwrap(),
        ];
        let filtered = apply_filters(results, &filters);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].text, "Books");
    }

    #[test]
    fn test_matches_text_substring() {
        let r = make_result("p", None, &[], "Hello World", &[]);
        assert!(QueryFilter::parse("text~=world").unwrap().matches(&r));
        assert!(QueryFilter::parse("text~=World").unwrap().matches(&r));
        assert!(!QueryFilter::parse("text~=foo").unwrap().matches(&r));
    }

    #[test]
    fn test_matches_exact() {
        let r = make_result("p", None, &[], "Exact", &[]);
        assert!(QueryFilter::parse("text==Exact").unwrap().matches(&r));
        assert!(!QueryFilter::parse("text==exact").unwrap().matches(&r));
    }

    #[test]
    fn test_matches_not_equal() {
        let r = make_result("p", None, &[], "Foo", &[]);
        assert!(QueryFilter::parse("text!=Bar").unwrap().matches(&r));
        assert!(!QueryFilter::parse("text!=Foo").unwrap().matches(&r));
    }

    #[test]
    fn test_matches_starts_ends_with() {
        let r = make_result("a", None, &[], "Link", &[("href", "https://example.com")]);
        assert!(QueryFilter::parse("href^=https").unwrap().matches(&r));
        assert!(!QueryFilter::parse("href^=httpz").unwrap().matches(&r));
        assert!(QueryFilter::parse("href$=.com").unwrap().matches(&r));
        assert!(!QueryFilter::parse("href$=.org").unwrap().matches(&r));
    }

    #[test]
    fn test_matches_regex() {
        let r = make_result(
            "a",
            None,
            &[],
            "Link",
            &[("href", "https://example.com/page")],
        );
        assert!(QueryFilter::parse("href=.*/page").unwrap().matches(&r));
        assert!(!QueryFilter::parse("href=.*/other").unwrap().matches(&r));
    }

    #[test]
    fn test_matches_tag_and_id() {
        let r = make_result("div", Some("main"), &["box"], "Content", &[]);
        assert!(QueryFilter::parse("tag==div").unwrap().matches(&r));
        assert!(QueryFilter::parse("id==main").unwrap().matches(&r));
        assert!(!QueryFilter::parse("id==other").unwrap().matches(&r));
    }

    #[test]
    fn test_matches_class() {
        let r = make_result("div", None, &["foo", "bar"], "Text", &[]);
        assert!(QueryFilter::parse("class~=foo").unwrap().matches(&r));
        assert!(QueryFilter::parse("class~=bar").unwrap().matches(&r));
        assert!(!QueryFilter::parse("class~=baz").unwrap().matches(&r));
    }

    #[test]
    fn test_apply_filters_and() {
        let r1 = make_result("p", None, &[], "Hello World", &[]);
        let r2 = make_result("p", None, &[], "Goodbye", &[]);
        let results = vec![r1, r2];
        let filters = vec![
            QueryFilter::parse("text~=Hello").unwrap(),
            QueryFilter::parse("tag==p").unwrap(),
        ];
        let filtered = apply_filters(results, &filters);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].text, "Hello World");
    }

    #[test]
    fn test_missing_attribute_no_match() {
        let r = make_result("p", None, &[], "Text", &[]);
        assert!(!QueryFilter::parse("href^=https").unwrap().matches(&r));
    }

    #[test]
    fn test_empty_filters_returns_all() {
        let r = make_result("p", None, &[], "Text", &[]);
        let results = vec![r];
        let filtered = apply_filters(results, &[]);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_invalid_regex_falls_back_to_substring() {
        let r = make_result("p", None, &[], "[abc", &[]);
        // '[' alone looks like regex but isn't valid
        let f = QueryFilter::parse("text=[abc").unwrap();
        assert!(f.matches(&r));
    }
}
