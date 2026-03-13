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

/// GPS 軌跡分析工具
///
/// 用於解析 KML 檔案、提取軌跡資訊並輸出為多種格式。
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// KML 檔案路徑
    ///
    /// 若未提供，將尋找預設檔案：
    /// 1. 移動軌跡.kml
    /// 2. Movement Tracks.kml
    #[arg(short, long, value_name = "PATH")]
    pub file: Option<String>,

    /// 輸出類型
    #[arg(short, long, value_enum, default_value = "file")]
    pub output: OutputMode,

    /// 輸出格式
    #[arg(short, long, value_enum, default_value = "csv")]
    pub format: OutputFormat,

    /// 輸出檔案路徑或目錄
    ///
    /// 若為目錄，自動生成檔案名稱
    /// 若為檔案路徑，直接使用
    #[arg(short, long, value_name = "PATH")]
    pub export: Option<String>,

    /// 顯示進度資訊
    #[arg(short, long)]
    pub verbose: bool,
}

/// 輸出模式
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputMode {
    /// 輸出到命令行
    #[value(name = "shell")]
    Shell,
    /// 輸出到檔案
    #[value(name = "file")]
    File,
}

/// 輸出格式
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// JSON 格式
    #[value(name = "json")]
    Json,
    /// CSV 格式
    #[value(name = "csv")]
    Csv,
    /// TSV 格式
    #[value(name = "tsv")]
    Tsv,
    /// 表格格式
    #[value(name = "table")]
    Table,
}

impl Args {
    /// 解析命令行參數
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
```

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

use crate::cli::{Args, OutputFormat, OutputMode};
use crate::config::Config;
use crate::error::ApplicationError;

/// 將命令行參數轉換為應用配置
pub fn build_config(args: Args) -> Result<Config, ApplicationError> {
    let kml_file = resolve_kml_path(&args.file)?;

    let export_path = match (&args.output, &args.export) {
        (OutputMode::Shell, Some(path)) => {
            return Err(ApplicationError::InvalidArgument(
                "shell 模式不支援 --export 參數".to_string(),
            ));
        }
        (OutputMode::File, path) => path.clone(),
        _ => None,
    };

    Ok(Config {
        kml_file,
        output_mode: args.output,
        output_format: args.format,
        export_path,
        verbose: args.verbose,
    })
}

/// 解析 KML 檔案路徑
fn resolve_kml_path(provided: &Option<String>) -> Result<String, ApplicationError> {
    if let Some(path) = provided {
        return Ok(path.clone());
    }

    // 尋找預設檔案
    let defaults = ["移動軌跡.kml", "Movement Tracks.kml"];
    for default_file in &defaults {
        if std::path::Path::new(default_file).exists() {
            return Ok(default_file.to_string());
        }
    }

    Err(ApplicationError::FileNotFound(
        "未找到 KML 檔案。請使用 -f 指定檔案路徑或確保 '移動軌跡.kml' 或 'Movement Tracks.kml' 存在於當前目錄。".to_string(),
    ))
}
```

---

## 配置結構體（config.rs）

### 設計原則

配置應集中於 `config.rs`，包含驗證邏輯：

```rust
// src/config.rs

use crate::cli::{OutputFormat, OutputMode};
use std::path::{Path, PathBuf};

/// 應用配置
#[derive(Debug, Clone)]
pub struct Config {
    /// KML 檔案路徑（已驗證存在）
    pub kml_file: String,

    /// 輸出模式
    pub output_mode: OutputMode,

    /// 輸出格式
    pub output_format: OutputFormat,

    /// 輸出檔案路徑（可選）
    pub export_path: Option<String>,

    /// 冗長模式
    pub verbose: bool,
}

impl Config {
    /// 取得完整的輸出檔案路徑
    pub fn get_output_path(&self, default_name: &str) -> Result<PathBuf, String> {
        match &self.export_path {
            None => Ok(PathBuf::from(default_name)),
            Some(path) => {
                let p = Path::new(path);
                if p.is_dir() {
                    Ok(p.join(default_name))
                } else {
                    Ok(p.to_path_buf())
                }
            }
        }
    }

    /// 驗證配置的合法性
    pub fn validate(&self) -> Result<(), String> {
        if !Path::new(&self.kml_file).exists() {
            return Err(format!("KML 檔案不存在: {}", self.kml_file));
        }
        Ok(())
    }
}
```

---

## 主程式入口（main.rs）

### 清潔的設計

```rust
// src/main.rs

use movement_tracks_analyzer::{
    cli, converter, output,
    parser::extract_placemarks_with_paths,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 解析命令行參數
    let args = cli::Args::parse();

    // 轉換為配置
    let config = converter::build_config(args)?;

    // 驗證配置
    config.validate()?;

    // 提取軌跡資料
    let placemarks = extract_placemarks_with_paths(&config.kml_file)?;

    // 輸出結果
    output::output_results(&placemarks, &config)?;

    Ok(())
}
```

**特點**：清潔、簡短、易讀

---

## 使用者互動與驗證

### 友善的錯誤訊息

```rust
// src/main.rs

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Args::parse();
    let config = converter::build_config(args)?;

    match config.validate() {
        Ok(_) => {
            // 繼續處理
        }
        Err(e) => {
            eprintln!("❌ 設定錯誤: {}", e);
            eprintln!("\n使用 '--help' 獲取使用說明。");
            std::process::exit(1);
        }
    }

    // ...後續邏輯
    Ok(())
}
```

### 進度提示

當 `--verbose` 標誌被設定時：

```rust
if config.verbose {
eprintln!("📂 讀取檔案: {}", config.kml_file);
eprintln!("⚙️  輸出格式: {:?}", config.output_format);
}
```

---

## 命令行範例與使用案例

### 基本用法

```bash
# 預設行為：尋找 KML 檔案、輸出到 CSV
./movement_tracks_analyzer

# 指定檔案，輸出為 JSON
./movement_tracks_analyzer -f my_tracks.kml -m json -o shell

# 指定檔案與輸出路徑
./movement_tracks_analyzer --file tracks.kml --format table --export output.txt

# 詳細模式
./movement_tracks_analyzer --verbose
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
pub fn build_config(args: Args) -> Result<Config, ApplicationError> {
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
        assert_eq!(format!("{:?}", args.format), "Csv");
    }

    #[test]
    fn test_custom_file_path() {
        let args = Args::try_parse_from(vec!["prog", "-f", "custom.kml"]).unwrap();
        assert_eq!(args.file, Some("custom.kml".to_string()));
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
