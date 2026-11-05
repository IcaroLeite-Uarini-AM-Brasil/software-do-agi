pub mod modules;

use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::{stream, StreamExt};
use reqwest::{Client, StatusCode};
use url::Url;

/// Um documento baixado. O conteúdo permanece em bytes para evitar corrupção
/// silenciosa de fontes que não estejam em UTF-8; a decodificação e a política
/// de tipo de conteúdo serão tratadas em uma etapa posterior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDocument {
    pub source_url: String,
    pub content: Vec<u8>,
    pub fetched_at_timestamp: u64,
}

/// Limites explícitos de recursos e os hosts HTTPS exatos que o coletor pode
/// acessar. A correspondência de host é exata; subdomínios devem ser listados separadamente.
#[derive(Debug, Clone)]
pub struct HarvesterConfig {
    pub allowed_hosts: HashSet<String>,
    pub max_concurrency: usize,
    pub max_sources_per_batch: usize,
    pub max_response_bytes: usize,
    pub request_timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarvestError {
    InvalidConfiguration(&'static str),
    InvalidUrl,
    HostNotAllowed,
    RequestFailed(String),
    HttpStatus(u16),
    ResponseTooLarge,
    ClockError,
    TooManySources { supplied: usize, limit: usize },
}

impl fmt::Display for HarvestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration(message) => write!(f, "configuração inválida do coletor: {message}"),
            Self::InvalidUrl => write!(f, "URL inválida ou não suportada (HTTPS obrigatório)"),
            Self::HostNotAllowed => write!(f, "o host da URL não está na lista de permissões configurada"),
            Self::RequestFailed(message) => write!(f, "falha na requisição HTTP: {message}"),
            Self::HttpStatus(code) => write!(f, "a fonte retornou o status HTTP {code}"),
            Self::ResponseTooLarge => write!(f, "a resposta excede o limite de bytes configurado"),
            Self::ClockError => write!(f, "o relógio do sistema está antes da época UNIX"),
            Self::TooManySources { supplied, limit } => {
                write!(f, "o lote contém {supplied} fontes; o limite configurado é {limit}")
            }
        }
    }
}

impl Error for HarvestError {}

pub struct AsyncHarvester {
    client: Client,
    config: HarvesterConfig,
}

impl AsyncHarvester {
    pub fn new(mut config: HarvesterConfig) -> Result<Self, HarvestError> {
        if config.max_concurrency == 0 {
            return Err(HarvestError::InvalidConfiguration("max_concurrency deve ser maior que zero"));
        }
        if config.max_sources_per_batch == 0 {
            return Err(HarvestError::InvalidConfiguration("max_sources_per_batch deve ser maior que zero"));
        }
        if config.max_response_bytes == 0 {
            return Err(HarvestError::InvalidConfiguration("max_response_bytes deve ser maior que zero"));
        }
        if config.request_timeout.is_zero() {
            return Err(HarvestError::InvalidConfiguration("request_timeout deve ser maior que zero"));
        }
        if config.allowed_hosts.is_empty() {
            return Err(HarvestError::InvalidConfiguration("allowed_hosts não pode estar vazio"));
        }

        config.allowed_hosts = config
            .allowed_hosts
            .into_iter()
            .map(|host| host.trim().trim_end_matches('.').to_ascii_lowercase())
            .filter(|host| !host.is_empty())
            .collect();
        if config.allowed_hosts.is_empty() {
            return Err(HarvestError::InvalidConfiguration("allowed_hosts deve conter um nome de host"));
        }

        // Não seguir redirecionamentos: o destino precisaria ser validado na
        // lista de permissões antes que uma nova requisição fosse feita.
        let client = Client::builder()
            .timeout(config.request_timeout)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("software-do-agi-ingestion/0.1 (+operator-configured sources)")
            .build()
            .map_err(|error| HarvestError::RequestFailed(error.to_string()))?;

        Ok(Self { client, config })
    }

    /// Busca somente as URLs fornecidas explicitamente pelo chamador. Isto não
    /// é um rastreador: links não são descobertos e a lista de fontes não é ampliada.
    /// A concorrência e o tamanho do lote são limitados pela configuração.
    pub async fn fetch_many(&self, urls: &[String]) -> Result<Vec<Result<RawDocument, HarvestError>>, HarvestError> {
        if urls.len() > self.config.max_sources_per_batch {
            return Err(HarvestError::TooManySources {
                supplied: urls.len(),
                limit: self.config.max_sources_per_batch,
            });
        }

        let mut results = stream::iter(urls.iter().enumerate().map(|(index, url)| async move {
            (index, self.fetch_one(url).await)
        }))
        .buffer_unordered(self.config.max_concurrency)
        .collect::<Vec<_>>()
        .await;

        results.sort_unstable_by_key(|(index, _)| *index);
        Ok(results.into_iter().map(|(_, result)| result).collect())
    }

    async fn fetch_one(&self, source: &str) -> Result<RawDocument, HarvestError> {
        let url = Url::parse(source).map_err(|_| HarvestError::InvalidUrl)?;
        if url.scheme() != "https" || url.username() != "" || url.password().is_some() {
            return Err(HarvestError::InvalidUrl);
        }
        let host = url.host_str().ok_or(HarvestError::InvalidUrl)?.trim_end_matches('.').to_ascii_lowercase();
        if !self.config.allowed_hosts.contains(&host) {
            return Err(HarvestError::HostNotAllowed);
        }

        let mut response = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(|error| HarvestError::RequestFailed(error.to_string()))?;
        if response.status() != StatusCode::OK {
            return Err(HarvestError::HttpStatus(response.status().as_u16()));
        }
        if response
            .content_length()
            .is_some_and(|length| length > self.config.max_response_bytes as u64)
        {
            return Err(HarvestError::ResponseTooLarge);
        }

        let mut content = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| HarvestError::RequestFailed(error.to_string()))?
        {
            if content.len().saturating_add(chunk.len()) > self.config.max_response_bytes {
                return Err(HarvestError::ResponseTooLarge);
            }
            content.extend_from_slice(&chunk);
        }

        let fetched_at_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| HarvestError::ClockError)?
            .as_secs();

        Ok(RawDocument {
            source_url: url.into(),
            content,
            fetched_at_timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_harvester() -> AsyncHarvester {
        let config = HarvesterConfig {
            allowed_hosts: ["example.org".to_owned()].into_iter().collect(),
            max_concurrency: 2,
            max_sources_per_batch: 4,
            max_response_bytes: 1024,
            request_timeout: Duration::from_secs(2),
        };
        AsyncHarvester::new(config).expect("configuração de teste válida")
    }

    #[test]
    fn rejects_non_https_urls() {
        assert_eq!(test_harvester().validate_source("http://example.org/a"), Err(HarvestError::InvalidUrl));
    }

    #[test]
    fn requires_exact_allowlisted_host() {
        assert_eq!(test_harvester().validate_source("https://sub.example.org/a"), Err(HarvestError::HostNotAllowed));
    }

    #[test]
    fn accepts_allowlisted_https_url() {
        assert!(test_harvester().validate_source("https://example.org/a").is_ok());
    }

    #[test]
    fn rejects_zero_concurrency() {
        let config = HarvesterConfig {
            allowed_hosts: ["example.org".to_owned()].into_iter().collect(),
            max_concurrency: 0,
            max_sources_per_batch: 4,
            max_response_bytes: 1024,
            request_timeout: Duration::from_secs(2),
        };
        assert!(matches!(AsyncHarvester::new(config), Err(HarvestError::InvalidConfiguration(_))));
    }

    impl AsyncHarvester {
        fn validate_source(&self, source: &str) -> Result<(), HarvestError> {
            let url = Url::parse(source).map_err(|_| HarvestError::InvalidUrl)?;
            if url.scheme() != "https" || url.username() != "" || url.password().is_some() {
                return Err(HarvestError::InvalidUrl);
            }
            let host = url.host_str().ok_or(HarvestError::InvalidUrl)?.trim_end_matches('.').to_ascii_lowercase();
            if !self.config.allowed_hosts.contains(&host) {
                return Err(HarvestError::HostNotAllowed);
            }
            Ok(())
        }
    }
}
