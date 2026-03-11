// 公開模組
pub mod format;
pub mod metadata;
pub mod parser;
pub mod path;
pub mod regex;

// 重新導出常用類型
pub use format::{format_output, OutputFormat};
pub use metadata::TrackMetadata;
pub use parser::extract_placemarks_with_paths;
pub use path::extract_categories;
pub use regex::{START_TIME_PATTERN, END_TIME_PATTERN};
