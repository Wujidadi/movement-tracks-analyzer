use crate::config::{Config, OutputType};
use movement_tracks_analyzer::{format_output, OutputFormat, Result, TrackMetadata};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

/// 輸出結果
pub fn output_results(placemarks: &[(Vec<String>, TrackMetadata)], config: &Config) -> Result<()> {
    let output = format_output(config.format, placemarks);
    dispatch_output(&output, config)
}

/// 依輸出目標分派結果
fn dispatch_output(output: &str, config: &Config) -> Result<()> {
    match config.output_type {
        OutputType::Shell => {
            print!("{}", output);
            Ok(())
        }
        OutputType::File => save_to_file(output, config.format, config.export_path.as_deref()),
    }
}

/// 儲存輸出到檔案
fn save_to_file(output: &str, format: OutputFormat, export_path: Option<&Path>) -> Result<()> {
    let file_path = determine_file_path(export_path, format);

    let mut file = File::create(&file_path)?;
    file.write_all(output.as_bytes())?;
    println!("Output saved to: {}", file_path.display());

    Ok(())
}

/// 確定輸出檔案路徑
///
/// 支援：
/// 1. 未指定路徑 → 使用當前目錄預設檔名
/// 2. 目錄路徑 → 使用預設檔名 (e.g., `/tmp` → `/tmp/tracks_output.csv`)
/// 3. 完整檔案路徑 → 直接使用 (e.g., `/tmp/data.csv` → `/tmp/data.csv`)
fn determine_file_path(export_path: Option<&Path>, format: OutputFormat) -> PathBuf {
    let Some(path) = export_path else {
        return get_default_filename(format);
    };
    resolve_export_path(path, format)
}

/// 解析使用者指定的匯出路徑
fn resolve_export_path(path: &Path, format: OutputFormat) -> PathBuf {
    if has_file_extension(path) {
        path.to_path_buf()
    } else {
        path.join(get_default_filename(format))
    }
}

/// 判斷路徑是否包含檔案副檔名
fn has_file_extension(path: &Path) -> bool {
    path.extension().is_some()
}

/// 根據輸出格式取得預設檔名
fn get_default_filename(format: OutputFormat) -> PathBuf {
    let filename = match format {
        OutputFormat::Json => "tracks_output.json",
        OutputFormat::Csv => "tracks_output.csv",
        OutputFormat::Tsv => "tracks_output.tsv",
        OutputFormat::Table => "tracks_output.csv",
    };
    PathBuf::from(filename)
}
