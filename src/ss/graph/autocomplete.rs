//! Suggest paper query completions
//!
//! To support interactive query-completion, return minimal information about papers
//! matching a partial query.
//!
//! `GET /paper/autocomplete`
//!

use crate::{
    error::Result,
    ss::{
        client::{Method, Query, S2RequestFailedError, SemanticScholar, build_request},
        graph::BASE_URL,
    },
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

/// Parameters for the autocomplete query
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PaperAutocompleteParam {
    /// Plain-text partial query string. Will be truncated to first 100 characters.
    pub query: String,
}

impl PaperAutocompleteParam {
    /// Create a new autocomplete query parameters
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
        }
    }
}

/// Response for autocomplete query
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PaperAutocompleteResponse {
    pub matches: Vec<AutocompletePaper>,
}

/// Inner struct for autocomplete query
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutocompletePaper {
    /// The paper's primary unique identifier.
    pub id: String,
    /// Title of the paper.
    pub title: String,
    /// Summary of the authors and year of publication.
    pub authors_year: String,
}

impl AutocompletePaper {
    /// Get the authors of the paper
    pub fn authors(&self) -> String {
        self.authors_year
            .split(",")
            .nth(0)
            .unwrap_or_default()
            .to_string()
    }

    /// Get the year of the paper
    pub fn year(&self) -> Option<u32> {
        self.authors_year
            .split(",")
            .nth(1)
            .and_then(|year| year.trim().parse().ok())
    }
}

impl Query for PaperAutocompleteParam {
    type Response = Vec<AutocompletePaper>;

    async fn query(&self, client: &SemanticScholar) -> Result<Self::Response> {
        let url = format!("{}/paper/autocomplete", BASE_URL);
        let req_builder = build_request(client, Method::Get, &url);
        let res = req_builder.query(self).send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json::<PaperAutocompleteResponse>().await?.matches),
            _ => Err(S2RequestFailedError {
                error: res.text().await?,
            }
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_autocomplete() {
        let client = SemanticScholar::default();
        let query = PaperAutocompleteParam {
            query: "semantic".to_string(),
        };
        let res = client.query(&query).await.unwrap();
        println!("{:?}", res);
    }
}
