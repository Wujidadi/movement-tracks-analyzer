use std::{error::Error, path::PathBuf};

/// 解析 KML 檔案路徑
pub fn resolve_kml_file(cli_file: Option<PathBuf>) -> Result<PathBuf, Box<dyn Error>> {
    // 檢查命令行參數
    if let Some(path) = cli_file {
        if path.exists() {
            println!("Using KML file from command line: {}", path.display());
            return Ok(path);
        }
        return Err(format!("KML file not found: {}", path.display()).into());
    }

    // 檢查執行檔所在目錄
    if let Some(path) = check_exe_directory() {
        return Ok(path);
    }

    // 檢查當前工作目錄
    check_current_directory()
}

/// 檢查指定目錄是否存在 KML 檔案
fn check_path_with_filenames(base_path: &std::path::Path, filenames: &[&str]) -> Option<PathBuf> {
    filenames
        .iter()
        .map(|filename| base_path.join(filename))
        .find(|path| path.exists())
}

/// 檢查執行檔所在目錄是否存在 KML 檔案
fn check_exe_directory() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    check_path_with_filenames(exe_dir, &["移動軌跡.kml", "Movement Tracks.kml"])
        .inspect(|path| println!("Using default KML file: {}", path.display()))
}

/// 檢查當前工作目錄是否存在 KML 檔案
fn check_current_directory() -> Result<PathBuf, Box<dyn Error>> {
    let current_dir = std::env::current_dir()?;
    check_path_with_filenames(&current_dir, &["移動軌跡.kml", "Movement Tracks.kml"])
        .inspect(|path| println!("Using default KML file: {}", path.display()))
        .ok_or_else(|| "No KML file found. Please specify with -f / --file or place 移動軌跡.kml or Movement Tracks.kml in the current directory.".into())
}
