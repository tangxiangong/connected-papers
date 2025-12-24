use crate::{
    error::Result,
    ss::client::{Method, Query, RequestFailedError, SemanticScholar, build_request},
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.semanticscholar.org/graph/v1";

/// Suggest paper query completions
///
/// To support interactive query-completion, return minimal information about papers
/// matching a partial query.
///
/// `GET /paper/autocomplete`
///
/// Example: `https://api.semanticscholar.org/graph/v1/paper/autocomplete?query=semantic`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoCompleteParam {
    /// Plain-text partial query string. Will be truncated to first 100 characters.
    pub query: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AutoCompleteResponse {
    pub matches: Vec<AutoCompletePaper>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoCompletePaper {
    /// The paper's primary unique identifier.
    pub id: String,
    /// Title of the paper.
    pub title: String,
    /// Summary of the authors and year of publication.
    pub authors_year: String,
}

impl AutoCompletePaper {
    pub fn authors(&self) -> String {
        self.authors_year
            .split(",")
            .nth(0)
            .unwrap_or_default()
            .to_string()
    }

    pub fn year(&self) -> Option<u32> {
        self.authors_year
            .split(",")
            .nth(1)
            .and_then(|year| year.trim().parse().ok())
    }
}

impl Query for AutoCompleteParam {
    type Response = Vec<AutoCompletePaper>;

    async fn query(&self, client: &SemanticScholar) -> Result<Self::Response> {
        let url = format!("{}/paper/autocomplete", BASE_URL);
        let req_builder = build_request(client, Method::Get, &url);
        let resp = req_builder.query(self).send().await?;
        let res = match resp.status() {
            StatusCode::OK => resp.json::<AutoCompleteResponse>().await?,
            _ => {
                return Err(RequestFailedError {
                    error: resp.text().await?,
                }
                .into());
            }
        };
        Ok(res.matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_autocomplete() {
        let client = SemanticScholar::default();
        let query = AutoCompleteParam {
            query: "semantic".to_string(),
        };
        let res = client.query(&query).await.unwrap();
        println!("{:?}", res);
    }
}
