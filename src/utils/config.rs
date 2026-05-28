/// Configuração do FAF Browser
#[derive(Debug, Clone)]
pub struct Config {
    pub user_agent: String,
    pub timeout_secs: u64,
    pub proxy: Option<String>,
    pub retries: u64,
    pub retry_delay_ms: u64,
    pub cookies_path: Option<String>,
    pub cookies_jar_path: Option<String>,
    pub cache_dir: Option<String>,
    pub cache_ttl_secs: u64,
    pub no_cache: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_agent: concat!(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) ",
                "AppleWebKit/537.36 (KHTML, like Gecko) ",
                "Chrome/125.0.0.0 Safari/537.36"
            )
            .to_string(),
            timeout_secs: 30,
            proxy: None,
            retries: 0,
            retry_delay_ms: 1000,
            cookies_path: None,
            cookies_jar_path: None,
            cache_dir: None,
            cache_ttl_secs: 300,
            no_cache: false,
        }
    }
}
