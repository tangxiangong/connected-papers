use crate::{
    error::Result,
    ss::client::{Method, Query, RequestFailedError, SemanticScholar, build_request},
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize, Serializer};

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

#[derive(Debug, Clone)]
pub struct BatchDetailQueryParam(pub Vec<PaperId>, pub Option<Vec<PaperField>>);

impl Query for BatchDetailQueryParam {
    type Response = reqwest::Response;

    async fn query(&self, client: &SemanticScholar) -> Result<Self::Response> {
        let paper_ids = PaperIds {
            ids: self.0.clone(),
        };
        let url = if let Some(ref fields) = self.1
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
            StatusCode::OK => Ok(resp),
            _ => Err(RequestFailedError {
                error: resp.text().await?,
            }
            .into()),
        }
        // TODO: parse the response
    }
}

#[derive(Debug, Clone, Serialize)]
struct PaperIds {
    ids: Vec<PaperId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaperId {
    /// Semantic Scholar ID, e.g. `649def34f8be52c8b66281af98ae884c09aef38b`
    S2Id(String),
    /// a Semantic Scholar numerical ID, e.g. `CorpusId:215416146`
    CorpusId(u64),
    /// a Digital Object Identifier, e.g. `DOI:10.18653/v1/N18-3011`
    DOI(String),
    /// arXiv.org, e.g. `ARXIV:2106.15928`
    ArXiv(String),
    /// Microsoft Academic Graph, e.g. `MAG:112218234`
    MAG(u64),
    /// Association for Computational Linguistics, e.g. `ACL:W12-3903`
    ACL(String),
    /// PubMed/Medline, e.g. `PMID:19872477`
    PubMed(u64),
    /// PubMed Central, e.g. `PMCID:2323736`
    PubMedCentral(u64),
    /// URL from one of the sites listed below, e.g. `URL:https://arxiv.org/abs/2106.15928v1`
    URL(String),
}

impl PaperId {
    /// Create a Semantic Scholar ID from a string-like value
    #[inline]
    pub fn id<S: Into<String>>(s: S) -> Self {
        PaperId::S2Id(s.into())
    }

    /// Create a corpus ID from a numerical value
    #[inline]
    pub fn corpus(id: u64) -> Self {
        PaperId::CorpusId(id)
    }

    /// Create a DOI from a string-like value
    #[inline]
    pub fn doi<S: Into<String>>(s: S) -> Self {
        PaperId::DOI(s.into())
    }

    /// Create an arXiv ID from a string-like value
    #[inline]
    pub fn arxiv<S: Into<String>>(s: S) -> Self {
        PaperId::ArXiv(s.into())
    }

    /// Create an ACL ID from a string-like value
    #[inline]
    pub fn acl<S: Into<String>>(s: S) -> Self {
        PaperId::ACL(s.into())
    }

    /// Create a URL from a string-like value
    #[inline]
    pub fn url<S: Into<String>>(s: S) -> Self {
        PaperId::URL(s.into())
    }

    /// Create a PubMed ID from a numerical value
    #[inline]
    pub fn pubmed(id: u64) -> Self {
        PaperId::PubMed(id)
    }

    /// Create a PubMed Central ID from a numerical value
    #[inline]
    pub fn pubmed_central(id: u64) -> Self {
        PaperId::PubMedCentral(id)
    }

    /// Create a MAG ID from a numerical value
    #[inline]
    pub fn mag(id: u64) -> Self {
        PaperId::MAG(id)
    }
}

impl Serialize for PaperId {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PaperId::S2Id(s) => serializer.serialize_str(s),
            PaperId::CorpusId(v) => serializer.serialize_str(&format!("CorpusId:{}", v)),
            PaperId::DOI(v) => serializer.serialize_str(&format!("DOI:{}", v)),
            PaperId::ArXiv(v) => serializer.serialize_str(&format!("ARXIV:{}", v)),
            PaperId::MAG(v) => serializer.serialize_str(&format!("MAG:{}", v)),
            PaperId::ACL(v) => serializer.serialize_str(&format!("ACL:{}", v)),
            PaperId::PubMed(v) => serializer.serialize_str(&format!("PMID:{}", v)),
            PaperId::PubMedCentral(v) => serializer.serialize_str(&format!("PMCID:{}", v)),
            PaperId::URL(v) => serializer.serialize_str(&format!("URL:{}", v)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PaperField {
    CorpusId,
    ExternalIds,
    URL,
    Title,
    Abstract,
    Venue,
    PublicationVenue,
    Year,
    ReferenceCount,
    CitationCount,
    InfluentialCitationCount,
    IsOpenAccess,
    OpenAccessPDF,
    FieldsOfStudy,
    S2FieldsOfStudy,
    PublicationTypes,
    PublicationDate,
    Journal,
    CitationStyles,
    Authors,
    Citations,
    References,
    Embedding,
    Tldr,
}

impl std::fmt::Display for PaperField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaperField::CorpusId => write!(f, "corpusId"),
            PaperField::ExternalIds => write!(f, "externalIds"),
            PaperField::URL => write!(f, "url"),
            PaperField::Title => write!(f, "title"),
            PaperField::Abstract => write!(f, "abstract"),
            PaperField::Venue => write!(f, "venue"),
            PaperField::PublicationVenue => write!(f, "publicationVenue"),
            PaperField::Year => write!(f, "year"),
            PaperField::ReferenceCount => write!(f, "referenceCount"),
            PaperField::CitationCount => write!(f, "citationCount"),
            PaperField::InfluentialCitationCount => write!(f, "influentialCitationCount"),
            PaperField::IsOpenAccess => write!(f, "isOpenAccess"),
            PaperField::OpenAccessPDF => write!(f, "openAccessPdf"),
            PaperField::FieldsOfStudy => write!(f, "fieldsOfStudy"),
            PaperField::S2FieldsOfStudy => write!(f, "s2FieldsOfStudy"),
            PaperField::PublicationTypes => write!(f, "publicationTypes"),
            PaperField::PublicationDate => write!(f, "publicationDate"),
            PaperField::Journal => write!(f, "journal"),
            PaperField::CitationStyles => write!(f, "citationStyles"),
            PaperField::Authors => write!(f, "authors"),
            PaperField::Citations => write!(f, "citations"),
            PaperField::References => write!(f, "references"),
            PaperField::Embedding => write!(f, "embedding"),
            PaperField::Tldr => write!(f, "tldr"),
        }
    }
}

fn merge_paper_fields(fields: &[PaperField]) -> String {
    fields
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join(",")
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

    #[test]
    fn test_id_serialization() {
        let id = PaperId::id("649def34f8be52c8b66281af98ae884c09aef38b");
        let id_serialized = serde_json::to_string(&id).unwrap();
        assert_eq!(
            id_serialized,
            "\"649def34f8be52c8b66281af98ae884c09aef38b\""
        );
        let corpus_id = PaperId::CorpusId(215416146);
        let corpus_id_serialized = serde_json::to_string(&corpus_id).unwrap();
        assert_eq!(corpus_id_serialized, "\"CorpusId:215416146\"");
        let doi = PaperId::doi("10.18653/v1/N18-3011");
        let doi_serialized = serde_json::to_string(&doi).unwrap();
        assert_eq!(doi_serialized, "\"DOI:10.18653/v1/N18-3011\"");
        let arxiv = PaperId::arxiv("2106.15928");
        let arxiv_serialized = serde_json::to_string(&arxiv).unwrap();
        assert_eq!(arxiv_serialized, "\"ARXIV:2106.15928\"");
        let mag = PaperId::mag(112218234);
        let mag_serialized = serde_json::to_string(&mag).unwrap();
        assert_eq!(mag_serialized, "\"MAG:112218234\"");
        let acl = PaperId::acl("W12-3903");
        let acl_serialized = serde_json::to_string(&acl).unwrap();
        assert_eq!(acl_serialized, "\"ACL:W12-3903\"");
        let pubmed = PaperId::pubmed(19872477);
        let pubmed_serialized = serde_json::to_string(&pubmed).unwrap();
        assert_eq!(pubmed_serialized, "\"PMID:19872477\"");
        let pubmed_central = PaperId::pubmed_central(2323736);
        let pubmed_central_serialized = serde_json::to_string(&pubmed_central).unwrap();
        assert_eq!(pubmed_central_serialized, "\"PMCID:2323736\"");
        let url = PaperId::url("https://arxiv.org/abs/2106.15928v1");
        let url_serialized = serde_json::to_string(&url).unwrap();
        assert_eq!(url_serialized, "\"URL:https://arxiv.org/abs/2106.15928v1\"");
        let ids = PaperIds {
            ids: vec![
                id,
                corpus_id,
                doi,
                arxiv,
                mag,
                acl,
                pubmed,
                pubmed_central,
                url,
            ],
        };
        let ids_serialized = serde_json::to_string(&ids).unwrap();
        assert_eq!(
            ids_serialized,
            "{\"ids\":[\"649def34f8be52c8b66281af98ae884c09aef38b\",\"CorpusId:215416146\",\"DOI:10.18653/v1/N18-3011\",\"ARXIV:2106.15928\",\"MAG:112218234\",\"ACL:W12-3903\",\"PMID:19872477\",\"PMCID:2323736\",\"URL:https://arxiv.org/abs/2106.15928v1\"]}"
        );
    }

    #[test]
    fn test_paper_field_merge() {
        let fields = vec![
            PaperField::CorpusId,
            PaperField::ExternalIds,
            PaperField::URL,
            PaperField::Title,
            PaperField::Abstract,
            PaperField::Venue,
            PaperField::PublicationVenue,
            PaperField::Year,
            PaperField::ReferenceCount,
            PaperField::CitationCount,
            PaperField::InfluentialCitationCount,
            PaperField::IsOpenAccess,
            PaperField::OpenAccessPDF,
            PaperField::FieldsOfStudy,
            PaperField::S2FieldsOfStudy,
            PaperField::PublicationTypes,
            PaperField::PublicationDate,
            PaperField::Journal,
            PaperField::CitationStyles,
            PaperField::Authors,
            PaperField::Citations,
            PaperField::References,
            PaperField::Embedding,
            PaperField::Tldr,
        ];
        let fields_merged = merge_paper_fields(&fields);
        assert_eq!(
            fields_merged,
            "corpusId,externalIds,url,title,abstract,venue,publicationVenue,year,referenceCount,citationCount,influentialCitationCount,isOpenAccess,openAccessPdf,fieldsOfStudy,s2FieldsOfStudy,publicationTypes,publicationDate,journal,citationStyles,authors,citations,references,embedding,tldr"
        );
    }

    #[tokio::test]
    async fn test_batch_query() {
        let ids = vec![PaperId::id("649def34f8be52c8b66281af98ae884c09aef38b")];
        let fields = vec![PaperField::IsOpenAccess];
        let param = BatchDetailQueryParam(ids, Some(fields));

        let client = SemanticScholar::default();
        let res = client.query(&param).await.unwrap();
        println!("{:?}", res);
    }
}
