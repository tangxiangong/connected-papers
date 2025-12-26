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
        client::{Method, Query, RequestFailedError, SemanticScholar, build_request},
        graph::{
            _Date, BASE_URL, Date, FieldOfStudy, Paper, PaperField, PublicationType,
            merge_fields_of_study, merge_paper_fields, merge_publication_types,
        },
    },
};
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum QueryNode {
    Term(String),   // word
    Phrase(String), // "word phrase"
    Prefix(String), // word*

    FuzzyTerm(String, Option<u8>), // word~N
    ProximityPhrase(String, u8),   // "word phrase"~N

    And(Vec<QueryNode>), // +
    Or(Vec<QueryNode>),  // |
    Not(Box<QueryNode>), // -
}

impl QueryNode {
    pub fn term(term: &str) -> Self {
        QueryNode::Term(term.to_string())
    }

    pub fn phrase(phrase: &str) -> Self {
        QueryNode::Phrase(phrase.to_string())
    }

    pub fn prefix(prefix: &str) -> Self {
        QueryNode::Prefix(prefix.to_string())
    }

    pub fn fuzzy(term: &str, distance: Option<u8>) -> Self {
        QueryNode::FuzzyTerm(term.to_string(), distance)
    }

    pub fn proximity(phrase: &str, distance: u8) -> Self {
        QueryNode::ProximityPhrase(phrase.to_string(), distance)
    }

    pub fn and(self, other: QueryNode) -> Self {
        match self {
            QueryNode::And(mut nodes) => {
                nodes.push(other);
                QueryNode::And(nodes)
            }
            _ => QueryNode::And(vec![self, other]),
        }
    }

    pub fn or(self, other: QueryNode) -> Self {
        match self {
            QueryNode::Or(mut nodes) => {
                nodes.push(other);
                QueryNode::Or(nodes)
            }
            _ => QueryNode::Or(vec![self, other]),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        QueryNode::Not(Box::new(self))
    }
}

impl std::fmt::Display for QueryNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryNode::Term(term) => write!(f, "{}", term),
            QueryNode::Phrase(phrase) => write!(f, "\"{}\"", phrase),
            QueryNode::Prefix(prefix) => write!(f, "{}*", prefix),
            QueryNode::FuzzyTerm(term, n) => {
                if let Some(n) = n {
                    write!(f, "{}~{}", term, n)
                } else {
                    write!(f, "{}~", term)
                }
            }
            QueryNode::ProximityPhrase(phrase, n) => write!(f, "\"{}\"~{}", phrase, n),
            QueryNode::Not(node) => match **node {
                QueryNode::And(_) | QueryNode::Or(_) => write!(f, "-({})", node),
                _ => write!(f, "-{}", node),
            },
            QueryNode::And(list) => {
                let content = list
                    .iter()
                    .map(|q| match q {
                        QueryNode::Or(_) => format!("({})", q),
                        _ => format!("{}", q),
                    })
                    .collect::<Vec<_>>()
                    .join(" + ");
                write!(f, "{}", content)
            }

            QueryNode::Or(list) => {
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
    /// A comma-separated list of the fields to be returned.
    fields: Option<Vec<PaperField>>,
    /// Provides the option to sort the results by the following fields:
    /// - `paperId`
    /// - `publicationDate`
    /// - `citationCount`
    ///
    /// Uses the format field:order. Ties are broken by paperId. The default field is paperId and the default order is asc. Records for which the sort value are not defined will appear at the end of sort, regardless of asc/desc order.
    ///
    /// ## Examples
    ///
    /// - `publicationDate:asc` - return oldest papers first.
    /// - `citationCount:desc` - return most highly-cited papers first.
    /// - `paperId` - return papers in ID order, low-to-high.
    ///
    /// Please be aware that if the relevant data changes while paging through results, records can be returned in an unexpected way. The default paperId sort avoids this edge case.
    sort: Option<SortBy>,
    /// Restricts results to any of the paper publication types.
    publication_types: Option<Vec<PublicationType>>,
    /// Restricts results to only include papers with a public PDF.
    /// This parameter does not accept any values.
    open_access_pdf: Option<bool>,
    /// Restricts results to only include papers with the minimum number of citations.
    min_citation_count: Option<u32>,
    /// Restricts results to the given range of publication dates. Accepts the format `<startDate>:<endDate>` with each date in YYYY-MM-DD format.
    ///
    /// Each term is optional, allowing for specific dates, fixed ranges, or open-ended ranges. In addition, prefixes are supported as a shorthand, e.g. 2020-06 matches all dates in June 2020.
    ///
    /// Specific dates are not known for all papers, so some records returned with this filter will have a null value for publicationDate. year, however, will always be present. For records where a specific publication date is not known, they will be treated as if published on January 1st of their publication year.
    ///
    /// ## Examples
    ///
    /// - `2019-03-05` on March 5th, 2019
    /// - `2019-03` during March 2019
    /// - `2016-03-05:2020-06-06` as early as March 5th, 2016 or as late as June 6th, 2020
    /// - `1981-08-25:` on or after August 25th, 1981
    /// - `:2015-01` before or on January 31st, 2015
    publication_date: Option<(Option<Date>, Option<Date>)>,
    /// Restricts results to the given publication year or range of years (inclusive).
    ///
    /// ## Examples
    /// - `2019` in 2019
    /// - `2016-2020` as early as 2016 or as late as 2020
    /// - `2010-` during or after 2010
    /// - `-2015` before or during 2015
    year: Option<(Option<u32>, Option<u32>)>,
    /// Restricts results to papers in the given fields of study, formatted as a comma-separated list.
    fields_of_study: Option<Vec<FieldOfStudy>>,
    /// Restricts results to papers published in the given venues, formatted as a comma-separated list.
    ///
    /// Input could also be an ISO4 abbreviation.
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
        let url = format!("{}/paper/search?{}", BASE_URL, self.query_string());
        let req_builder = build_request(client, Method::Get, &url);

        let resp = req_builder.send().await?;
        match resp.status() {
            StatusCode::OK => Ok(resp.json().await?),
            _ => Err(RequestFailedError {
                error: resp.text().await?,
            }
            .into()),
        }
    }
}

/// Builder for the paper search parameters
#[derive(Debug, Clone, Default)]
pub struct PaperBulkSearchParamBuilder {
    query: Option<QueryNode>,
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
    pub fn query(&mut self, query: &QueryNode) -> &mut Self {
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
    fn test_paper_search_param_builder() {
        let mut builder = PaperBulkSearchParamBuilder::default();
        builder
            .query(&QueryNode::term("test"))
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
        assert_eq!(param.year, Some((Some(2020), Some(2020))));
        assert_eq!(
            param.fields_of_study,
            Some(vec![FieldOfStudy::ComputerScience])
        );
    }
}
