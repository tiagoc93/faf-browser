use crate::utils::config::Config;
use reqwest::Client;
use std::collections::HashMap;

/// Resposta de uma requisição HTTP
#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Cliente HTTP do FAF Browser
#[derive(Clone)]
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

    /// Executa uma requisição GET com retry e exponential backoff
    pub async fn get(&self, url: &str) -> anyhow::Result<FetchResponse> {
        self.get_with_retry(url, Vec::new()).await
    }

    /// Faz uma requisição GET com headers customizados e retry
    pub async fn get_with_headers(
        &self,
        url: &str,
        headers: Vec<(&str, &str)>,
    ) -> anyhow::Result<FetchResponse> {
        self.get_with_retry(url, headers).await
    }

    /// Lógica central de GET com retry exponencial
    async fn get_with_retry(
        &self,
        url: &str,
        headers: Vec<(&str, &str)>,
    ) -> anyhow::Result<FetchResponse> {
        let max_retries = self.config.retries;
        let mut delay_ms = self.config.retry_delay_ms;

        for attempt in 0..=max_retries {
            let mut req = self.client.get(url);
            for (key, value) in &headers {
                req = req.header(*key, *value);
            }

            match req.send().await {
                Ok(response) => {
                    let status = response.status();

                    // HTTP 429: respeitar header Retry-After se presente
                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS
                        && attempt < max_retries
                        && let Some(retry_after) = response.headers().get("retry-after")
                            && let Ok(retry_str) = retry_after.to_str()
                                && let Ok(secs) = retry_str.parse::<u64>() {
                                    log::warn!(
                                        "HTTP 429 em {}, aguardando {}s (Retry-After)",
                                        url,
                                        secs
                                    );
                                    tokio::time::sleep(std::time::Duration::from_secs(secs))
                                        .await;
                                    continue;
                                }
                    if !status.is_success() {
                        if attempt < max_retries && Self::is_retryable(status) {
                            log::warn!(
                                "Request falhou (tentativa {}/{}), retry em {}ms",
                                attempt + 1,
                                max_retries,
                                delay_ms
                            );
                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms))
                                .await;
                            delay_ms *= 2;
                            continue;
                        }
                        anyhow::bail!("HTTP {} ao acessar {}", status, url);
                    }

                    let status_u16 = status.as_u16();
                    let status_text = status.canonical_reason().unwrap_or("").to_string();
                    let headers_map: HashMap<String, String> = response
                        .headers()
                        .iter()
                        .map(|(k, v)| {
                            (k.to_string(), v.to_str().unwrap_or("").to_string())
                        })
                        .collect();
                    let body = response.text().await?;
                    return Ok(FetchResponse {
                        status: status_u16,
                        status_text,
                        headers: headers_map,
                        body,
                    });
                }
                Err(e) => {
                    // Erro de rede
                    if attempt < max_retries {
                        log::warn!(
                            "Request falhou (tentativa {}/{}), retry em {}ms: {}",
                            attempt + 1,
                            max_retries,
                            delay_ms,
                            e
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                        delay_ms *= 2;
                        continue;
                    }
                    return Err(anyhow::anyhow!("Erro de rede ao acessar {}: {}", url, e));
                }
            }
        }

        anyhow::bail!("Excedido número de retries para {}", url)
    }

    /// Determina se um status HTTP deve ser retentado
    fn is_retryable(status: reqwest::StatusCode) -> bool {
        status.is_server_error()
            || status == reqwest::StatusCode::TOO_MANY_REQUESTS
            || status == reqwest::StatusCode::REQUEST_TIMEOUT
    }

    /// Retorna referência à config
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Retorna uma cópia do cliente reqwest interno
    pub fn inner_client(&self) -> reqwest::Client {
        self.client.clone()
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

    #[test]
    fn test_retryable_statuses() {
        assert!(HttpClient::is_retryable(reqwest::StatusCode::INTERNAL_SERVER_ERROR));
        assert!(HttpClient::is_retryable(reqwest::StatusCode::BAD_GATEWAY));
        assert!(HttpClient::is_retryable(reqwest::StatusCode::SERVICE_UNAVAILABLE));
        assert!(HttpClient::is_retryable(reqwest::StatusCode::TOO_MANY_REQUESTS));
        assert!(HttpClient::is_retryable(reqwest::StatusCode::REQUEST_TIMEOUT));
        assert!(!HttpClient::is_retryable(reqwest::StatusCode::NOT_FOUND));
        assert!(!HttpClient::is_retryable(reqwest::StatusCode::BAD_REQUEST));
    }
}
