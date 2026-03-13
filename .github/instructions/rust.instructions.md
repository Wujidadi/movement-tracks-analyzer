---
applyTo: "src/**,tests/**,Cargo.toml"
---

# Rust 開發規範

本文件說明本專案的架構模式、修改入口、程式風格與測試規範，供 AI 處理 Rust 開發需求時參照。

## 技術棧

- **語言**：Rust 2024 Edition
- **命令行框架**：Clap 4.5（derive 宏風格）
- **XML 解析**：quick-xml 0.39（流式解析，狀態機模式）
- **序列化**：serde + serde_json
- **時間處理**：chrono 0.4
- **正規表示**：regex 1.12
- **編譯方式**：`cargo build --release` 產生最終執行檔

## 架構地圖

```
.
├── src/
│   ├── lib.rs                     # 函式庫根，導出公開 API（49 行）
│   ├── main.rs                    # CLI 主程序（26 行）- run() 模式入口，搭配自訂 Result 型態
│   ├── cli.rs                     # 命令行參數定義（Clap derive 宏）
│   ├── config.rs                  # 配置結構體（執行期設定）
│   ├── path_resolver.rs           # 檔案路徑解析，支援預設檔案自動尋找
│   ├── converter.rs               # 參數轉換，將 CLI args 轉為應用配置
│   ├── output.rs                  # 輸出結果（shell / file）與檔案路徑判定
│   ├── error.rs                   # 自訂錯誤類型（AnalyzerError 枚舉 + Result<T> 別名）
│   ├── regex.rs                   # 正規表示式模式集中定義
│   ├── parser.rs                  # 流式 XML 解析（狀態機），避免全檔案載入記憶體
│   ├── path.rs                    # GPS 軌跡路徑提取與分類邏輯
│   ├── metadata.rs                # 軌跡詮釋資料結構（TrackMetadata）
│   └── format.rs                  # 輸出格式化（JSON、CSV、TSV、表格）
└── tests/
    └── fixtures/                  # 測試用 KML 檔案
        └── tracks.kml
```

## 模組設計原則

### 單一職責

- 每個模組應只負責一個功能領域。
- `cli.rs` 負責命令行定義，`converter.rs` 負責轉換邏輯，`config.rs` 負責配置結構，`output.rs` 負責輸出流程。

### 流式解析架構

- `parser.rs` 採用狀態機模式進行流式 XML 解析，避免全檔案載入記憶體。
- 對大型 KML 檔案（數百 MB）的效能至關重要，詳見 `PERFORMANCE.md`。

### 公開 API 與私有實現

- `src/lib.rs` 應清楚界定公開 API，以便外部呼用或測試。
- 內部模組細節無需對外公開。

## 程式風格與慣例

### 命名規則

| 項目     | 慣例                                                        |
| -------- | ----------------------------------------------------------- |
| 模組名   | `snake_case`（如 `path_resolver`）                          |
| 結構體名 | `PascalCase`（如 `Config`、`TrackMetadata`）                |
| 函數名   | `snake_case`（如 `extract_placemarks_with_paths`）          |
| 常數名   | `SCREAMING_SNAKE_CASE`（如 `DEFAULT_FILE_NAME`）            |
| 型別參數 | `PascalCase`，通常用單字母（如 `T`）或特定名稱（如 `Item`） |

### 可見性與文件註解

- 使用 `pub` 標記公開項目，`pub(crate)` 標記模組內部公開。
- 公開 API 應附上文件註解（`///`）說明功能、參數、回傳值與可能的錯誤。
- 複雜邏輯應在關鍵步驟加上 `//` 行內註解，解釋「為什麼」而非「做什麼」。

### 錯誤處理

- 使用 `Result<T, E>` 作為函數回傳型別，避免 `unwrap()` 或 `panic!()`。
- `src/error.rs` 中定義專案級別的錯誤類型 `AnalyzerError`，手動實作 `Display`、`Error`、`From` traits，搭配 `Result<T>` 型態別名。
- 錯誤訊息應清楚、可操作，協助使用者理解發生的問題。

## 修改入口

### 添加新的命令行參數

1. 編輯 `src/cli.rs`，在 `Args` 結構體中添加欄位
2. 在 `src/converter.rs` 中更新 `build_config()` 函數
3. 若需驗證或預處理參數，編輯 `src/config.rs`

### 修改 KML 解析邏輯

1. 若修改狀態機流程，編輯 `src/parser.rs`
2. 若修改軌跡計算（距離、時間等），編輯 `src/metadata.rs`
3. 若修改路徑提取與分類邏輯，編輯 `src/path.rs`
4. 若修改資料結構，更新 `src/metadata.rs`
5. 使用 `tests/fixtures/tracks.kml` 驗證

### 添加新的輸出格式

1. 在 `src/format.rs` 中的 `OutputFormat` 枚舉添加新變體，並添加格式化函數
2. 在 `src/cli.rs` 中的 `OutputFormatArg` 枚舉添加對應的 CLI 變體
3. 在 `src/converter.rs` 中映射新格式（`OutputFormatArg` → `OutputFormat`）
4. 編寫測試驗證輸出

### 改進效能

1. 閱讀 `PERFORMANCE.md` 了解既有的優化方案
2. 若涉及記憶體或執行速度改進，應先評估影響範圍
3. 效能改動後執行 `cargo build --release` 與效能基準測試

## 測試規範

### 單元測試

單元測試應直接包含在模組中，使用 `#[cfg(test)]` 區塊：

```rust
// src/path.rs

pub fn extract_categories(folder_path: &[String]) -> (String, String, String, String) {
    // ... 實作
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_categories_empty_path() {
        let path: Vec<String> = vec![];
        let (cat, act, year, month) = extract_categories(&path);
        assert_eq!((cat.as_str(), act.as_str(), year.as_str(), month.as_str()), ("", "", "", ""));
    }

    #[test]
    fn test_extract_categories_full_path() {
        let path = vec![
            "移動軌跡".to_string(), "戶外運動".to_string(),
            "步行".to_string(), "2026".to_string(), "2026-03".to_string(),
        ];
        let (cat, act, year, month) = extract_categories(&path);
        assert_eq!(cat, "戶外運動");
        assert_eq!(month, "2026-03");
    }
}
```

### 集成測試

集成測試應放在 `tests/` 目錄下的獨立檔案：

```rust
// tests/kml_parsing.rs

use movement_tracks_analyzer::extract_placemarks_with_paths;
use std::path::PathBuf;

#[test]
fn test_parse_sample_kml() {
    let path = PathBuf::from("tests/fixtures/tracks.kml");
    let result = extract_placemarks_with_paths(&path);
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(!data.is_empty());
}
```

### 執行測試

```bash
# 執行所有測試
cargo test

# 執行單個測試函數
cargo test test_calculate_distance

# 執行單個檔案的所有測試
cargo test --test kml_parsing

# 執行附加詳細輸出
cargo test -- --nocapture
```

## 依賴管理

- 依賴定義於 `Cargo.toml`，應使用穩定版本且定期檢查安全性更新。
- 新增依賴前應評估其大小、維護狀態與安全性。
- 若改動依賴，應在 commit 訊息中說明原因與影響。

## 檔案編碼與路徑

- 原始碼應使用 UTF-8 編碼。
- 路徑處理應使用 `std::path::Path` 與 `PathBuf`，支援跨平台相容性。
- 測試檔案應清楚標記編碼，例如 KML 檔案的 XML 宣告。

## 文件與註解

- 公開函數應包含文件註解，說明用途、參數、回傳值。
- 複雜的演算法應添加邏輯註解，解釋實現方式。
- 若模組改動涉及架構層面，應更新 `ARCHITECTURE.md`。
- 若涉及效能相關改動，應更新 `PERFORMANCE.md`。
- 若涉及重構相關改動，應更新 `REFACTORING.md`。

## 最佳實踐

- **遵守 Rust 慣例**：優先沿用專案既有寫法與型別宣告。
- **編譯檢查**：修改前後執行 `cargo check` 驗證無編譯錯誤。
- **測試驅動**：新功能應附上對應的單元與集成測試。
- **避免 panic**：除非在不可恢復的情況，應使用 `Result` 傳遞錯誤。
- **記憶體安全**：Rust 的型別系統會在編譯期確保記憶體安全，不應繞過此機制。
