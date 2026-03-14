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
├── main.rs           # ← 清潔的程式入口（run() 模式）
└── output.rs         # ← 輸出結果到控制台或檔案
```

### 資料流

```
命令行參數 → cli::Args (Clap derive) → converter::build_config()
→ config::Config (應用配置) → 核心邏輯處理 → output::output_results()
```

---

## 定義命令行參數（cli.rs）

### 結構體定義

```rust
#[derive(Parser, Debug)]
#[command(name = "Movement Tracks Analyzer")]
#[command(about = "Parse KML GPS tracks and generate analysis reports")]
#[command(help_template = HELP_TEMPLATE)]
#[command(override_usage = "movement_tracks_analyzer [OPTIONS]")]
pub struct Args {
    /// KML file path (priority: command line > executable directory > current directory)
    #[arg(short, long, value_name = "PATH")]
    pub file: Option<PathBuf>,

    /// Output target
    #[arg(short = 'o', long, default_value = "file", value_name = "OUTPUT")]
    pub output: OutputTypeArg,

    /// Output format
    #[arg(short = 'm', long, default_value = "csv", value_name = "FORMAT")]
    pub format: OutputFormatArg,

    /// Output file path
    #[arg(short = 'x', long, value_name = "PATH")]
    pub export: Option<PathBuf>,
}
```

> **注意**：CLI 層的枚舉命名為 `OutputTypeArg` / `OutputFormatArg`，與函式庫 crate 的 `OutputType` / `OutputFormat` 區隔，避免名稱衝突。

### Clap Derive 宏常用屬性

| 屬性              | 說明                                         |
| ----------------- | -------------------------------------------- |
| `#[command(...)]` | 應用級設定（作者、版本、說明等）             |
| `#[arg(...)]`     | 欄位級設定（短名、長名、預設值、值列舉等）   |
| `short` / `long`  | 短參數名（如 `-f`）/ 長參數名（如 `--file`） |
| `default_value`   | 參數預設值                                   |
| `value_enum`      | 列舉值須實現 `ValueEnum` trait               |
| `value_name`      | 在說明中顯示的參數名（如 `<PATH>`）          |

---

## 參數轉換與驗證（converter.rs）

`build_config(args: Args) -> Result<Config>` 負責將 CLI 參數轉為應用配置：

- `map_output_type()`：`OutputTypeArg` → `OutputType`
- `map_output_format()`：`OutputFormatArg` → `OutputFormat`
- `resolve_format()`：表格 + 檔案輸出時自動降級為 CSV
- `resolve_kml_file()`：解析 KML 檔案路徑（含預設檔案自動尋找）

---

## 配置結構體（config.rs）

`Config` 結構體包含以下欄位：

- `kml_file: PathBuf` — KML 檔案路徑（已驗證存在）
- `output_type: OutputType` — 輸出類型（Shell / File）
- `format: OutputFormat` — 輸出格式（引用函式庫 crate 的型別，非 CLI 層的 `OutputFormatArg`）
- `export_path: Option<PathBuf>` — 輸出檔案路徑（可選）

> 轉換邏輯集中在 `converter.rs`，`Config` 不直接依賴 CLI 型別。

---

## 主程式入口（main.rs）

採用 `run()` 函式模式，將錯誤處理集中在 `main()` 中：

- 使用自訂 `Result` 型態（`movement_tracks_analyzer::Result<()>`）
- 錯誤由 `eprintln!` 輸出後以非零狀態碼退出
- 二進位 crate 透過 `mod` 宣告引入自己的模組，透過 `use movement_tracks_analyzer::...` 引用函式庫 crate

流程：`Args::parse()` → `build_config()` → `extract_placemarks_with_paths()` → `output_results()`

---

## 路徑解析（path_resolver.rs）

當使用者未指定檔案路徑時，`resolve_kml_file()` 依序嘗試：

1. `移動軌跡.kml` → `Movement Tracks.kml` → `移動軌跡.kmz` → `Movement Tracks.kmz`（KML 優先）
2. 先搜尋執行檔所在目錄，再搜尋當前工作目錄

找到預設檔案時，使用 `.inspect()` 輸出提示訊息。

---

## 使用者互動與錯誤處理

- 自訂 `AnalyzerError` 提供清晰的錯誤訊息（如 `File not found: /path/to/file.kml`）
- `From` trait 實作支援 `?` 運算子自動轉換

---

## 命令行使用範例

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

---

## 添加新的命令行參數

1. **`cli.rs`**：在 `Args` 結構體中新增欄位（`#[arg(...)]`）
2. **`converter.rs`**：在 `build_config()` 中處理新參數的轉換邏輯
3. **`config.rs`**：在 `Config` 結構體中新增對應欄位
4. **業務邏輯**：在 `main.rs` 或對應模組中使用新設定
5. **編寫測試**：參照 [`skills/testing/SKILL.md`](../testing/SKILL.md) 的測試策略

---

## 最佳實踐

- **清晰的參數名**：使用有意義的短名與長名
- **完整的說明**：提供詳細的 help 說明，包括預設值與使用範例
- **合理的預設值**：選擇最常見的用例作為預設行為
- **驗證參數**：在 converter 或 config 中集中驗證邏輯
- **友善的錯誤訊息**：明確說明問題與解決方案
