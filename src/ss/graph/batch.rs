//! Get details for multiple papers at once.
//!
//! `POST /paper/batch`
//!
//! ## Limitations
//! - Can only process 500 paper ids at a time.
//! - Can only return up to 10 MB of data at a time.
//! - Can only return up to 9999 citations at a time.

use crate::{
    error::{Error, Result},
    ss::{
        client::{Method, Query, S2RequestFailedError, SemanticScholar, build_request},
        graph::{BASE_URL, NestedPaper, PaperField, PaperId, merge_paper_fields},
    },
};
use reqwest::StatusCode;
use serde::Serialize;

/// Parameters for the paper batch query
#[derive(Debug, Clone)]
pub struct PaperBatchParam {
    pub ids: Vec<PaperId>,
    pub fields: Option<Vec<PaperField>>,
}

/// Builder for the paper batch query parameters
#[derive(Debug, Clone, Default)]
pub struct PaperBatchParamBuilder {
    ids: Vec<PaperId>,
    fields: Option<Vec<PaperField>>,
}

impl PaperBatchParamBuilder {
    /// Add a paper id to the query
    pub fn id(&mut self, id: PaperId) -> &mut Self {
        self.ids.push(id);
        self
    }

    /// Add a paper field to the query
    pub fn field(&mut self, field: PaperField) -> &mut Self {
        if let Some(ref mut fields) = self.fields {
            fields.push(field);
        } else {
            self.fields = Some(vec![field]);
        }
        self
    }

    /// Build the paper batch query parameters
    pub fn build(&self) -> Result<PaperBatchParam> {
        if self.ids.is_empty() {
            Err(Error::InvalidParameter("ids is empty".to_string()))
        } else {
            Ok(PaperBatchParam {
                ids: self.ids.clone(),
                fields: self.fields.clone(),
            })
        }
    }
}

/// Inner struct for the paper batch query
#[derive(Debug, Clone, Serialize)]
struct PaperIds {
    ids: Vec<PaperId>,
}

impl Query for PaperBatchParam {
    type Response = Vec<NestedPaper>;

    async fn query(&self, client: &SemanticScholar) -> Result<Self::Response> {
        let paper_ids = PaperIds {
            ids: self.ids.clone(),
        };
        let url = if let Some(ref fields) = self.fields
            && !fields.is_empty()
        {
            format!(
                "{}/paper/batch?fields={}",
                BASE_URL,
                merge_paper_fields(fields)
            )
        } else {
            format!("{}/paper/batch", BASE_URL)
        };
        let req_builder = build_request(client, Method::Post, &url);

        let resp = req_builder.json(&paper_ids).send().await?;
        match resp.status() {
            StatusCode::OK => Ok(resp.json().await?),
            _ => Err(S2RequestFailedError {
                error: resp.text().await?,
            }
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_batch_param_builder() {
        let mut builder = PaperBatchParamBuilder::default();
        builder.id(PaperId::id("649def34f8be52c8b66281af98ae884c09aef38b"));
        builder.field(PaperField::IsOpenAccess);
        let param = builder.build().unwrap();
        assert_eq!(
            param.ids,
            vec![PaperId::id("649def34f8be52c8b66281af98ae884c09aef38b")]
        );
        assert_eq!(param.fields, Some(vec![PaperField::IsOpenAccess]));
    }

    #[tokio::test]
    async fn test_batch_query() {
        let ids = vec![PaperId::id("649def34f8be52c8b66281af98ae884c09aef38b")];
        let fields = vec![PaperField::IsOpenAccess];
        let param = PaperBatchParam {
            ids,
            fields: Some(fields),
        };

        let client = SemanticScholar::default();
        let res = client.query(&param).await.unwrap();
        println!("{:?}", res);
    }
}
