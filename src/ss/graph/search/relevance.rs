//! Paper relevance search
//!
//! `Get /paper/search`
//!
//! `/paper/search?query={query}`
//!
//! ## Limitations
//! - Can only return up to 1,000 relevance-ranked results. For larger queries, see "/search/bulk" or the Datasets API.
//! - Can only return up to 10 MB of data at a time.

use crate::{
    error::{Error, Result},
    ss::{
        client::{Method, Query, RequestFailedError, SemanticScholar, build_request},
        graph::{
            _Date, BASE_URL, Date, FieldOfStudy, NestedPaper, PaperField, PublicationType,
            merge_fields_of_study, merge_paper_fields, merge_publication_types,
        },
    },
};
use reqwest::StatusCode;
use serde::Deserialize;

/// Query parameters for the paper search
#[derive(Debug, Clone)]
pub struct PaperSearchParam {
    /// A plain-text search query string.
    ///
    /// - No special query syntax is supported.
    /// - Hyphenated query terms yield no matches (replace it with space to find matches)
    query: String,
    /// A comma-separated list of the fields to be returned.
    fields: Option<Vec<PaperField>>,
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
    /// Used for pagination. When returning a list of results, start with the element at this position in the list (default: 0).
    offset: Option<u32>,
    /// The maximum number of results to return (default: 100).
    ///
    /// Must be <= 100.
    limit: Option<u8>,
}

impl PaperSearchParam {
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

        if let Some(offset) = self.offset {
            query_string.push_str(&format!("&offset={}", offset));
        }

        if let Some(limit) = self.limit {
            query_string.push_str(&format!("&limit={}", limit));
        }

        query_string
    }
}

impl Query for PaperSearchParam {
    type Response = PaperSearchResponse;

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
pub struct PaperSearchParamBuilder {
    query: String,
    fields: Option<Vec<PaperField>>,
    publication_types: Option<Vec<PublicationType>>,
    open_access_pdf: Option<bool>,
    min_citation_count: Option<u32>,
    publication_date: Option<(Option<_Date>, Option<_Date>)>,
    year: Option<(Option<u32>, Option<u32>)>,
    fields_of_study: Option<Vec<FieldOfStudy>>,
    venue: Option<Vec<String>>,
    offset: Option<u32>,
    limit: Option<u8>,
}

impl PaperSearchParamBuilder {
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

    /// Used for pagination. When returning a list of results, start with the element at this position in the list (default: 0).
    pub fn offset(&mut self, offset: u32) -> &mut Self {
        self.offset = Some(offset);
        self
    }

    /// The maximum number of results to return (default: 100).
    /// Must be <= 100.
    pub fn limit(&mut self, limit: u8) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    /// Build the paper search parameters
    pub fn build(&self) -> Result<PaperSearchParam> {
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

        Ok(PaperSearchParam {
            query: self.query.clone(),
            fields: self.fields.clone(),
            publication_types: self.publication_types.clone(),
            open_access_pdf: self.open_access_pdf,
            min_citation_count: self.min_citation_count,
            publication_date,
            year: self.year,
            fields_of_study: self.fields_of_study.clone(),
            venue: self.venue.clone(),
            offset: self.offset,
            limit: self.limit,
        })
    }
}

/// Response for the paper search
#[derive(Debug, Clone, Deserialize)]
pub struct PaperSearchResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<NestedPaper>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_relevance_search_param_builder() {
        let mut builder = PaperSearchParamBuilder::new("test");
        builder
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
