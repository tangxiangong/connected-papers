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
            BASE_URL, FieldOfStudy, Paper, PaperField, PublicationType, merge_fields_of_study,
            merge_paper_fields, merge_publication_types,
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
    pub query: String,
    /// A comma-separated list of the fields to be returned.
    pub fields: Option<Vec<PaperField>>,
    /// Restricts results to any of the paper publication types.
    pub publication_types: Option<Vec<PublicationType>>,
    /// Restricts results to only include papers with a public PDF.
    /// This parameter does not accept any values.
    pub open_access_pdf: Option<bool>,
    /// Restricts results to only include papers with the minimum number of citations.
    pub min_citation_count: Option<u32>,
    /// Restricts results to the given range of publication dates or years (inclusive). Accepts the format <startDate>:<endDate> with each date in YYYY-MM-DD format.
    ///
    /// Each term is optional, allowing for specific dates, fixed ranges, or open-ended ranges. In addition, prefixes are supported as a shorthand, e.g. 2020-06 matches all dates in June 2020.
    ///
    /// Specific dates are not known for all papers, so some records returned with this filter will have a null value for publicationDate. year, however, will always be present. For records where a specific publication date is not known, they will be treated as if published on January 1st of their publication year.
    ///
    /// ## Examples
    ///
    /// - `2019-03-05` on March 5th, 2019
    /// - `2019-03` during March 2019
    /// - `2019` during 2019
    /// - `2016-03-05:2020-06-06` as early as March 5th, 2016 or as late as June 6th, 2020
    /// - `1981-08-25:` on or after August 25th, 1981
    /// - `:2015-01` before or on January 31st, 2015
    /// - `2015:2020` between January 1st, 2015 and December 31st, 2020
    pub publication_date_or_year: Option<String>,
    /// Restricts results to the given publication year or range of years (inclusive).
    ///
    /// ## Examples
    /// - `2019` in 2019
    /// - `2016-2020` as early as 2016 or as late as 2020
    /// - `2010-` during or after 2010
    /// - `-2015` before or during 2015
    pub year: Option<YearRange>,
    /// Restricts results to papers in the given fields of study, formatted as a comma-separated list.
    pub fields_of_study: Option<Vec<FieldOfStudy>>,
    /// Restricts results to papers published in the given venues, formatted as a comma-separated list.
    ///
    /// Input could also be an ISO4 abbreviation.
    pub venue: Option<Vec<String>>,
    /// Used for pagination. When returning a list of results, start with the element at this position in the list (default: 0).
    pub offset: Option<u32>,
    /// The maximum number of results to return (default: 100).
    ///
    /// Must be <= 100.
    pub limit: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YearRange {
    start: Option<u32>,
    end: Option<u32>,
}

impl YearRange {
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
        }
    }

    pub fn from(start: u32) -> Self {
        Self {
            start: Some(start),
            end: None,
        }
    }

    pub fn to(end: u32) -> Self {
        Self {
            start: None,
            end: Some(end),
        }
    }

    pub fn at(year: u32) -> Self {
        Self {
            start: Some(year),
            end: Some(year),
        }
    }
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

        if let Some(ref publication_date_or_year) = self.publication_date_or_year {
            query_string.push_str(&format!(
                "&publicationDateOrYear={}",
                publication_date_or_year
            ));
        }

        if let Some(year) = self.year {
            match (year.start, year.end) {
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
    publication_date_or_year: Option<String>,
    year: Option<YearRange>,
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

    /// Restricts results to the given range of publication dates or years (inclusive). Accepts the format <startDate>:<endDate> with each date in YYYY-MM-DD format.
    pub fn publication_date_or_year(&mut self, publication_date_or_year: &str) -> &mut Self {
        self.publication_date_or_year = Some(publication_date_or_year.to_owned());
        self
    }

    /// Restricts results to the given publication year range (inclusive).
    pub fn year_range(&mut self, start: u32, end: u32) -> &mut Self {
        self.year = Some(YearRange::new(start, end));
        self
    }

    /// Restricts results to the given publication year start to now.
    pub fn year_from(&mut self, start: u32) -> &mut Self {
        self.year = Some(YearRange::from(start));
        self
    }

    /// Restricts results to papers published before or during the given year.
    pub fn year_to(&mut self, end: u32) -> &mut Self {
        self.year = Some(YearRange::to(end));
        self
    }

    /// Restricts results to papers published in the given year.
    pub fn year_at(&mut self, year: u32) -> &mut Self {
        self.year = Some(YearRange::at(year));
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
            && let Some(start) = year.start
            && let Some(end) = year.end
            && start > end
        {
            return Err(Error::InvalidParameter(
                "start year must be less than or equal to end year".to_string(),
            ));
        }
        Ok(PaperSearchParam {
            query: self.query.clone(),
            fields: self.fields.clone(),
            publication_types: self.publication_types.clone(),
            open_access_pdf: self.open_access_pdf,
            min_citation_count: self.min_citation_count,
            publication_date_or_year: self.publication_date_or_year.clone(),
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
    pub title: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<Paper>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_search_param_builder() {
        let mut builder = PaperSearchParamBuilder::new("test");
        builder
            .field(PaperField::Title)
            .publication_type(PublicationType::JournalArticle)
            .open_access_pdf()
            .min_citation_count(10)
            .publication_date_or_year("2020-01-01:2020-12-31")
            .year_at(2020)
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
            param.publication_date_or_year,
            Some("2020-01-01:2020-12-31".to_owned())
        );
        assert_eq!(param.year, Some(YearRange::at(2020)));
        assert_eq!(
            param.fields_of_study,
            Some(vec![FieldOfStudy::ComputerScience])
        );
    }
}
