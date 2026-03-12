//! 高效的 KML GPS 軌跡分析庫
//!
//! 用於解析 KML 格式的 GPS 軌跡檔案，提取軌跡資料並支援多種輸出格式。
//!
//! # 功能
//!
//! - **流式 XML 解析**：高效處理大型 KML 檔案（50MB+）
//! - **軌跡分析**：計算距離、時間、座標等資訊
//! - **多格式輸出**：支援 JSON、CSV、TSV 和表格格式
//! - **自訂錯誤處理**：清晰的錯誤類型和訊息
//!
//! # 範例
//!
//! ```
//! use movement_tracks_analyzer::{extract_placemarks_with_paths, format_output, OutputFormat};
//! use std::path::PathBuf;
//!
//! // 解析 KML 檔案
//! let placemarks = extract_placemarks_with_paths(&PathBuf::from("tests/fixtures/tracks.kml"))?;
//! let json = format_output(OutputFormat::Json, &placemarks);
//! assert!(!json.is_empty());
//! assert!(json.contains("2026-03"));
//!
//! // 輸出為 CSV
//! let csv = format_output(OutputFormat::Csv, &placemarks);
//! assert!(csv.contains("Start,End"));
//!
//! // 輸出為表格
//! let table = format_output(OutputFormat::Table, &placemarks);
//! assert!(table.contains("2026-03"));
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
