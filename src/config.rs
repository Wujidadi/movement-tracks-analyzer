use movement_tracks_analyzer::OutputFormat;
use std::path::PathBuf;

/// CLI 參數設定
pub struct Config {
    pub kml_file: PathBuf,
    pub output_type: OutputType,
    pub format: OutputFormat,
    pub export_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputType {
    Shell,
    File,
}
