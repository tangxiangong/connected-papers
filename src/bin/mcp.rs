use connected_papers::{ConnectedPapers, GraphResponse, GraphResponseType};
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{InitializeResult, ServerCapabilities},
    schemars, tool, tool_router,
    transport::stdio,
};
use serde_json::json;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
pub struct ConnectedPapersMCP {
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
    api_key: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetGraphRequest {
    #[schemars(description = "The (Semantic Scholar primary) ID of the paper to get the graph of")]
    pub id: String,
    #[schemars(description = "If true, force a fresh graph rebuild (ignore cached graphs)")]
    #[serde(default)]
    pub fresh_only: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetPaperInfoRequest {
    #[schemars(description = "The (Semantic Scholar primary) ID of the paper")]
    pub id: String,
    #[schemars(description = "If true, force a fresh graph rebuild (ignore cached graphs)")]
    #[serde(default)]
    pub fresh_only: bool,
}

#[tool_router]
impl ConnectedPapersMCP {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_owned(),
            tool_router: Self::tool_router(),
        }
    }

    fn format_graph_response(response: GraphResponse) -> String {
        let status_str = match response.status {
            GraphResponseType::BadId => "BAD_ID",
            GraphResponseType::Error => "ERROR",
            GraphResponseType::NotInDb => "NOT_IN_DB",
            GraphResponseType::OldGraph => "OLD_GRAPH",
            GraphResponseType::FreshGraph => "FRESH_GRAPH",
            GraphResponseType::InProgress => "IN_PROGRESS",
            GraphResponseType::Queued => "QUEUED",
            GraphResponseType::BadToken => "BAD_TOKEN",
            GraphResponseType::BadRequest => "BAD_REQUEST",
            GraphResponseType::OutOfRequests => "OUT_OF_REQUESTS",
            GraphResponseType::Overloaded => "OVERLOADED",
        };

        let mut result = json!({
            "status": status_str,
        });

        if let Some(progress) = response.progress {
            result["progress"] = json!(progress);
        }

        if let Some(remaining) = response.remaining_requests {
            result["remaining_requests"] = json!(remaining);
        }

        if let Some(graph) = response.graph_json {
            result["graph"] = json!({
                "start_id": graph.start_id,
                "nodes_count": graph.nodes.len(),
                "edges_count": graph.edges.len(),
                "citations_count": graph.citations.len(),
                "references_count": graph.references.len(),
                "authors_count": graph.authors.len(),
                "parameters": {
                    "paper_id": graph.parameters.paper_id,
                    "total_nodes": graph.parameters.total_nodes,
                    "num_commons": graph.parameters.num_commons,
                    "max_load": graph.parameters.max_load,
                    "num_neighbors": graph.parameters.num_neighbors,
                    "spring_iterations": graph.parameters.spring_iterations,
                },
                "current_corpus_date": graph.current_corpus_date.to_string(),
                "creation_time": graph.creation_time.to_string(),
            });

            // Include the start paper (main paper) details
            if let Some(start_paper) = graph.nodes.get(&graph.start_id) {
                result["start_paper"] = json!({
                    "id": start_paper.id,
                    "title": start_paper.title,
                    "authors": start_paper.authors.as_ref().map(|a| a.iter().map(|author| {
                        author.name.as_deref().unwrap_or("Unknown")
                    }).collect::<Vec<_>>()),
                    "year": start_paper.year,
                    "venue": start_paper.venue,
                    "journal_name": start_paper.journal_name,
                    "doi": start_paper.doi,
                    "arxiv_id": start_paper.arxiv_id,
                    "abstract": start_paper.abstract_,
                    "url": start_paper.url,
                    "is_open_access": start_paper.is_open_access,
                    "citations_length": start_paper.citations_length,
                    "references_length": start_paper.references_length,
                });
            }
        }

        serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{:?}", result))
    }

    #[tool(
        description = "Get the graph of a paper by its Semantic Scholar ID. Returns graph structure, status, and metadata."
    )]
    pub async fn get_graph(
        &self,
        Parameters(GetGraphRequest { id, fresh_only }): Parameters<GetGraphRequest>,
    ) -> String {
        let client = ConnectedPapers::with_api_key(&self.api_key);

        match client.get_graph(&id, fresh_only).await {
            Ok(response) => Self::format_graph_response(response),
            Err(e) => serde_json::to_string_pretty(&json!({
                "error": format!("Failed to get graph: {}", e),
            }))
            .unwrap_or_else(|_| format!("Error: Failed to get graph: {}", e)),
        }
    }

    #[tool(
        description = "Get detailed information about a paper from its graph, including title, authors, abstract, and metadata."
    )]
    pub async fn get_paper_info(
        &self,
        Parameters(GetPaperInfoRequest { id, fresh_only }): Parameters<GetPaperInfoRequest>,
    ) -> String {
        let client = ConnectedPapers::with_api_key(&self.api_key);

        match client.get_graph(&id, fresh_only).await {
            Ok(response) => {
                if let Some(graph) = response.graph_json {
                    if let Some(paper) = graph.nodes.get(&graph.start_id) {
                        let result = json!({
                            "id": paper.id,
                            "paper_id": paper.paper_id,
                            "title": paper.title,
                            "authors": paper.authors.as_ref().map(|a| a.iter().map(|author| {
                                json!({
                                    "name": author.name,
                                    "ids": author.ids,
                                })
                            }).collect::<Vec<_>>()),
                            "year": paper.year,
                            "venue": paper.venue,
                            "journal_name": paper.journal_name,
                            "journal_volume": paper.journal_volume,
                            "journal_pages": paper.journal_pages,
                            "doi": paper.doi,
                            "pmid": paper.pmid,
                            "arxiv_id": paper.arxiv_id,
                            "mag_id": paper.mag_id,
                            "abstract": paper.abstract_,
                            "tldr": paper.tldr,
                            "url": paper.url,
                            "pdf_urls": paper.pdf_urls,
                            "is_open_access": paper.is_open_access,
                            "fields_of_study": paper.fields_of_study.as_ref().map(|f| f.iter().map(|field| format!("{}", field)).collect::<Vec<_>>()),
                            "publication_types": paper.publication_types.as_ref().map(|p| p.iter().map(|pt| format!("{}", pt)).collect::<Vec<_>>()),
                            "publication_date": paper.publication_date.map(|d| d.to_string()),
                            "citations_length": paper.citations_length,
                            "references_length": paper.references_length,
                            "number_of_authors": paper.number_of_authors,
                            "corpus_id": paper.corpus_id,
                        });
                        serde_json::to_string_pretty(&result)
                            .unwrap_or_else(|_| format!("{:?}", result))
                    } else {
                        serde_json::to_string_pretty(&json!({
                            "error": format!("Paper {} not found in graph", id),
                        }))
                        .unwrap_or_else(|_| format!("Error: Paper {} not found in graph", id))
                    }
                } else {
                    serde_json::to_string_pretty(&json!({
                        "error": format!("Graph not available. Status: {:?}", response.status),
                        "status": format!("{:?}", response.status),
                        "progress": response.progress,
                    }))
                    .unwrap_or_else(|_| {
                        format!("Error: Graph not available. Status: {:?}", response.status)
                    })
                }
            }
            Err(e) => serde_json::to_string_pretty(&json!({
                "error": format!("Failed to get paper info: {}", e),
            }))
            .unwrap_or_else(|_| format!("Error: Failed to get paper info: {}", e)),
        }
    }

    #[tool(description = "Get the remaining number of API requests available for your API key.")]
    pub async fn get_remaining_usages(&self) -> String {
        let client = ConnectedPapers::with_api_key(&self.api_key);

        match client.get_remaining_usages().await {
            Ok(remaining) => serde_json::to_string_pretty(&json!({
                "remaining_usages": remaining,
            }))
            .unwrap_or_else(|_| format!("Remaining usages: {}", remaining)),
            Err(e) => serde_json::to_string_pretty(&json!({
                "error": format!("Failed to get remaining usages: {}", e),
            }))
            .unwrap_or_else(|_| format!("Error: Failed to get remaining usages: {}", e)),
        }
    }

    #[tool(description = "Get a list of paper IDs that have free access (no API key required).")]
    pub async fn get_free_access_papers(&self) -> String {
        let client = ConnectedPapers::with_api_key(&self.api_key);

        match client.get_free_access_papers().await {
            Ok(papers) => serde_json::to_string_pretty(&json!({
                "free_access_papers": papers,
                "count": papers.len(),
            }))
            .unwrap_or_else(|_| format!("Free access papers count: {}", papers.len())),
            Err(e) => serde_json::to_string_pretty(&json!({
                "error": format!("Failed to get free access papers: {}", e),
            }))
            .unwrap_or_else(|_| format!("Error: Failed to get free access papers: {}", e)),
        }
    }
}

impl ServerHandler for ConnectedPapersMCP {
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: rmcp::model::ProtocolVersion::default(),
            server_info: rmcp::model::Implementation {
                name: "connected-papers".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                title: Some("Connected Papers MCP Server".to_owned()),
                icons: None,
                website_url: Some("https://github.com/tangxiangong/connected-papers".to_owned()),
            },
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            instructions: Some("MCP Server for Connected Papers. Provides tools to query paper graphs, get paper information, check API usage, and access free papers.".to_owned()),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Connected Papers MCP server");

    let api_key =
        std::env::var("CONNECTED_PAPERS_API_KEY").unwrap_or_else(|_| "TEST_TOKEN".to_string());

    let service = ConnectedPapersMCP::new(&api_key)
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("Error: {:?}", e);
        })?;

    service.waiting().await?;

    Ok(())
}
