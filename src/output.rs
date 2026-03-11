use crate::config::{Config, OutputType};
use movement_tracks_analyzer::{OutputFormat, TrackMetadata};
use std::{
    error::Error,
    path::{Path, PathBuf},
};

/// 輸出結果
pub fn output_results(
    placemarks: &[(Vec<String>, TrackMetadata)],
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    use movement_tracks_analyzer::format_output;

    let output = format_output(config.format, placemarks);

    match config.output_type {
        OutputType::Shell => {
            print!("{}", output);
        }
        OutputType::File => {
            save_to_file(&output, config.format, config.export_path.as_deref())?;
        }
    }

    Ok(())
}

/// 儲存輸出到檔案
fn save_to_file(
    output: &str,
    format: OutputFormat,
    export_path: Option<&Path>,
) -> Result<(), Box<dyn Error>> {
    use std::{fs::File, io::Write};

    let file_path = determine_file_path(export_path, format)?;

    let mut file = File::create(&file_path)?;
    file.write_all(output.as_bytes())?;
    println!("Output saved to: {}", file_path.display());

    Ok(())
}

/// 確定輸出檔案路徑
///
/// 支援：
/// 1. 目錄路徑 → 使用預設檔名 (e.g., `/tmp` → `/tmp/tracks_output.csv`)
/// 2. 完整檔案路徑 → 直接使用 (e.g., `/tmp/data.csv` → `/tmp/data.csv`)
fn determine_file_path(
    export_path: Option<&Path>,
    format: OutputFormat,
) -> Result<PathBuf, Box<dyn Error>> {
    match export_path {
        None => {
            // 未指定路徑，輸出到當前目錄
            Ok(get_default_filename(format))
        }
        Some(path) => {
            // 檢查路徑是否已包含檔名（判斷依據：是否有副檔名）
            if has_file_extension(path) {
                // 用戶指定了完整檔案路徑，直接使用
                Ok(path.to_path_buf())
            } else {
                // 用戶指定了目錄，附加預設檔名
                Ok(path.join(get_default_filename(format)))
            }
        }
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
