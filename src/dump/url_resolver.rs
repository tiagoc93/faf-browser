pub fn resolve_url(src: &str, base_url: &str) -> String {
    if src.starts_with("http://")
        || src.starts_with("https://")
        || src.starts_with("data:")
        || src.starts_with("javascript:")
        || src.starts_with("mailto:")
        || src.starts_with("tel:")
        || src.starts_with("#")
    {
        return src.to_string();
    }
    if src.is_empty() {
        return src.to_string();
    }
    if let Ok(base) = url::Url::parse(base_url) {
        if let Ok(resolved) = base.join(src) {
            return resolved.to_string();
        }
    }
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        src.trim_start_matches('/')
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_absolute_already() {
        let result = resolve_url("https://example.com/page", "https://other.com");
        assert_eq!(result, "https://example.com/page");
    }

    #[test]
    fn test_resolve_relative_to_base() {
        let result = resolve_url("/about", "https://site.com/blog/");
        assert_eq!(result, "https://site.com/about");
    }

    #[test]
    fn test_resolve_relative_path() {
        let result = resolve_url("../style.css", "https://site.com/dir/page.html");
        assert_eq!(result, "https://site.com/style.css");
    }

    #[test]
    fn test_resolve_same_directory() {
        let result = resolve_url("img/logo.png", "https://site.com/page/");
        assert_eq!(result, "https://site.com/page/img/logo.png");
    }

    #[test]
    fn test_preserve_data_uri() {
        let result = resolve_url("data:image/png;base64,abc123", "https://site.com");
        assert_eq!(result, "data:image/png;base64,abc123");
    }

    #[test]
    fn test_preserve_anchor() {
        let result = resolve_url("#section", "https://site.com");
        assert_eq!(result, "#section");
    }

    #[test]
    fn test_preserve_empty() {
        let result = resolve_url("", "https://site.com");
        assert_eq!(result, "");
    }
}
