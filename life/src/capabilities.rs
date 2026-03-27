//! Capabilities — Tools Star can use
//!
//! These are the abilities that let Star interact with the world.
//! Each capability is self-contained and can be extended.

pub mod reader;
pub mod websearch;

pub use reader::FileReader;
pub use websearch::WebSearch;
