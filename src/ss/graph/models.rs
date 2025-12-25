//! Models for the Semantic Scholar Graph API
//!

use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashSet;

/// Paper ID
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
    /// URL from one of the sites listed below, e.g. `URL:<https://arxiv.org/abs/2106.15928v1>`
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

/// Paper field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FieldOfStudy {
    #[serde(rename = "Computer Science")]
    ComputerScience,
    Medicine,
    Chemistry,
    Biology,
    #[serde(rename = "Materials Science")]
    MaterialsScience,
    Physics,
    Geology,
    Psychology,
    Art,
    History,
    Geography,
    Sociology,
    Business,
    #[serde(rename = "Political Science")]
    PoliticalScience,
    Economics,
    Philosophy,
    Mathematics,
    Engineering,
    #[serde(rename = "Environmental Science")]
    EnvironmentalScience,
    #[serde(rename = "Agricultural and Food Sciences")]
    AgriculturalAndFoodSciences,
    Education,
    Law,
    Linguistics,
}

impl std::fmt::Display for FieldOfStudy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldOfStudy::ComputerScience => write!(f, "Computer Science"),
            FieldOfStudy::Medicine => write!(f, "Medicine"),
            FieldOfStudy::Chemistry => write!(f, "Chemistry"),
            FieldOfStudy::Biology => write!(f, "Biology"),
            FieldOfStudy::MaterialsScience => write!(f, "Materials Science"),
            FieldOfStudy::Physics => write!(f, "Physics"),
            FieldOfStudy::Geology => write!(f, "Geology"),
            FieldOfStudy::Psychology => write!(f, "Psychology"),
            FieldOfStudy::Art => write!(f, "Art"),
            FieldOfStudy::History => write!(f, "History"),
            FieldOfStudy::Geography => write!(f, "Geography"),
            FieldOfStudy::Sociology => write!(f, "Sociology"),
            FieldOfStudy::Business => write!(f, "Business"),
            FieldOfStudy::PoliticalScience => write!(f, "Political Science"),
            FieldOfStudy::Economics => write!(f, "Economics"),
            FieldOfStudy::Philosophy => write!(f, "Philosophy"),
            FieldOfStudy::Mathematics => write!(f, "Mathematics"),
            FieldOfStudy::Engineering => write!(f, "Engineering"),
            FieldOfStudy::EnvironmentalScience => write!(f, "Environmental Science"),
            FieldOfStudy::AgriculturalAndFoodSciences => {
                write!(f, "Agricultural and Food Sciences")
            }
            FieldOfStudy::Education => write!(f, "Education"),
            FieldOfStudy::Law => write!(f, "Law"),
            FieldOfStudy::Linguistics => write!(f, "Linguistics"),
        }
    }
}

/// Merge paper fields into a comma-separated string
pub(crate) fn merge_paper_fields(fields: &[PaperField]) -> String {
    fields
        .iter()
        .copied()
        .collect::<HashSet<_>>()
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join(",")
}

/// Merge paper publication types into a comma-separated string
pub(crate) fn merge_publication_types(types: &[PublicationType]) -> String {
    types
        .iter()
        .copied()
        .collect::<HashSet<_>>()
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join(",")
}

/// Merge paper fields of study into a comma-separated string
pub(crate) fn merge_fields_of_study(fields: &[FieldOfStudy]) -> String {
    fields
        .iter()
        .copied()
        .collect::<HashSet<_>>()
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join(",")
}

/// Inner struct for the paper/batch query response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paper {
    /// Semantic Scholar's primary unique identifier for a paper.
    pub paper_id: String,
    /// Semantic Scholar's secondary unique identifier for a paper.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corpus_id: Option<u64>,
    /// An object that contains the paper's unique identifiers in external sources.
    /// The external sources are limited to: ArXiv, MAG, ACL, PubMed, Medline, PubMedCentral, DBLP, and DOI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ids: Option<ExternalIds>,
    /// URL of the paper on the Semantic Scholar website.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Title of the paper.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The paper's abstract. Note that due to legal reasons, this may be missing even if we display an abstract on the website.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "abstract")]
    pub abstract_: Option<String>,
    /// The name of the paper's publication venue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    /// An object that contains the following information about the journal or
    /// conference in which this paper was published: id (the venue's unique ID),
    /// name (the venue's name), type (the type of venue), alternate_names (an array
    /// of alternate names for the venue), and url (the venue's website).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_venue: Option<PublicationVenue>,
    /// The year the paper was published.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    /// The total number of papers this paper references.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_count: Option<u32>,
    /// The total number of papers that references this paper.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_count: Option<u32>,
    /// A subset of the citation count, where the cited publication has a significant
    /// impact on the citing publication. Determined by Semantic Scholar's algorithm:
    /// <https://www.semanticscholar.org/faq#influential-citations>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub influential_citation_count: Option<u32>,
    /// Whether the paper is open access. More information here: <https://www.openaccess.nl/en/what-is-open-access>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_open_access: Option<bool>,
    /// An object that contains the following parameters: url (a link to the paper's
    /// PDF), status (the type of open access <https://en.wikipedia.org/wiki/Open_access#Colour_naming_system>), the paper's license, and a legal disclaimer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_access_pdf: Option<OpenAccessPdf>,
    /// A list of the paper's high-level academic categories from external sources.
    /// The possible fields are: Computer Science, Medicine, Chemistry, Biology,
    /// Materials Science, Physics, Geology, Psychology, Art, History, Geography,
    /// Sociology, Business, Political Science, Economics, Philosophy,
    /// Mathematics, Engineering, Environmental Science, Agricultural and
    /// Food Sciences, Education, Law, and Linguistics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_of_study: Option<Vec<FieldOfStudy>>,
    /// An array of objects. Each object contains the following parameters: category (a field of study. The possible fields are the same as in
    /// fieldsOfStudy), and source (specifies whether the category was classified by Semantic Scholar or by an external source. More information on how Semantic
    /// Scholar classifies papers <https://medium.com/ai2-blog/announcing-s2fos-an-open-source-academic-field-of-study-classifier-9d2f641949e5>).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s2_fields_of_study: Option<Vec<S2FieldsOfStudy>>,
    /// The type of this publication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_types: Option<Vec<PublicationType>>,
    /// The date when this paper was published, in YYYY-MM-DD format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_date: Option<String>,
    /// An object that contains the following parameters, if available: name (the journal name), volume (the journal’s volume number), and pages (the page number range).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal: Option<Journal>,
    /// The BibTex bibliographical citation of the paper.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_styles: Option<CitationStyles>,
    /// Array of authors info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<Author>>,
    /// Array of papers that cite this paper.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<AssociatedPaper>>,
    /// Array of papers that this paper cites.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<Vec<AssociatedPaper>>,
    /// Embedding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Embedding>,
    /// fulltext, abstract, or none, based on what we have available for this paper.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_availability: Option<String>,
}

/// Inner struct for the embedding field in the paper/batch query response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Embedding {
    /// The Spector vector embedding model version: <https://github.com/allenai/spector>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Numerical embedding vector.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<Vec<f64>>,
}

/// Inner struct for the tldr field in the paper/batch query response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tldr {
    /// The tldr model version number: <https://github.com/allenai/scitldr>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The TL;DR summary of the paper.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Inner struct for the citation styles field in the paper/batch query response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationStyles {
    pub bibtex: Option<String>,
}

/// Inner struct for the associated paper field in the paper/batch query response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssociatedPaper {
    /// Semantic Scholar's primary unique identifier for a paper.
    pub paper_id: String,
    /// Semantic Scholar's secondary unique identifier for a paper.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corpus_id: Option<u64>,
    /// An object that contains the paper's unique identifiers in external sources.
    /// The external sources are limited to: ArXiv, MAG, ACL, PubMed, Medline, PubMedCentral, DBLP, and DOI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ids: Option<ExternalIds>,
    /// URL of the paper on the Semantic Scholar website.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Title of the paper.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The paper's abstract. Note that due to legal reasons, this may be missing even if we display an abstract on the website.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "abstract")]
    pub abstract_: Option<String>,
    /// The name of the paper's publication venue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    /// An object that contains the following information about the journal or
    /// conference in which this paper was published: id (the venue's unique ID),
    /// name (the venue's name), type (the type of venue), alternate_names (an array
    /// of alternate names for the venue), and url (the venue's website).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_venue: Option<PublicationVenue>,
    /// The year the paper was published.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    /// The total number of papers this paper references.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_count: Option<u32>,
    /// The total number of papers that references this paper.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_count: Option<u32>,
    /// A subset of the citation count, where the cited publication has a significant
    /// impact on the citing publication. Determined by Semantic Scholar's algorithm:
    /// <https://www.semanticscholar.org/faq#influential-citations>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub influential_citation_count: Option<u32>,
    /// Whether the paper is open access. More information here: <https://www.openaccess.nl/en/what-is-open-access>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_open_access: Option<bool>,
    /// An object that contains the following parameters: url (a link to the paper's
    /// PDF), status (the type of open access <https://en.wikipedia.org/wiki/Open_access#Colour_naming_system>), the paper's license, and a legal disclaimer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_access_pdf: Option<OpenAccessPdf>,
    /// A list of the paper's high-level academic categories from external sources.
    /// The possible fields are: Computer Science, Medicine, Chemistry, Biology,
    /// Materials Science, Physics, Geology, Psychology, Art, History, Geography,
    /// Sociology, Business, Political Science, Economics, Philosophy,
    /// Mathematics, Engineering, Environmental Science, Agricultural and
    /// Food Sciences, Education, Law, and Linguistics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_of_study: Option<Vec<FieldOfStudy>>,
    /// An array of objects. Each object contains the following parameters: category (a field of study. The possible fields are the same as in
    /// fieldsOfStudy), and source (specifies whether the category was classified by Semantic Scholar or by an external source. More information on how Semantic
    /// Scholar classifies papers <https://medium.com/ai2-blog/announcing-s2fos-an-open-source-academic-field-of-study-classifier-9d2f641949e5>).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s2_fields_of_study: Option<Vec<S2FieldsOfStudy>>,
    /// The type of this publication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_types: Option<Vec<PublicationType>>,
    /// The date when this paper was published, in YYYY-MM-DD format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_date: Option<String>,
    /// An object that contains the following parameters, if available: name (the journal name), volume (the journal’s volume number), and pages (the page number range).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal: Option<Journal>,
    /// The BibTex bibliographical citation of the paper.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_styles: Option<CitationStyles>,
    /// Array of authors info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<Author>>,
}

/// Inner struct for the author field in the paper query response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    /// Semantic Scholar's unique ID for the author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<String>,
    /// An object that contains the ORCID/DBLP IDs for the author, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ids: Option<AuthorExternalIds>,
    /// URL of the author on the Semantic Scholar website.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Author's name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Array of organizational affiliations for the author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affiliations: Option<Vec<String>>,
    /// The author’s homepage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// The author's total publications count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper_count: Option<String>,
    /// The author's total citations count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_count: Option<String>,
    /// The author’s h-index, which is a measure of the productivity and citation impact of the author’s publications: <https://www.semanticscholar.org/faq#h-index>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h_index: Option<String>,
}

/// Inner struct for the author external ids field in the paper query response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct AuthorExternalIds {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orcid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dblp: Option<String>,
}

/// Inner struct for the journal field in the paper query response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Journal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<String>,
}

/// Inner struct for the s2 fields of study field in the paper query response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S2FieldsOfStudy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Inner struct for the open access pdf field in the paper query response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAccessPdf {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_disclaimer: Option<String>,
}

/// Publication type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PublicationType {
    Review,
    JournalArticle,
    CaseReport,
    ClinicalTrial,
    Conference,
    Dataset,
    Editorial,
    LettersAndComments,
    MetaAnalysis,
    News,
    Study,
    Book,
    BookSection,
}

impl std::fmt::Display for PublicationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicationType::Review => write!(f, "Review"),
            PublicationType::JournalArticle => write!(f, "JournalArticle"),
            PublicationType::CaseReport => write!(f, "CaseReport"),
            PublicationType::ClinicalTrial => write!(f, "ClinicalTrial"),
            PublicationType::Conference => write!(f, "Conference"),
            PublicationType::Dataset => write!(f, "Dataset"),
            PublicationType::Editorial => write!(f, "Editorial"),
            PublicationType::LettersAndComments => write!(f, "LettersAndComments"),
            PublicationType::MetaAnalysis => write!(f, "MetaAnalysis"),
            PublicationType::News => write!(f, "News"),
            PublicationType::Study => write!(f, "Study"),
            PublicationType::Book => write!(f, "Book"),
            PublicationType::BookSection => write!(f, "BookSection"),
        }
    }
}

/// Inner struct for the publication venue field in the paper query response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicationVenue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Inner struct for the external ids field in the paper query response
#[derive(Debug, Clone, Deserialize)]
pub struct ExternalIds {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "CorpusId")]
    pub corpus_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ArXiv")]
    pub arxiv: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "MAG")]
    pub mag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ACL")]
    pub acl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "PubMed")]
    pub pubmed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "PubMedCentral")]
    pub pubmed_central: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DBLP")]
    pub dblp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DOI")]
    pub doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Medline")]
    pub medline: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
