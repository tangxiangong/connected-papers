//! Paper Search `/paper/search/*`
//!

pub mod relevance;
pub use relevance::*;
pub mod bulk;
pub use bulk::*;
pub mod paper_id;
pub mod title;
pub use title::*;
