//! Details about a paper
//!
//! `Get /paper/{paperId}`
//!
//! `/paper/{paperId}?fields={fields}`
//!
//! ## Limitations
//! - Can only return up to 10 MB of data at a time.

use crate::{
    error::{Error, Result},
    ss::{
        client::{Method, Query, SemanticScholar, build_request},
        graph::{BASE_URL, NestedPaper, PaperField, PaperId, merge_paper_fields},
    },
};
use reqwest::StatusCode;

/// Query parameters for the paper search
#[derive(Debug, Clone)]
pub struct PaperIdSearchParam {
    /// A plain-text search query string.
    ///
    /// - No special query syntax is supported.
    /// - Hyphenated query terms yield no matches (replace it with space to find matches)
    id: PaperId,
    /// A comma-separated list of the fields to be returned.
    fields: Option<Vec<PaperField>>,
}

impl PaperIdSearchParam {
    pub fn new(id: &PaperId) -> Self {
        Self {
            id: id.to_owned(),
            fields: None,
        }
    }

    pub fn add_field(&mut self, field: PaperField) -> &mut Self {
        if let Some(ref mut fields) = self.fields {
            fields.push(field);
        } else {
            self.fields = Some(vec![field]);
        }
        self
    }

    pub(crate) fn query_string(&self) -> String {
        let mut query_string = self.id.to_string();
        if let Some(ref fields) = self.fields
            && !fields.is_empty()
        {
            let fields_string = merge_paper_fields(fields);
            query_string.push_str(&format!("?fields={}", fields_string));
        }

        query_string
    }
}

impl Query for PaperIdSearchParam {
    type Response = Option<NestedPaper>;

    async fn query(&self, client: &SemanticScholar) -> Result<Self::Response> {
        let url = format!("{}/paper/{}", BASE_URL, self.query_string());
        let req_builder = build_request(client, Method::Get, &url);

        let resp = req_builder.send().await?;
        match resp.status() {
            StatusCode::OK => Ok(Some(resp.json().await?)),
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(Error::RequestFailed(resp.text().await?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query() {
        let mut param =
            PaperIdSearchParam::new(&PaperId::id("649def34f8be52c8b66281af98ae884c09aef38b"));
        param.add_field(PaperField::Title);
        let client = SemanticScholar::default();
        let resp = param.query(&client).await.unwrap();
        assert!(resp.is_some());
        let paper = resp.unwrap();
        println!("{:#?}", paper);
    }
}
