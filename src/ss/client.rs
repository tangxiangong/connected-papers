//! Semantic Scholar Client

use crate::error::Result;
use reqwest::{Client, RequestBuilder};
use std::time::Duration;

static APP_USER_AGENT: &str =
    concat!("RS", env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// Client
#[derive(Debug, Clone)]
pub struct SemanticScholar {
    api_key: Option<String>,
    client: Client,
}

impl Default for SemanticScholar {
    fn default() -> Self {
        Self {
            api_key: None,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent(APP_USER_AGENT)
                .build()
                .unwrap(),
        }
    }
}

impl SemanticScholar {
    /// Create a new client with the given API key
    pub fn with_api_key(api_key: &str) -> Self {
        Self {
            api_key: Some(api_key.to_owned()),
            ..Self::default()
        }
    }

    /// Create a new client from the environment variable `SEMANTIC_SCHOLAR_API_KEY`
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("SEMANTIC_SCHOLAR_API_KEY")?;
        Ok(Self::with_api_key(&api_key))
    }

    pub(crate) fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    /// Query the Semantic Scholar API
    pub async fn query<Q: Query>(&self, query: &Q) -> Result<Q::Response> {
        query.query(self).await
    }
}

/// Query trait
pub trait Query {
    type Response;

    fn query(
        &self,
        client: &SemanticScholar,
    ) -> impl std::future::Future<Output = Result<Self::Response>> + Send;
}

pub(crate) fn build_request(client: &SemanticScholar, method: Method, url: &str) -> RequestBuilder {
    let mut req_builder = match method {
        Method::Get => client.client().get(url),
        Method::Post => client.client().post(url),
    };
    if let Some(api_key) = client.api_key() {
        req_builder = req_builder.header("x-api-key", api_key);
    }
    req_builder
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Method {
    Get,
    Post,
}
