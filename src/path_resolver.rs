use movement_tracks_analyzer::Result;
use std::path::PathBuf;

/// 預設的 KML/KMZ 檔案名稱（KML 優先）
const DEFAULT_FILENAMES: &[&str] = &[
    "移動軌跡.kml",
    "Movement Tracks.kml",
    "移動軌跡.kmz",
    "Movement Tracks.kmz",
];

/// 解析 KML/KMZ 檔案路徑
pub fn resolve_kml_file(cli_file: Option<PathBuf>) -> Result<PathBuf> {
    // 檢查命令行參數
    if let Some(path) = cli_file {
        if path.exists() {
            println!("Using file from command line: {}", path.display());
            return Ok(path);
        }
        return Err(format!("File not found: {}", path.display()).into());
    }

    // 檢查執行檔所在目錄
    if let Some(path) = check_exe_directory() {
        return Ok(path);
    }

    // 檢查當前工作目錄
    check_current_directory()
}

/// 檢查指定目錄是否存在 KML/KMZ 檔案
fn check_path_with_filenames(base_path: &std::path::Path, filenames: &[&str]) -> Option<PathBuf> {
    filenames
        .iter()
        .map(|filename| base_path.join(filename))
        .find(|path| path.exists())
}

/// 檢查執行檔所在目錄是否存在 KML/KMZ 檔案
fn check_exe_directory() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    check_path_with_filenames(exe_dir, DEFAULT_FILENAMES)
        .inspect(|path| println!("Using default file: {}", path.display()))
}

/// 檢查當前工作目錄是否存在 KML/KMZ 檔案
fn check_current_directory() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    check_path_with_filenames(&current_dir, DEFAULT_FILENAMES)
        .inspect(|path| println!("Using default file: {}", path.display()))
        .ok_or_else(|| "No KML/KMZ file found. Please specify with -f / --file or place 移動軌跡.kml (.kmz) or Movement Tracks.kml (.kmz) in the current directory.".into())
}
