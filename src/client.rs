//! Connected Papers Client

use crate::{
    Error,
    error::Result,
    utils::{APIKey, Method, build_request},
};
use reqwest::{Client, StatusCode};
use std::time::Duration;

static APP_USER_AGENT: &str =
    concat!("RS", env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

const BASE_URL: &str = "https://rest.prod.connectedpapers.com/papers-api";

#[derive(Debug, Clone)]
pub struct ConnectedPapers {
    api_key: Option<String>,
    client: Client,
}

impl Default for ConnectedPapers {
    fn default() -> Self {
        Self {
            api_key: None,
            client: Client::builder()
                .timeout(Duration::from_secs(90))
                .user_agent(APP_USER_AGENT)
                .build()
                .unwrap(),
        }
    }
}

impl ConnectedPapers {
    /// Create a new client with the given API key
    pub fn with_api_key(api_key: &str) -> Self {
        Self {
            api_key: Some(api_key.to_owned()),
            ..Self::default()
        }
    }

    /// Create a new client from the environment variable `CONNECTED_PAPERS_API_KEY`
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("CONNECTED_PAPERS_API_KEY")?;
        Ok(Self::with_api_key(&api_key))
    }

    pub async fn get_graph(&self, _id: &str) -> Result<()> {
        todo!()
    }

    pub(crate) fn api_key(&self) -> Option<APIKey> {
        self.api_key.as_ref().map(|key| APIKey {
            header: "X-Api-Key".to_owned(),
            value: key.to_owned(),
        })
    }

    pub async fn get_remaining_usages(&self) -> Result<u64> {
        let url = format!("{}/remaining-usages", BASE_URL);
        let req_builder = build_request(&self.client, Method::Get, &url, self.api_key());
        let resp = req_builder.send().await?;
        match resp.status() {
            StatusCode::OK => {
                let body = resp.json::<serde_json::Value>().await?;
                let remaining_usages = body["remaining"].as_u64().unwrap_or(0);
                Ok(remaining_usages)
            }
            _ => Err(Error::RequestFailed(resp.text().await?)),
        }
    }

    pub async fn get_free_access_papers(&self) -> Result<Vec<String>> {
        let url = format!("{}/free-access-papers", BASE_URL);
        let req_builder = build_request(&self.client, Method::Get, &url, self.api_key());
        let resp = req_builder.send().await?;
        match resp.status() {
            StatusCode::OK => {
                let body = resp.json::<serde_json::Value>().await?;
                let free_access_papers = body["papers"]
                    .as_array()
                    .map(|values| {
                        values
                            .iter()
                            .map(|value| value.as_str().unwrap_or_default().to_owned())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                Ok(free_access_papers)
            }
            _ => Err(Error::RequestFailed(resp.text().await?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_remaining_usages() {
        let client = ConnectedPapers::with_api_key("TEST_TOKEN");
        let remaining_usages = client.get_remaining_usages().await.unwrap();
        println!("Remaining usages: {}", remaining_usages);
    }

    #[tokio::test]
    async fn test_get_free_access_papers() {
        let client = ConnectedPapers::with_api_key("TEST_TOKEN");
        let free_access_papers = client.get_free_access_papers().await.unwrap();
        println!("Free access papers: {:?}", free_access_papers);
    }
}
