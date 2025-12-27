//! Paper Bulk Search
//!
//! `Get /paper/search/bulk`
//!
//! Behaves similarly to [`/paper/search`](super::relevance), but its intended for basic paper data without search relevance.
//!
//! - Text query is optional and supports boolean logic for document matching.
//! - Papers can be filtered using various criteria.
//! - Up to 1,000 papers will be returned in each call.
//! - If there are more matching papers, a continuation "token" will be present.
//! - The query can be repeated with the token param added to efficiently continue fetching matching papers.
//!
//! Returns a structure with the estimated total matches, batch of matching papers, and a continuation token if more results are available.
//!
//! ## Limitations
//!
//! - Nested paper data, such as citations, references, etc, is not available via this method.
//! - Up to 10,000,000 papers can be fetched via this method.

use crate::{
    error::{Error, Result},
    ss::{
        client::{Method, Query, S2RequestFailedError, SemanticScholar, build_request},
        graph::{
            _Date, BASE_URL, Date, FieldOfStudy, Paper, PaperField, PublicationType,
            merge_fields_of_study, merge_paper_fields, merge_publication_types,
        },
    },
};
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum QueryExpr {
    Term(String),                  // word
    Phrase(String),                // "word phrase"
    Prefix(String),                // word*
    FuzzyTerm(String, Option<u8>), // word~N
    ProximityPhrase(String, u8),   // "word phrase"~N
    And(Vec<QueryExpr>),           // +
    Or(Vec<QueryExpr>),            // |
    Not(Box<QueryExpr>),           // -
}

impl QueryExpr {
    /// Create a new term query expression
    pub fn term(term: &str) -> Self {
        QueryExpr::Term(term.to_string())
    }

    /// Create a new phrase query expression
    pub fn phrase(phrase: &str) -> Self {
        QueryExpr::Phrase(phrase.to_string())
    }

    /// Create a new prefix query expression
    pub fn prefix(prefix: &str) -> Self {
        QueryExpr::Prefix(prefix.to_string())
    }

    /// Create a new fuzzy term query expression
    pub fn fuzzy(term: &str, distance: Option<u8>) -> Self {
        QueryExpr::FuzzyTerm(term.to_string(), distance)
    }

    /// Create a new proximity phrase query expression
    pub fn proximity(phrase: &str, distance: u8) -> Self {
        QueryExpr::ProximityPhrase(phrase.to_string(), distance)
    }

    /// Create a new AND query expression
    pub fn and(self, other: QueryExpr) -> Self {
        match self {
            QueryExpr::And(mut nodes) => {
                nodes.push(other);
                QueryExpr::And(nodes)
            }
            _ => QueryExpr::And(vec![self, other]),
        }
    }

    /// Create a new OR query expression
    pub fn or(self, other: QueryExpr) -> Self {
        match self {
            QueryExpr::Or(mut nodes) => {
                nodes.push(other);
                QueryExpr::Or(nodes)
            }
            _ => QueryExpr::Or(vec![self, other]),
        }
    }

    /// Create a new NOT query expression
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        QueryExpr::Not(Box::new(self))
    }
}

impl std::fmt::Display for QueryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryExpr::Term(term) => write!(f, "{}", term),
            QueryExpr::Phrase(phrase) => write!(f, "\"{}\"", phrase),
            QueryExpr::Prefix(prefix) => write!(f, "{}*", prefix),
            QueryExpr::FuzzyTerm(term, n) => {
                if let Some(n) = n {
                    write!(f, "{}~{}", term, n)
                } else {
                    write!(f, "{}~", term)
                }
            }
            QueryExpr::ProximityPhrase(phrase, n) => write!(f, "\"{}\"~{}", phrase, n),
            QueryExpr::Not(node) => match **node {
                QueryExpr::And(_) | QueryExpr::Or(_) => write!(f, "-({})", node),
                _ => write!(f, "-{}", node),
            },
            QueryExpr::And(list) => {
                let content = list
                    .iter()
                    .map(|q| match q {
                        QueryExpr::Or(_) => format!("({})", q),
                        _ => format!("{}", q),
                    })
                    .collect::<Vec<_>>()
                    .join(" + ");
                write!(f, "{}", content)
            }

            QueryExpr::Or(list) => {
                let content = list
                    .iter()
                    .map(|q| format!("{}", q))
                    .collect::<Vec<_>>()
                    .join(" | ");
                write!(f, "{}", content)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    PaperId(SortOrder),
    PublicationDate(SortOrder),
    CitationCount(SortOrder),
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::Ascending => write!(f, "asc"),
            SortOrder::Descending => write!(f, "desc"),
        }
    }
}

impl std::fmt::Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortBy::PaperId(order) => write!(f, "paperId:{}", order),
            SortBy::PublicationDate(order) => write!(f, "publicationDate:{}", order),
            SortBy::CitationCount(order) => write!(f, "citationCount:{}", order),
        }
    }
}

/// Query parameters for the paper search
#[derive(Debug, Clone)]
pub struct PaperBulkSearchParam {
    /// Text query that will be matched against the paper's title and abstract.
    /// All terms are stemmed in English. By default all terms in the query must be present in the paper.
    ///
    /// The match query supports the following syntax:
    ///
    /// - `+` for AND operation
    /// - `|` for OR operation
    /// - `-` negates a term
    /// - `"` collects terms into a phrase
    /// - `*` can be used to match a prefix
    /// - `(` and `)` for precedence
    /// - `~N` after a word matches within the edit distance of N (Defaults to 2 if N is omitted)
    /// - `~N` after a phrase matches with the phrase terms separated up to N terms apart (Defaults to 2 if N is omitted)
    ///
    /// ## Examples:
    ///
    /// - `fish ladder` matches papers that contain "fish" and "ladder"
    /// - `fish -ladder` matches papers that contain "fish" but not "ladder"
    /// - `fish | ladder` matches papers that contain "fish" or "ladder"
    /// - `"fish ladder"` matches papers that contain the phrase "fish ladder"
    /// - `(fish ladder) | outflow` matches papers that contain "fish" and "ladder" OR "outflow"
    /// - `fish~` matches papers that contain "fish", "fist", "fihs", etc.
    /// - `"fish ladder"~3` matches papers that contain the phrase "fish ladder" or "fish is on a ladder"
    query: String,
    /// Used for pagination. This string token is provided when the original query returns, and is used to fetch the next batch of papers. Each call will return a new token.
    token: Option<String>,
    fields: Option<Vec<PaperField>>,
    sort: Option<SortBy>,
    publication_types: Option<Vec<PublicationType>>,
    open_access_pdf: Option<bool>,
    min_citation_count: Option<u32>,
    publication_date: Option<(Option<Date>, Option<Date>)>,
    year: Option<(Option<u32>, Option<u32>)>,
    fields_of_study: Option<Vec<FieldOfStudy>>,
    venue: Option<Vec<String>>,
}

impl PaperBulkSearchParam {
    pub(crate) fn query_string(&self) -> String {
        let mut query_string = format!("query={}", &self.query);

        if let Some(ref token) = self.token {
            query_string.push_str(&format!("&token={}", token));
        }

        if let Some(sort_by) = self.sort {
            query_string.push_str(&format!("&sort={}", sort_by));
        }

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

impl Query for PaperBulkSearchParam {
    type Response = PaperBulkSearchResponse;

    async fn query(&self, client: &SemanticScholar) -> Result<Self::Response> {
        let url = format!("{}/paper/search/bulk?{}", BASE_URL, self.query_string());
        let req_builder = build_request(client, Method::Get, &url);

        let resp = req_builder.send().await?;
        match resp.status() {
            StatusCode::OK => Ok(resp.json().await?),
            _ => Err(S2RequestFailedError {
                error: resp.text().await?,
            }
            .into()),
        }
    }
}

/// Builder for the paper search parameters
#[derive(Debug, Clone, Default)]
pub struct PaperBulkSearchParamBuilder {
    query: Option<QueryExpr>,
    token: Option<String>,
    sort: Option<SortBy>,
    fields: Option<Vec<PaperField>>,
    publication_types: Option<Vec<PublicationType>>,
    open_access_pdf: Option<bool>,
    min_citation_count: Option<u32>,
    publication_date: Option<(Option<_Date>, Option<_Date>)>,
    year: Option<(Option<u32>, Option<u32>)>,
    fields_of_study: Option<Vec<FieldOfStudy>>,
    venue: Option<Vec<String>>,
}

impl PaperBulkSearchParamBuilder {
    pub fn query(&mut self, query: &QueryExpr) -> &mut Self {
        self.query = Some(query.clone());
        self
    }

    pub fn token(&mut self, token: &str) -> &mut Self {
        self.token = Some(token.to_owned());
        self
    }

    pub fn sort_by(&mut self, sort_by: SortBy) -> &mut Self {
        self.sort = Some(sort_by);
        self
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
    pub fn build(&self) -> Result<PaperBulkSearchParam> {
        if self.query.is_none() {
            return Err(Error::InvalidParameter("query must be set".to_owned()));
        }

        if let Some(ref fields) = self.fields {
            let unsupported_fields = vec![
                PaperField::Citations,
                PaperField::References,
                PaperField::Embedding,
                PaperField::Tldr,
            ];
            for field in unsupported_fields {
                if fields.contains(&field) {
                    return Err(Error::InvalidParameter(format!(
                        "{} is not supported",
                        field
                    )));
                }
            }
        }

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

        Ok(PaperBulkSearchParam {
            query: self.query.clone().unwrap().to_string(),
            token: self.token.clone(),
            sort: self.sort,
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

/// Response for the paper search
#[derive(Debug, Clone, Deserialize)]
pub struct PaperBulkSearchResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<Paper>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_bulk_search_param_builder() {
        let mut builder = PaperBulkSearchParamBuilder::default();
        builder
            .query(&QueryExpr::term("test"))
            .field(PaperField::Title)
            .publication_type(PublicationType::JournalArticle)
            .open_access_pdf()
            .min_citation_count(10)
            .from_date(2020, 1, 1)
            .field_of_study(FieldOfStudy::ComputerScience);
        let param = builder.build().unwrap();
        assert_eq!(param.query, "test");
        assert_eq!(param.fields, Some(vec![PaperField::Title]));
        assert_eq!(
            param.publication_types,
            Some(vec![PublicationType::JournalArticle])
        );
        assert_eq!(param.open_access_pdf, Some(true));
        assert_eq!(param.min_citation_count, Some(10));
        assert_eq!(
            param.fields_of_study,
            Some(vec![FieldOfStudy::ComputerScience])
        );
    }
}
