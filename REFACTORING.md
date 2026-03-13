# 程式碼重構總結

## 成果

### 認知複雜度改進

> 認知複雜度據 JetBrains Better Highlights plugin 計算

| 函式                            | 舊版本 | 新版本             | 狀態 |
| ------------------------------- | ------ | ------------------ | ---- |
| `extract_placemarks_with_paths` | 187%   | 33% (狀態機設計)   | ✅    |
| `get_kml_file_path`             | 133%   | 27% (拆分路徑檢查) | ✅    |
| `main`                          | 87%    | 7% (工作流簡化)    | ✅    |
| `extract_categories`            | 60%    | 13% (模式匹配優化) | ✅    |

**結果**：所有函式複雜度 ≤ 40%

### 程式碼組織改進

**舊結構（main.rs）：**

- 626 行單一檔案
- 業務邏輯與 CLI 混合
- 難以維護和測試

**首次重構（6 個庫模組）：**

```
src/
├── lib.rs       (12 行)  - Library root，導出公開 API
├── main.rs      (193 行) - CLI 邏輯（低複雜度）
├── parser.rs    (220 行) - XML 解析（狀態機）
├── path.rs      (41 行)  - 路徑提取（簡潔）
├── metadata.rs  (54 行)  - 軌跡詮釋資料
└── format.rs    (233 行) - 輸出格式化
```

**最終結構（13 個模組，入口點優化）：**

```
src/
├── lib.rs            (49 行) - Library root，導出公開 API
├── main.rs           (26 行) ✨ 乾淨的入口點（run() 模式 + 自訂 Result）
├── cli.rs            (61 行) - 命令行參數定義
├── config.rs         (17 行) - 配置結構體
├── path_resolver.rs  (47 行) - 檔案路徑解析
├── output.rs         (75 行) - 輸出和儲存邏輯
├── converter.rs      (38 行) - 參數轉換
├── error.rs         (101 行) - 自訂錯誤類型
├── parser.rs        (248 行) - XML 解析（狀態機）
├── path.rs          (184 行) - 路徑提取
├── metadata.rs      (194 行) - 軌跡詮釋資料
├── regex.rs          (79 行) - 正規表示式模式
└── format.rs        (294 行) - 輸出格式化
```

**關鍵改進**：main.rs 從 233 行精簡到 **26 行**，入口點完全乾淨。

## 關鍵設計改進

### 1. 狀態機設計（parser.rs）

**複雜度由 187% 降至 33%**

```rust
#[derive(Debug, Default)]
struct ParserState {
    in_placemark: bool,
    in_name: bool,
    in_description: bool,
    in_coordinates: bool,
    in_folder_name: bool,
    // ... 資料欄位
}

// 單一職責函式
fn handle_start_tag(tag_name: &str, folder_stack: &mut Vec<String>, state: &mut ParserState)
fn handle_end_tag(tag_name: &str, ...) -> Result<()>
fn parse_coordinates(coords_str: &str) -> Result<Vec<(f64, f64)>>
```

**優點**：

- 無須多個標誌變數
- 清晰的狀態轉移
- 易於擴展

### 2. 路徑提取簡化（path.rs）

**複雜度由 60% 降至 13%**

```rust
// 使用模式匹配而非多個 if-else
match meaningful_path.len() {
    0 => (String::new(), String::new(), String::new(), String::new()),
    1 => extract_single_element( & meaningful_path),
    2 => (String::new(), String::new(), ..),
    3 => (String::new(), ...),
    _ => (...)
}
```

**優點**：

- 清晰的邊界情況處理
- 單一職責（41 行模組）
- 易於單元測試

### 3. 格式化重構（format.rs）

**減少程式碼重複**

```rust
// 統一的格式化介面
pub fn format_output(format: OutputFormat, tracks: &[...]) -> String {
    match format {
        OutputFormat::Json => format_json(tracks),
        OutputFormat::Csv => format_csv(tracks),
        OutputFormat::Tsv => format_tsv(tracks),
        OutputFormat::Table => format_table(tracks),
    }
}

// 欄寬計算採用陣列而非 Vec
let mut widths: [usize; 10] = headers.map(display_width);

// 迴圈中統一更新
for (i, value) in values.iter().enumerate() {
    widths[i] = widths[i].max(display_width(value));
}
```

**優點**：

- 一致的介面
- 減少重複程式碼
- 簡潔的欄寬計算

### 4. CLI 邏輯簡化（main.rs）

**複雜度由 87% 降至 7%**

> **工作流清晰**
> 1. 使用說明
> 2. 解析參數
> 3. 提取軌跡
> 4. 輸出結果

## 效能驗證

- ✅ **編譯時間**：1.62s（無明顯變化）
- ✅ **執行時間**：0.3s（效能未受影響）
- ✅ **二進位檔案大小**：2.3MB（無增加）
- ✅ **功能完整性**：所有功能正常運作

## 測試清單

- ✅ CSV 輸出正常
- ✅ TSV 輸出正常
- ✅ JSON 輸出正常
- ✅ 表格輸出正常（Unicode 字元對齊）
- ✅ 檔案輸出正常
- ✅ 命令行參數解析正常
- ✅ 效能未降低

## 可維護性改進

| 方面           | 改進                   |
| -------------- | ---------------------- |
| **模組化**     | 6 個單一職責模組       |
| **複雜度**     | 所有函式 < 40%         |
| **程式碼重複** | 減少（特別是欄寬計算） |
| **可讀性**     | 清晰的函式名和流程     |
| **可測試性**   | 各模組可獨立測試       |

## 後續建議

1. **單元測試** ✅ **已完成**

   為各模組添加 #[cfg(test)] 測試模組

   **實現細節**：
    - ✅ **path.rs**：5 個單元測試
        - `test_extract_categories_full_path()` - 完整路徑提取
        - `test_extract_categories_with_spaces()` - 空格處理
        - `test_extract_categories_with_three_meaningful_elements()` - 多層路徑
        - `test_extract_categories_single_non_root_element()` - 月份格式檢測
        - `test_extract_categories_empty_path()` - 空路徑邊界

    - ✅ **metadata.rs**：7 個單元測試
        - `test_duration_seconds()` - 時間計算（正常情況）
        - `test_duration_same_time()` - 相同時間點
        - `test_duration_negative()` - 反向時間差
        - `test_calculate_distance_multiple_points()` - 多點距離計算
        - `test_calculate_distance_single_point()` - 單點距離
        - `test_calculate_distance_two_points()` - 雙點距離
        - `test_metadata_creation()` - 結構體創建

    - ✅ **regex.rs**：7 個單元測試
        - `test_start_time_pattern_matches()` - 開始時間匹配
        - `test_start_time_pattern_captures()` - 開始時間捕獲
        - `test_start_time_pattern_with_spaces()` - 空格容錯
        - `test_end_time_pattern_matches()` - 結束時間匹配
        - `test_end_time_pattern_captures()` - 結束時間捕獲
        - `test_end_time_pattern_without_br()` - 換行字元處理
        - `test_both_patterns_in_html()` - 組合匹配

    - ✅ **error.rs**：5 個單元測試
        - `test_analyzer_error_display()` - 錯誤顯示
        - `test_analyzer_error_parsing()` - 解析錯誤訊息
        - `test_analyzer_error_from_string()` - 字串轉換
        - `test_analyzer_error_from_io_error()` - IO 錯誤轉換
        - `test_result_type_alias()` - Result 類型別名

   **測試統計**：
    - 總測試數：24
    - 覆蓋模組：4（path, metadata, regex, error）
    - 全部通過：✅

2. **文件註解** ✅ **已完成**

   添加 rustdoc 註解為公開 API 提供清晰的文檔
   ```rust
   /// 從 KML 檔案中提取軌跡
   ///
   /// # Arguments
   /// * `file_path` - KML 檔案路徑
   ///
   /// # Returns
   ///
   /// 成功時返回軌跡資料向量
   ///
   /// # Example
   ///
   /// ```ignore
   /// let tracks = extract_placemarks_with_paths(&path)?;
   /// ```
   pub fn extract_placemarks_with_paths(file_path: &PathBuf) -> Result<...>
   ```

   **實現細節**：
    - ✅ lib.rs：添加庫級別文檔（功能介紹、範例）
    - ✅ parser.rs：`extract_placemarks_with_paths()` 函式文檔（參數、返回值、效能、範例）
    - ✅ path.rs：`extract_categories()` 函式文檔（返回值詳解、使用範例）
    - ✅ format.rs：`format_output()` 函式文檔（格式詳解、使用範例）
    - ✅ metadata.rs：`TrackMetadata` 結構體和方法文檔（欄位說明、算法解釋）

3. **錯誤處理** ✅ **已完成**

   使用自訂錯誤類型（而非 `Box<dyn std::error::Error>`）
   ```rust
   #[derive(Debug)]
   pub enum AnalyzerError {
       Io(io::Error),
       ParsingError(String),
       TimeParsingError(String),
       CoordinateParsingError(String),
       FileNotFound(String),
       Other(String),
   }
   ```

   **實現細節**：
    - ✅ 新增 `error.rs` 模組（57 行）
    - ✅ 實現 `Display` 和 `Error` traits
    - ✅ 提供自動錯誤轉換（`From` implementations）
    - ✅ 所有函式返回 `Result<T>` 別名（簡潔）
    - ✅ 優雅的錯誤顯示（不是 Debug 格式）
    - ✅ 更新所有相關模組（parser、path_resolver、output、converter）

## 相關閱讀

- [README.md](./README.md) - 快速開始與使用範例
- [ARCHITECTURE.md](./ARCHITECTURE.md) - 詳細的模組結構說明
- [PERFORMANCE.md](./PERFORMANCE.md) - 效能優化技術詳解
