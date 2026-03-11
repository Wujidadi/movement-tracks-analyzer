//! 高效的 KML GPS 軌跡分析庫
//!
//! 用於解析 KML 格式的 GPS 軌跡檔案，提取軌跡資料並支援多種輸出格式。
//!
//! # 功能
//!
//! - **流式 XML 解析**：高效處理大型 KML 檔案（50MB+）
//! - **軌跡分析**：計算距離、時間、座標等資訊
//! - **多格式輸出**：支援 JSON、CSV、TSV 和表格格式
//! - **自訂錯誤處理**：清晰的錯誤類型和消息
//!
//! # 範例
//!
//! ```rust
//! use movement_tracks_analyzer::{extract_placemarks_with_paths, format_output, OutputFormat};
//! use std::path::PathBuf;
//!
//! let file_path = PathBuf::from("tracks.kml");
//! let placemarks = extract_placemarks_with_paths(&file_path)?;
//! let output = format_output(OutputFormat::Json, &placemarks);
//! println!("{}", output);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

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
