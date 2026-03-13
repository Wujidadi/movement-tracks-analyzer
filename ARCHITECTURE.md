# 專案結構導覽

## 📁 專案佈局

```
movement-tracks-analyzer/
├── Cargo.toml                        # 專案配置（8 個依賴）
├── Cargo.lock                        # 版本鎖定
├── README.md                         # 使用指南
├── PERFORMANCE.md                    # 效能優化說明
├── REFACTORING.md                    # 程式碼重構總結
├── ARCHITECTURE.md                   # 本文檔
├── AGENTS.md                         # Agent 操作指引（→ .github/AGENTS.md 軟連結）
├── .github/
│   ├── copilot-instructions.md       # AI 協作入口
│   ├── AGENTS.md                     # 全域操作規範
│   ├── instructions/
│   │   └── rust.instructions.md      # Rust 開發規範
│   └── skills/                       # 任務技能模組
│       ├── testing/SKILL.md
│       ├── cli-development/SKILL.md
│       └── kml-parsing/SKILL.md
├── src/
│   ├── lib.rs                        # Library root，導出公開 API
│   ├── main.rs                       # CLI 主程式（26 行）
│   ├── cli.rs                        # 命令行參數定義
│   ├── config.rs                     # 配置結構體
│   ├── path_resolver.rs              # 檔案路徑解析
│   ├── output.rs                     # 輸出和儲存邏輯
│   ├── converter.rs                  # 參數轉換
│   ├── error.rs                      # 自訂錯誤類型
│   ├── regex.rs                      # 正規表示式模式
│   ├── parser.rs                     # XML 流式解析（狀態機）
│   ├── path.rs                       # 路徑提取邏輯
│   ├── metadata.rs                   # 軌跡詮釋資料結構
│   └── format.rs                     # 輸出格式化
├── tests/
│   └── fixtures/
│       └── tracks.kml                # 測試用 KML 檔案
└── target/
    └── release/
        └── movement_tracks_analyzer  # 編譯的可執行檔
```

## 🔧 模組詳解

### 🎯 `main.rs` (26 行) - 核心入口

```rust
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

- **職責**：清潔的程式入口，流程一目瞭然
- **特點**：採用 `run()` 函式模式搭配自訂 `Result` 型態，錯誤集中由 `main()` 處理

### `cli.rs` (61 行)

- **職責**：命令行參數定義和輸出使用說明
- **內容**：
    - `HELP_TEMPLATE` - 使用說明模板
    - `Args` 結構體 - 參數定義
    - `OutputTypeArg` 枚舉 - 輸出目標
    - `OutputFormatArg` 枚舉 - 輸出格式

### `config.rs` (17 行)

- **職責**：配置結構體定義
- **內容**：
    - `Config` 結構體
    - `OutputType` 枚舉

### `path_resolver.rs` (47 行)

- **職責**：KML 檔案路徑解析
- **函式**：
    - `resolve_kml_file()` - 主函式
    - `check_exe_directory()` - 檢查執行檔目錄
    - `check_current_directory()` - 檢查當前目錄
    - `check_path_with_filenames()` - 工具函式

### `output.rs` (75 行)

- **職責**：結果輸出和檔案儲存
- **函式**：
    - `output_results()` - 輸出主函式
    - `save_to_file()` - 儲存到檔案
    - `determine_file_path()` - 確定輸出路徑
    - `has_file_extension()` - 判斷路徑是否含副檔名
    - `get_default_filename()` - 取得預設檔名

### `converter.rs` (38 行)

- **職責**：命令行參數轉換
- **函式**：
    - `build_config()` - 參數到配置的轉換

### `error.rs` (101 行)

- **職責**：自訂錯誤類型定義
- **內容**：
    - `AnalyzerError` 枚舉 - 6 種錯誤類型
    - `Display` trait 實現 - 用戶友好的錯誤訊息
    - `Error` trait 實現 - 標準錯誤接口
    - `From` implementations - 自動錯誤轉換
    - `Result<T>` 型態別名 - 便捷的 Result 類型

### `regex.rs` (79 行)

- **職責**：正規表示式模式定義
- **內容**：
    - `DATETIME_PATTERN` - 日期時間格式常數
    - `START_TIME_PATTERN` - 提取開始時間的正規表達式
    - `END_TIME_PATTERN` - 提取結束時間的正規表達式
    - `create_time_pattern()` - 參數化生成時間模式的函式

### `lib.rs` (49 行)

- **職責**：Library root，導出公開 API
- **公開模組**：`error`、`format`、`metadata`、`parser`、`path`、`regex`
- **導出**：`AnalyzerError`、`Result`、`OutputFormat`、`format_output`、`TrackMetadata`、`extract_placemarks_with_paths`、`extract_categories`、`START_TIME_PATTERN`、`END_TIME_PATTERN`
- **用途**：允許將此專案作為庫使用

### `parser.rs` (248 行) - 認知複雜度 30%

- **職責**：KML 流式解析
- **設計**：狀態機模式
- **核心函式**：
    - `extract_placemarks_with_paths()` - 主解析函式
    - `handle_start_tag()` - 開始標籤處理
    - `handle_end_tag()` - 結束標籤處理
    - `parse_coordinates()` - 座標解析
    - `extract_times()` - 時間提取

### `path.rs` (184 行) - 認知複雜度 15%

- **職責**：路徑提取和分類
- **函式**：
    - `extract_categories()` - 提取分類/活動/年份/月份
    - `create_category_tuple()` - 構建分類元組
    - `empty_tuple()` - 返回空分類元組
    - `extract_single_element()` - 單一元素路徑處理

### `metadata.rs` (194 行)

- **職責**：軌跡資料結構和計算
- **結構**：`TrackMetadata`
- **方法**：
    - `calculate_distance()` - 半正矢（Haversine）公式
    - `duration_seconds()` - 時間計算

### `format.rs` (294 行)

- **職責**：輸出格式化
- **支援格式**：JSON、CSV、TSV、Table（命令行表格）
- **函式**：
    - `format_output()` - 統一格式化介面
    - `format_json()`
    - `format_csv()`
    - `format_tsv()`
    - `format_table()`
    - `calculate_column_widths()` - 欄寬計算
    - `format_cell()` - 單元格格式化（支援 Unicode）
    - `format_row_data()` - 格式化軌跡資料為字串陣列

## 🚀 資料流

```
main() → run()
  ↓
Args::parse() [cli.rs]
  ↓
build_config() [converter.rs]
  ├─→ resolve_kml_file() [path_resolver.rs]
  └─→ Config [config.rs]
  ↓
extract_placemarks_with_paths() [parser.rs]
  ├─→ XML 事件迴圈
  ├─→ handle_start_tag()
  ├─→ handle_end_tag()
  ├─→ extract_categories() [path.rs]
  └─→ TrackMetadata 結構化 [metadata.rs]
  ↓
format_output() [format.rs]
  ├─→ format_json()
  ├─→ format_csv()
  ├─→ format_tsv()
  └─→ format_table()
  ↓
output_results() [output.rs]
  ├─→ 命令行輸出 (Shell)
  └─→ save_to_file() (File)
```

## 📊 統計數據

| 指標               | 數值                    |
| ------------------ | ----------------------- |
| **程式碼總行數**   | 1,413                   |
| **模組數量**       | 13                      |
| **認知複雜度最高** | 30% (< 40% ✅)           |
| **依賴數量**       | 8                       |
| **編譯時間**       | 1.19s                   |
| **二進位檔案大小** | 2.3MB                   |
| **測試數據**       | 48MB KML / 2,164 個軌跡 |

## 🔑 關鍵技術

- **流式 XML 解析**：`quick-xml` (0.39)
- **日期時間**：`chrono` (0.4)
- **正規表示式**：`regex` (1.12)
- **Unicode 寬度**：`unicode-width` (0.2)
- **惰性靜態初始化**：`once_cell` (1.21)

## 💡 設計亮點

1. **狀態機設計**：避免多層嵌套，複雜度降 87%
2. **單一職責**：每個模組責任清晰
3. **無程式碼重複**：統一的格式化介面和欄寬計算
4. **流式處理**：只掃描 1 次大檔案
5. **模組化**：可作為庫使用

## 🧪 驗證清單

- ✅ 編譯成功（無警告）
- ✅ 所有 4 種輸出格式正常
- ✅ CLI 參數解析正常
- ✅ 效能未降低
- ✅ 認知複雜度 < 40%
- ✅ 程式碼重複減少
- ✅ 可維護性改進

## 📚 相關文檔

- [README.md](./README.md) - 使用指南與快速開始
- [PERFORMANCE.md](./PERFORMANCE.md) - 效能優化詳細說明
- [REFACTORING.md](./REFACTORING.md) - 重構詳細報告與設計改進
