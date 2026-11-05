use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::{stream, StreamExt};
use reqwest::{Client, StatusCode};
use url::Url;

/// A downloaded document. Content remains bytes so non-UTF-8 sources are not
/// silently corrupted; decoding and content-type policy belong to a later stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDocument {
    pub source_url: String,
    pub content: Vec<u8>,
    pub fetched_at_timestamp: u64,
}

/// Explicit resource limits and the exact HTTPS hosts this collector may fetch.
/// Host matching is exact; subdomains must be listed separately.
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
            Self::InvalidConfiguration(message) => write!(f, "invalid harvester configuration: {message}"),
            Self::InvalidUrl => write!(f, "invalid or unsupported URL (HTTPS required)"),
            Self::HostNotAllowed => write!(f, "URL host is not in the configured allowlist"),
            Self::RequestFailed(message) => write!(f, "HTTP request failed: {message}"),
            Self::HttpStatus(code) => write!(f, "source returned HTTP status {code}"),
            Self::ResponseTooLarge => write!(f, "response exceeds configured byte limit"),
            Self::ClockError => write!(f, "system clock is before UNIX epoch"),
            Self::TooManySources { supplied, limit } => {
                write!(f, "batch has {supplied} sources; configured limit is {limit}")
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
            return Err(HarvestError::InvalidConfiguration("max_concurrency must be greater than zero"));
        }
        if config.max_sources_per_batch == 0 {
            return Err(HarvestError::InvalidConfiguration("max_sources_per_batch must be greater than zero"));
        }
        if config.max_response_bytes == 0 {
            return Err(HarvestError::InvalidConfiguration("max_response_bytes must be greater than zero"));
        }
        if config.request_timeout.is_zero() {
            return Err(HarvestError::InvalidConfiguration("request_timeout must be greater than zero"));
        }
        if config.allowed_hosts.is_empty() {
            return Err(HarvestError::InvalidConfiguration("allowed_hosts must not be empty"));
        }

        config.allowed_hosts = config
            .allowed_hosts
            .into_iter()
            .map(|host| host.trim().trim_end_matches('.').to_ascii_lowercase())
            .filter(|host| !host.is_empty())
            .collect();
        if config.allowed_hosts.is_empty() {
            return Err(HarvestError::InvalidConfiguration("allowed_hosts must contain a hostname"));
        }

        // Do not follow redirects: the redirect target would otherwise need its
        // own allowlist validation before a request is made.
        let client = Client::builder()
            .timeout(config.request_timeout)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("software-do-agi-ingestion/0.1 (+operator-configured sources)")
            .build()
            .map_err(|error| HarvestError::RequestFailed(error.to_string()))?;

        Ok(Self { client, config })
    }

    /// Fetch only the URLs explicitly supplied by the caller. This is not a
    /// crawler: it does not discover links or expand the source list.
    /// Concurrent requests and batch size are bounded by configuration.
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

        // Preserve caller order even though network operations are concurrent.
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
        AsyncHarvester::new(config).expect("valid test config")
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
        let mut config = HarvesterConfig {
            allowed_hosts: ["example.org".to_owned()].into_iter().collect(),
            max_concurrency: 0,
            max_sources_per_batch: 4,
            max_response_bytes: 1024,
            request_timeout: Duration::from_secs(2),
        };
        assert!(matches!(AsyncHarvester::new(config.clone()), Err(HarvestError::InvalidConfiguration(_))));
        config.max_concurrency = 1;
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
