// 公開模組
pub mod error;
pub mod format;
pub mod metadata;
pub mod parser;
pub mod path;
pub mod regex;

// 重新導出常用 symbol
pub use error::{AnalyzerError, Result};
pub use format::{format_output, OutputFormat};
pub use metadata::TrackMetadata;
pub use parser::extract_placemarks_with_paths;
pub use path::extract_categories;
pub use regex::{END_TIME_PATTERN, START_TIME_PATTERN};
