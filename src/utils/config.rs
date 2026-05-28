/// Configuração do FAF Browser
#[derive(Debug, Clone)]
pub struct Config {
    pub user_agent: String,
    pub timeout_secs: u64,
    pub proxy: Option<String>,
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
        }
    }
}
