/// Erros específicos do FAF Browser
#[derive(Debug, thiserror::Error)]
pub enum FafError {
    #[error("Erro HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Erro de parsing HTML: {0}")]
    HtmlParse(String),

    #[error("Erro de seletor CSS: {0}")]
    CssSelector(String),

    #[error("Erro JavaScript: {0}")]
    Js(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Elemento não encontrado: {0}")]
    ElementNotFound(String),

    #[error("Erro de configuração: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, anyhow::Error>;
