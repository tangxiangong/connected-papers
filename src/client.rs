//! Connected Papers Client

use crate::{
    Error, ExternalIds, FieldOfStudy, PublicationType,
    error::Result,
    utils::{APIKey, Method, build_request},
};
#[cfg(feature = "stream")]
use async_stream::stream;
use chrono::{NaiveDate, NaiveDateTime};
#[cfg(feature = "stream")]
use futures::Stream;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
#[cfg(feature = "stream")]
use std::pin::Pin;
use std::{collections::HashMap, time::Duration};

static APP_USER_AGENT: &str =
    concat!("RS", env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

const BASE_URL: &str = "https://rest.prod.connectedpapers.com/papers-api";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GraphResponseType {
    BadId,
    Error,
    NotInDb,
    OldGraph,
    FreshGraph,
    InProgress,
    Queued,
    BadToken,
    BadRequest,
    OutOfRequests,
    Overloaded,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GraphResponse {
    pub status: GraphResponseType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_json: Option<Graph>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_requests: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Graph {
    pub nodes: HashMap<String, Paper>,
    pub edges: Vec<Edge>,
    #[serde(rename = "common_citations")]
    pub citations: Vec<Citation>,
    #[serde(rename = "common_references")]
    pub references: Vec<Reference>,
    #[serde(rename = "common_authors")]
    pub authors: Vec<AuthorDetail>,
    pub parameters: Parameter,
    pub path_lengths: HashMap<String, f64>,
    pub start_id: String,
    pub current_corpus_date: NaiveDate,
    pub creation_time: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Parameter {
    pub paper_id: String,
    pub total_nodes: u32,
    pub num_commons: u32,
    pub max_load: u32,
    pub num_neighbors: u32,
    pub spring_iterations: u32,
    pub params_version: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Edge(pub String, pub String, pub f64);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<Option<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthorDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention_indexes: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paper {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "corpusid")]
    pub corpus_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<Author>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_of_study: Option<Vec<FieldOfStudy>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_pages: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mag_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arxiv_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ids: Option<ExternalIds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_open_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tldr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_types: Option<Vec<PublicationType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_date: Option<NaiveDate>,
    #[serde(rename = "paperId")]
    pub paper_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "citations_length")]
    pub citations_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "references_length")]
    pub references_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "abstract")]
    pub abstract_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ref_with_start")]
    pub ref_with_start: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "cit_with_start")]
    pub cit_with_start: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "path_length")]
    pub path_length: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos: Option<[f64; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "number_of_authors")]
    pub number_of_authors: Option<u8>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "corpusid")]
    pub corpus_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<Author>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_of_study: Option<Vec<FieldOfStudy>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_pages: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mag_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arxiv_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ids: Option<ExternalIds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_open_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tldr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_types: Option<Vec<PublicationType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_date: Option<NaiveDate>,
    #[serde(rename = "paperId")]
    pub paper_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "edges_count")]
    pub edges_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "total_citations")]
    pub total_citations: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "citations_length")]
    pub citations_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "references_length")]
    pub references_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pi_name")]
    pub pi_name: Option<String>,
    #[serde(rename = "abstract")]
    pub abstract_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "local_references")]
    pub local_references: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "number_of_authors")]
    pub number_of_authors: Option<u8>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "corpusid")]
    pub corpus_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<Author>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_of_study: Option<Vec<FieldOfStudy>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_pages: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mag_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arxiv_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ids: Option<ExternalIds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_open_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tldr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_types: Option<Vec<PublicationType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_date: Option<NaiveDate>,
    #[serde(rename = "paperId")]
    pub paper_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "edges_count")]
    pub edges_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "total_citations")]
    pub total_citations: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "citations_length")]
    pub citations_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "references_length")]
    pub references_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pi_name")]
    pub pi_name: Option<String>,
    #[serde(rename = "abstract")]
    pub abstract_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "local_citations")]
    pub local_citations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "number_of_authors")]
    pub number_of_authors: Option<u8>,
}

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

    /// Get the graph for a given paper ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the paper to get the graph for
    /// * `fresh_only` - If `true`, force a fresh graph rebuild (ignore cached graphs)
    pub async fn get_graph(&self, id: &str, fresh_only: bool) -> Result<GraphResponse> {
        let url = if fresh_only {
            format!("{}/graph/1/{}", BASE_URL, id)
        } else {
            format!("{}/graph/0/{}", BASE_URL, id)
        };
        let req_builder = build_request(&self.client, Method::Get, &url, self.api_key());
        let resp = req_builder.send().await?;
        match resp.status() {
            StatusCode::OK => {
                let body = resp.json::<GraphResponse>().await?;
                Ok(body)
            }
            _ => Err(Error::RequestFailed(resp.text().await?)),
        }
    }

    #[cfg(feature = "stream")]
    #[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
    /// Get the graph as a stream, yielding status updates until completion
    ///
    /// This method continuously polls the API and yields `GraphResponse` updates
    /// as the graph is being built.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the paper to get the graph for
    /// * `fresh_only` - If `true`, force a fresh graph rebuild (ignore cached graphs)
    /// * `wait_until_complete` - If `true`, wait until a terminal status is reached
    ///   (FRESH_GRAPH, OLD_GRAPH, or error). If `false`, return immediately with current status.
    pub fn get_graph_stream(
        &self,
        id: &str,
        fresh_only: bool,
        wait_until_complete: bool,
    ) -> Pin<Box<dyn Stream<Item = Result<GraphResponse>> + Send + '_>> {
        let id = id.to_owned();
        Box::pin(stream! {
            let mut current_fresh_only = fresh_only;
            let mut newest_graph: Option<Graph> = None;

            loop {
                match self.get_graph(&id, current_fresh_only).await {
                    Ok(mut response) => {
                        if let Some(ref graph) = response.graph_json {
                            newest_graph = Some(graph.clone());
                        }

                        if response.status == GraphResponseType::OldGraph {
                            if wait_until_complete && !fresh_only {
                                current_fresh_only = true;
                                response.graph_json = newest_graph.clone();
                                yield Ok(response);
                                tokio::time::sleep(Duration::from_secs(1)).await;
                                continue;
                            } else if !fresh_only {
                                response.graph_json = newest_graph.clone();
                                yield Ok(response);
                                return;
                            }
                        }

                        if response.status == GraphResponseType::Overloaded {
                            for &delay in &[Duration::from_secs(5), Duration::from_secs(10), Duration::from_secs(20), Duration::from_secs(40)] {
                                tokio::time::sleep(delay).await;
                                match self.get_graph(&id, current_fresh_only).await {
                                    Ok(new_response) if new_response.status != GraphResponseType::Overloaded => {
                                        response = new_response;
                                        break;
                                    }
                                    Ok(new_response) => {
                                        response = new_response;
                                    }
                                    Err(e) => {
                                        yield Err(e);
                                        return;
                                    }
                                }
                            }
                        }

                        let status = response.status;
                        response.graph_json = newest_graph.clone();
                        yield Ok(response);

                        let is_terminal = matches!(
                            status,
                            GraphResponseType::BadId
                                | GraphResponseType::Error
                                | GraphResponseType::NotInDb
                                | GraphResponseType::FreshGraph
                                | GraphResponseType::BadToken
                                | GraphResponseType::BadRequest
                                | GraphResponseType::OutOfRequests
                        );

                        if !wait_until_complete || is_terminal {
                            return;
                        }

                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    Err(e) => {
                        yield Err(e);
                        return;
                    }
                }
            }
        })
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

    #[tokio::test]
    async fn test_get_graph() {
        let client = ConnectedPapers::with_api_key("TEST_TOKEN");
        let graph = client
            .get_graph("9397e7acd062245d37350f5c05faf56e9cfae0d6", false)
            .await
            .unwrap();
        println!("Graph: {:?}", graph);
    }
}
