//! Semantic Scholar Graph API

const BASE_URL: &str = "https://api.semanticscholar.org/graph/v1";

pub mod autocomplete;
pub use autocomplete::*;
pub mod batch;
pub use batch::*;
pub mod search;
pub use search::*;
