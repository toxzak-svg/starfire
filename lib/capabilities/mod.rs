//! Capabilities — things Star can do (file reading, web search, etc.)

pub mod reader;
pub mod web_reader;

pub use reader::FileReader;
pub use web_reader::WebReader;
