---
name: cli-development
description: 本專案的命令行界面開發指南。說明如何使用 Clap derive 宏定義參數、實現命令行解析與使用者互動、支援各種輸出選項與檔案處理。當使用者涉及命令行參數、使用者帳戶、參數驗證或命令行工作流改進時參照。
---

# 命令行界面（CLI）開發指南

## 技術棧

| 工具   | 版本 | 用途                                   |
| ------ | ---- | -------------------------------------- |
| Clap   | 4.5  | **命令行參數解析**；使用 derive 宏風格 |
| serde  | 1.0  | 序列化/反序列化結構體與命令行參數對映  |
| chrono | 0.4  | 時間處理與格式化，支援日期時間輸出     |

---

## 命令行架構

### 檔案結構

```
src/
├── cli.rs            # ← 命令行參數定義（Clap derive）
├── converter.rs      # ← 參數轉換與驗證
├── config.rs         # ← 應用配置結構體
├── main.rs           # ← 清潔的程式入口
└── output.rs         # ← 輸出結果到控制台或檔案
```

### 資料流

```
命令行參數
  ↓
cli::Args (Clap derive)
  ↓
converter::build_config() ← 轉換與驗證
  ↓
config::Config (應用配置)
  ↓
核心邏輯處理
  ↓
output::output_results() ← 格式化與輸出
```

---

## 定義命令行參數（cli.rs）

### 結構體定義範例

```rust
// src/cli.rs

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
```

> **注意**：CLI 層的枚舉命名為 `OutputTypeArg` / `OutputFormatArg`，與函式庫 crate 的 `OutputType` / `OutputFormat` 區隔，避免名稱衝突。

### Clap Derive 宏常用屬性

| 屬性              | 說明                                       |
| ----------------- | ------------------------------------------ |
| `#[command(...)]` | 應用級設定（作者、版本、說明等）           |
| `#[arg(...)]`     | 欄位級設定（短名、長名、預設值、值列舉等） |
| `#[value(name)]`  | 列舉變體的命令行名稱，可與實際名稱不同     |
| `short`           | 短參數名（如 `-f`）                        |
| `long`            | 長參數名（如 `--file`）                    |
| `default_value`   | 參數預設值                                 |
| `value_enum`      | 列舉值須實現 `ValueEnum` trait             |
| `value_name`      | 在說明中顯示的參數名（如 `<PATH>`）        |
| `help`            | 參數說明（自動用於 `-h` / `--help`）       |

---

## 參數轉換與驗證（converter.rs）

### 轉換函數範例

```rust
// src/converter.rs

use crate::{
    cli::{Args, OutputFormatArg, OutputTypeArg},
    config::{Config, OutputType},
    path_resolver::resolve_kml_file,
};
use movement_tracks_analyzer::{OutputFormat, Result};

/// 從 CLI 參數建立設定
pub fn build_config(args: Args) -> Result<Config> {
    let output_type = match args.output {
        OutputTypeArg::Shell => OutputType::Shell,
        OutputTypeArg::File => OutputType::File,
    };

    let format = match args.format {
        OutputFormatArg::Json => OutputFormat::Json,
        OutputFormatArg::Csv => OutputFormat::Csv,
        OutputFormatArg::Tsv => OutputFormat::Tsv,
        OutputFormatArg::Table => OutputFormat::Table,
    };

    // 當 format="table" 且 output="file" 時，自動使用 csv 格式
    let format = if matches!(args.format, OutputFormatArg::Table)
        && matches!(output_type, OutputType::File)
    {
        OutputFormat::Csv
    } else {
        format
    };

    Ok(Config {
        kml_file: resolve_kml_file(args.file)?,
        output_type,
        format,
        export_path: args.export,
    })
}
```

---

## 配置結構體（config.rs）

### 設計原則

配置應集中於 `config.rs`，結構簡潔：

```rust
// src/config.rs

use movement_tracks_analyzer::OutputFormat;
use std::path::PathBuf;

/// CLI 參數設定
pub struct Config {
    /// KML 檔案路徑（已驗證存在）
    pub kml_file: PathBuf,

    /// 輸出類型
    pub output_type: OutputType,

    /// 輸出格式
    pub format: OutputFormat,

    /// 輸出檔案路徑（可選）
    pub export_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputType {
    Shell,
    File,
}
```

> **注意**：`Config` 引用函式庫 crate 的 `OutputFormat`，而非 CLI 層的 `OutputFormatArg`。轉換邏輯集中在 `converter.rs`。

---

## 主程式入口（main.rs）

### 清潔的設計

```rust
// src/main.rs

mod cli;
mod config;
mod converter;
mod output;
mod path_resolver;

use clap::Parser;
use cli::Args;
use converter::build_config;
use movement_tracks_analyzer::extract_placemarks_with_paths;
use output::output_results;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> movement_tracks_analyzer::Result<()> {
    let args = Args::parse();
    let config = build_config(args)?;
    let placemarks = extract_placemarks_with_paths(&config.kml_file)?;
    output_results(&placemarks, &config)
}
```

**特點**：
- 採用 `run()` 函式模式，將錯誤處理集中在 `main()` 中
- 使用自訂 `Result` 型態（`movement_tracks_analyzer::Result<()>`）
- 二進位 crate 透過 `mod` 宣告引入自己的模組，透過 `use movement_tracks_analyzer::...` 引用函式庫 crate

---

## 使用者互動與錯誤處理

### 錯誤處理模式

專案使用 `run()` 函式模式集中處理錯誤，錯誤由 `eprintln!` 輸出後以非零狀態碼退出：

```rust
// src/main.rs

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

自訂 `AnalyzerError` 提供清晰的錯誤訊息（如 `File not found: /path/to/file.kml`），`From` trait 實作支援 `?` 運算子自動轉換。

### 路徑解析提示

當找到預設 KML 檔案時，程式會輸出提示訊息：

```rust
// src/path_resolver.rs

fn check_exe_directory() -> Option<PathBuf> {
    // ...
    .inspect(|path| println!("Using default KML file: {}", path.display()))
}
```

---

## 命令行範例與使用案例

### 基本用法

```bash
# 預設行為：尋找 KML 檔案、輸出到 CSV
./movement_tracks_analyzer

# 指定檔案，輸出為 JSON 到命令行
./movement_tracks_analyzer -f my_tracks.kml -m json -o shell

# 指定檔案與輸出路徑
./movement_tracks_analyzer --file tracks.kml --format csv --export /tmp/output.csv

# 輸出表格到命令行
./movement_tracks_analyzer -f tracks.kml -o shell -m table
```

### 顯示說明

```bash
./movement_tracks_analyzer --help
./movement_tracks_analyzer -h
```

---

## 添加新的命令行參數

### 步驟 1：在 cli.rs 中添加欄位

```rust
#[derive(Parser, Debug)]
pub struct Args {
    // ...既有欄位...

    /// 新參數說明
    #[arg(short, long)]
    pub new_param: bool,
}
```

### 步驟 2：在 converter.rs 中處理轉換

```rust
pub fn build_config(args: Args) -> Result<Config> {
    // ...既有邏輯...

    if args.new_param {
        // 處理新參數
    }

    Ok(config)
}
```

### 步驟 3：在 config.rs 中新增對應欄位

```rust
pub struct Config {
    // ...既有欄位...
    pub new_param: bool,
}
```

### 步驟 4：在 main.rs 或業務邏輯中使用

```rust
if config.new_param {
// 使用新參數
}
```

---

## 測試命令行參數解析

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_output_format() {
        let args = Args::try_parse_from(vec!["prog"]).unwrap();
        assert!(matches!(args.format, OutputFormatArg::Csv));
    }

    #[test]
    fn test_custom_file_path() {
        let args = Args::try_parse_from(vec!["prog", "-f", "custom.kml"]).unwrap();
        assert_eq!(args.file, Some(PathBuf::from("custom.kml")));
    }
}
```

---

## 最佳實踐

- **清晰的參數名**：使用有意義的短名與長名
- **完整的說明**：提供詳細的 help 說明，包括預設值與使用範例
- **合理的預設值**：選擇最常見的用例作為預設行為
- **驗證參數**：在 converter 或 config 中集中驗證邏輯
- **友善的錯誤訊息**：明確說明問題與解決方案
- **測試涵蓋**：為參數解析添加單元測試
