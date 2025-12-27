//! Paper title search
//!
//! `Get /paper/search/match`
//!
//! `/paper/search/match?query={query}`

use crate::{
    error::{Error, Result},
    ss::{
        client::{Query, SemanticScholar},
        graph::{
            _Date, Author, BASE_URL, CitationStyles, Date, Embedding, ExternalIds, FieldOfStudy,
            Journal, NestedPaper, OpenAccessPdf, Paper, PaperField, PublicationType,
            PublicationVenue, S2FieldsOfStudy, merge_fields_of_study, merge_paper_fields,
            merge_publication_types,
        },
    },
    utils::{Method, build_request},
};
use chrono::NaiveDate;
use reqwest::StatusCode;
use serde::Deserialize;

/// Query parameters for the paper title search
#[derive(Debug, Clone)]
pub struct PaperTitleSearchParam {
    query: String,
    fields: Option<Vec<PaperField>>,
    publication_types: Option<Vec<PublicationType>>,
    open_access_pdf: Option<bool>,
    min_citation_count: Option<u32>,
    publication_date: Option<(Option<Date>, Option<Date>)>,
    year: Option<(Option<u32>, Option<u32>)>,
    fields_of_study: Option<Vec<FieldOfStudy>>,
    venue: Option<Vec<String>>,
}

impl PaperTitleSearchParam {
    pub(crate) fn query_string(&self) -> String {
        let mut query_string = format!("query={}", &self.query);
        if let Some(ref fields) = self.fields
            && !fields.is_empty()
        {
            let fields_string = merge_paper_fields(fields);
            query_string.push_str(&format!("&fields={}", fields_string));
        }

        if let Some(ref publication_types) = self.publication_types
            && !publication_types.is_empty()
        {
            let publication_types_string = merge_publication_types(publication_types);
            query_string.push_str(&format!("&publicationTypes={}", publication_types_string));
        }

        if let Some(open_access) = self.open_access_pdf
            && open_access
        {
            query_string.push_str("&openAccessPdf");
        }

        if let Some(min_citation_count) = self.min_citation_count {
            query_string.push_str(&format!("&minCitationCount={}", min_citation_count));
        }

        if let Some((ref start, ref end)) = self.publication_date {
            match (start.as_ref(), end.as_ref()) {
                (Some(start), Some(end)) => {
                    query_string.push_str(&format!("&publicationDate={}:{}", start, end));
                }
                (Some(start), None) => {
                    query_string.push_str(&format!("&publicationDate={}:", start));
                }
                (None, Some(end)) => {
                    query_string.push_str(&format!("&publicationDate=:{}", end));
                }
                (None, None) => (),
            }
        }

        if let Some(year) = self.year {
            match (year.0, year.1) {
                (Some(start), Some(end)) => {
                    if start == end {
                        query_string.push_str(&format!("&year={}", start));
                    } else {
                        query_string.push_str(&format!("&year={}-{}", start, end));
                    }
                }
                (Some(start), None) => query_string.push_str(&format!("&year={}-", start)),
                (None, Some(end)) => query_string.push_str(&format!("&year=-{}", end)),
                _ => (),
            }
        }

        if let Some(ref fields_of_study) = self.fields_of_study
            && !fields_of_study.is_empty()
        {
            let fields_of_study_string = merge_fields_of_study(fields_of_study);
            query_string.push_str(&format!("&fieldsOfStudy={}", fields_of_study_string));
        }

        if let Some(ref venue) = self.venue
            && !venue.is_empty()
        {
            let venue_string = venue.join(",");
            query_string.push_str(&format!("&venue={}", venue_string));
        }

        query_string
    }
}

impl Query for PaperTitleSearchParam {
    type Response = Option<MatchedPaper>;

    async fn query(&self, client: &SemanticScholar) -> Result<Self::Response> {
        let url = format!("{}/paper/search/match?{}", BASE_URL, self.query_string());
        let req_builder = build_request(client.client(), Method::Get, &url, client.api_key());

        let resp = req_builder.send().await?;
        match resp.status() {
            StatusCode::OK => {
                let result = resp.json::<PaperTitleSearchResponse>().await?;
                if let Some(paper) = result.data.first() {
                    Ok(Some(paper.clone().into()))
                } else {
                    Ok(None)
                }
            }
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(Error::RequestFailed(resp.text().await?)),
        }
    }
}

/// Builder for the paper search parameters
#[derive(Debug, Clone, Default)]
pub struct PaperTitleSearchParamBuilder {
    query: String,
    fields: Option<Vec<PaperField>>,
    publication_types: Option<Vec<PublicationType>>,
    open_access_pdf: Option<bool>,
    min_citation_count: Option<u32>,
    publication_date: Option<(Option<_Date>, Option<_Date>)>,
    year: Option<(Option<u32>, Option<u32>)>,
    fields_of_study: Option<Vec<FieldOfStudy>>,
    venue: Option<Vec<String>>,
}

impl PaperTitleSearchParamBuilder {
    /// Create a new builder with the given query
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_owned(),
            ..Default::default()
        }
    }

    /// Add a field to the paper search parameters
    pub fn field(&mut self, field: PaperField) -> &mut Self {
        if let Some(ref mut fields) = self.fields {
            fields.push(field);
        } else {
            self.fields = Some(vec![field]);
        }
        self
    }

    /// Add a publication type to the paper search parameters
    pub fn publication_type(&mut self, type_: PublicationType) -> &mut Self {
        if let Some(ref mut publication_types) = self.publication_types {
            publication_types.push(type_);
        } else {
            self.publication_types = Some(vec![type_]);
        }
        self
    }

    /// Restricts results to only include papers with a public PDF
    pub fn open_access_pdf(&mut self) -> &mut Self {
        self.open_access_pdf = Some(true);
        self
    }

    /// Restricts results to only include papers with the minimum number of citations
    pub fn min_citation_count(&mut self, min_citation_count: u32) -> &mut Self {
        self.min_citation_count = Some(min_citation_count);
        self
    }

    /// Restricts results to the given range of publication dates.
    pub fn from_date(&mut self, year: i32, month: u32, day: u32) -> &mut Self {
        if let Some((ref mut start, _)) = self.publication_date {
            *start = Some(_Date(year, month, Some(day)));
        } else {
            self.publication_date = Some((Some(_Date(year, month, Some(day))), None));
        }
        self
    }

    pub fn to_date(&mut self, year: i32, month: u32, day: u32) -> &mut Self {
        if let Some((_, ref mut end)) = self.publication_date {
            *end = Some(_Date(year, month, Some(day)));
        } else {
            self.publication_date = Some((None, Some(_Date(year, month, Some(day)))));
        }
        self
    }

    pub fn from_month(&mut self, year: i32, month: u32) -> &mut Self {
        if let Some((ref mut start, _)) = self.publication_date {
            *start = Some(_Date(year, month, None));
        } else {
            self.publication_date = Some((Some(_Date(year, month, None)), None));
        }
        self
    }

    pub fn to_month(&mut self, year: i32, month: u32) -> &mut Self {
        if let Some((_, ref mut end)) = self.publication_date {
            *end = Some(_Date(year, month, None));
        } else {
            self.publication_date = Some((None, Some(_Date(year, month, None))));
        }
        self
    }

    /// Restricts results to the given publication year range (inclusive).
    pub fn from_year(&mut self, year: u32) -> &mut Self {
        if let Some((ref mut start, _)) = self.year {
            *start = Some(year);
        } else {
            self.year = Some((Some(year), None));
        }
        self
    }

    pub fn to_year(&mut self, year: u32) -> &mut Self {
        if let Some((_, ref mut end)) = self.year {
            *end = Some(year);
        } else {
            self.year = Some((None, Some(year)));
        }
        self
    }

    pub fn at_year(&mut self, year: u32) -> &mut Self {
        self.year = Some((Some(year), Some(year)));
        self
    }

    /// Add a field of study to the paper search parameters
    pub fn field_of_study(&mut self, field_of_study: FieldOfStudy) -> &mut Self {
        if let Some(ref mut fields_of_study) = self.fields_of_study {
            fields_of_study.push(field_of_study);
        } else {
            self.fields_of_study = Some(vec![field_of_study]);
        }
        self
    }

    /// Add a venue to the paper search parameters
    pub fn venue(&mut self, venue: &str) -> &mut Self {
        if let Some(ref mut venues) = self.venue {
            venues.push(venue.to_owned());
        } else {
            self.venue = Some(vec![venue.to_owned()]);
        }
        self
    }

    /// Build the paper search parameters
    pub fn build(&self) -> Result<PaperTitleSearchParam> {
        if let Some(year) = self.year
            && let Some(start) = year.0
            && let Some(end) = year.1
            && start > end
        {
            return Err(Error::InvalidParameter(
                "start year must be less than or equal to end year".to_string(),
            ));
        }

        let publication_date = match self.publication_date {
            Some((ref start, ref end)) => match (start.as_ref(), end.as_ref()) {
                (Some(start), Some(end)) => {
                    Some((Some(Date::try_from(start)?), Some(Date::try_from(end)?)))
                }
                (Some(start), None) => Some((Some(Date::try_from(start)?), None)),
                (None, Some(end)) => Some((None, Some(Date::try_from(end)?))),
                (None, None) => None,
            },
            None => None,
        };

        Ok(PaperTitleSearchParam {
            query: self.query.clone(),
            fields: self.fields.clone(),
            publication_types: self.publication_types.clone(),
            open_access_pdf: self.open_access_pdf,
            min_citation_count: self.min_citation_count,
            publication_date,
            year: self.year,
            fields_of_study: self.fields_of_study.clone(),
            venue: self.venue.clone(),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct PaperTitleSearchResponse {
    data: Vec<InnerPaperTitleSearchResponse>,
}

#[derive(Debug, Clone)]
pub struct MatchedPaper {
    pub score: f64,
    pub paper: NestedPaper,
}

/// Response for the paper search
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InnerPaperTitleSearchResponse {
    match_score: f64,
    paper_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    corpus_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_ids: Option<ExternalIds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "abstract")]
    abstract_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    venue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    publication_venue: Option<PublicationVenue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    citation_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    influential_citation_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_open_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    open_access_pdf: Option<OpenAccessPdf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields_of_study: Option<Vec<FieldOfStudy>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    s2_fields_of_study: Option<Vec<S2FieldsOfStudy>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    publication_types: Option<Vec<PublicationType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    publication_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    journal: Option<Journal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    citation_styles: Option<CitationStyles>,
    #[serde(skip_serializing_if = "Option::is_none")]
    authors: Option<Vec<Author>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    citations: Option<Vec<Paper>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    references: Option<Vec<Paper>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embedding: Option<Embedding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_availability: Option<String>,
}

impl From<InnerPaperTitleSearchResponse> for MatchedPaper {
    fn from(response: InnerPaperTitleSearchResponse) -> Self {
        MatchedPaper {
            score: response.match_score,
            paper: NestedPaper {
                paper_id: response.paper_id,
                corpus_id: response.corpus_id,
                external_ids: response.external_ids,
                url: response.url,
                title: response.title,
                abstract_: response.abstract_,
                venue: response.venue,
                publication_venue: response.publication_venue,
                year: response.year,
                reference_count: response.reference_count,
                citation_count: response.citation_count,
                influential_citation_count: response.influential_citation_count,
                is_open_access: response.is_open_access,
                open_access_pdf: response.open_access_pdf,
                fields_of_study: response.fields_of_study,
                s2_fields_of_study: response.s2_fields_of_study,
                publication_types: response.publication_types,
                publication_date: response.publication_date,
                journal: response.journal,
                citation_styles: response.citation_styles,
                authors: response.authors,
                text_availability: response.text_availability,
                citations: response.citations,
                references: response.references,
                embedding: response.embedding,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_query() {
        let client = SemanticScholar::default();
        let param = PaperTitleSearchParamBuilder::new(
            "Construction of the Literature Graph in Semantic Scholar",
        )
        .build()
        .unwrap();
        let result = client.query(&param).await.unwrap();
        assert!(result.is_some());
        println!("{:#?}", result);
    }
}
