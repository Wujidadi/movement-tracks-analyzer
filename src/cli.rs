use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// 使用說明訊息模板
pub const HELP_TEMPLATE: &str = r#"Parse KML GPS tracks and generate analysis reports

Usage: {usage}

Options:
{options}
"#;

#[derive(Parser, Debug)]
#[command(name = "Movement Tracks Analyzer")]
#[command(about = "Parse KML GPS tracks and generate analysis reports", long_about = None)]
#[command(help_template = HELP_TEMPLATE)]
#[command(override_usage = "movement_tracks_analyzer [OPTIONS]")]
pub struct Args {
    // KML 檔案路徑（優先級：命令行參數 > 執行檔目錄 > 當前目錄）
    /// KML file path (priority: command line > executable directory > current directory)
    #[arg(short, long, value_name = "PATH")]
    pub file: Option<PathBuf>,

    // 輸出目標：shell（命令行）或 file（檔案），預設為 file
    /// Output target
    #[arg(short = 'o', long, default_value = "file", value_name = "OUTPUT")]
    pub output: OutputTypeArg,

    // 輸出格式：json、csv、tsv 或 table，預設為 csv
    /// Output format
    #[arg(short = 'm', long, default_value = "csv", value_name = "FORMAT")]
    pub format: OutputFormatArg,

    // 輸出檔案路徑（支持目錄或完整檔案路徑，預設為當前目錄）
    /// Output file path
    #[arg(short = 'x', long, value_name = "PATH")]
    pub export: Option<PathBuf>,
}

/// 輸出目標枚舉
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputTypeArg {
    /// 命令行輸出
    Shell,
    /// 輸出到檔案
    File,
}

/// 輸出格式枚舉
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormatArg {
    /// JSON 格式
    Json,
    /// CSV 格式
    Csv,
    /// TSV 格式
    Tsv,
    /// 表格格式（命令行）
    Table,
}
