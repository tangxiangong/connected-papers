const BASE_URL: &str = "https://api.semanticscholar.org/graph/v1";

pub mod autocomplete;
pub use autocomplete::*;
pub mod batch;
pub use batch::*;
pub mod models;
pub use models::*;
pub mod search;
pub use search::*;
