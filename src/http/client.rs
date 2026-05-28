use crate::utils::config::Config;
use reqwest::Client;

/// Cliente HTTP do FAF Browser
pub struct HttpClient {
    client: Client,
    config: Config,
}

impl HttpClient {
    /// Cria um novo HttpClient com base na config
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let mut builder = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .user_agent(&config.user_agent)
            .danger_accept_invalid_certs(false);

        // Configurar proxy se fornecido
        if let Some(ref proxy_url) = config.proxy {
            let proxy = if proxy_url.starts_with("socks5") {
                reqwest::Proxy::all(proxy_url)?
            } else {
                reqwest::Proxy::http(proxy_url)?
            };
            builder = builder.proxy(proxy);
        }

        // Habilitar cookies
        builder = builder.cookie_store(true);

        let client = builder.build()?;
        Ok(Self { client, config })
    }

    /// Faz uma requisição GET e retorna o body como string
    pub async fn get(&self, url: &str) -> anyhow::Result<String> {
        let response = self.client.get(url).send().await?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("HTTP {} ao acessar {}", status, url);
        }

        let body = response.text().await?;
        Ok(body)
    }

    /// Faz uma requisição GET com headers customizados
    pub async fn get_with_headers(
        &self,
        url: &str,
        headers: Vec<(&str, &str)>,
    ) -> anyhow::Result<String> {
        let mut req = self.client.get(url);
        for (key, value) in headers {
            req = req.header(key, value);
        }
        let response = req.send().await?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("HTTP {} ao acessar {}", status, url);
        }

        let body = response.text().await?;
        Ok(body)
    }

    /// Retorna referência à config
    pub fn config(&self) -> &Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = Config::default();
        let client = HttpClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_proxy() {
        let config = Config {
            proxy: Some("socks5://127.0.0.1:9050".to_string()),
            ..Config::default()
        };
        let client = HttpClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_user_agent_default() {
        let config = Config::default();
        assert!(config.user_agent.contains("Chrome/125"));
    }
}
