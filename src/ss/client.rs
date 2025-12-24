use std::time::Duration;

use crate::error::Result;
use reqwest::{Client, RequestBuilder};
use serde::Deserialize;

#[derive(Debug)]
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
                .build()
                .unwrap(),
        }
    }
}

impl SemanticScholar {
    pub fn with_api_key(api_key: &str) -> Self {
        Self {
            api_key: Some(api_key.to_owned()),
            ..Self::default()
        }
    }

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

    pub async fn query<Q: Query>(&self, query: &Q) -> Result<Q::Response> {
        query.query(self).await
    }
}

pub trait Query {
    type Response;

    fn query(
        &self,
        client: &SemanticScholar,
    ) -> impl std::future::Future<Output = Result<Self::Response>> + Send;
}

#[derive(Debug, Clone, PartialEq, Deserialize, thiserror::Error)]
#[error("{error}")]
pub struct RequestFailedError {
    pub error: String,
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
