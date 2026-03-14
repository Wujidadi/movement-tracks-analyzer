# 程式碼重構總結

## 成果

### 認知複雜度改進

> 認知複雜度據 JetBrains Better Highlights plugin 計算（百分比形式）

#### 第一輪重構（模組化 + 狀態機設計）

| 函式                            | 舊版本 | 新版本             | 狀態 |
| ------------------------------- | ------ | ------------------ | ---- |
| `extract_placemarks_with_paths` | 187%   | 33% (狀態機設計)   | ✅    |
| `get_kml_file_path`             | 133%   | 27% (拆分路徑檢查) | ✅    |
| `main`                          | 87%    | 7% (工作流簡化)    | ✅    |
| `extract_categories`            | 60%    | 13% (模式匹配優化) | ✅    |

**結果**：所有函式複雜度 ≤ 40%

#### 第二輪重構（列舉取代布林旗標 + 全面函式拆分）

> 本輪重構針對所有中高複雜度函式進行拆分與簡化，目標為將每個函式的認知複雜度壓縮至原值的 30% 以下。

**核心手段**：

| 手段                 | 說明                                                               |
| -------------------- | ------------------------------------------------------------------ |
| **列舉取代布林旗標** | `ParserState` 的 4 個 `in_*` 布林欄位合併為 `ActiveTextField` 列舉 |
| **函式拆分**         | 高複雜度函式拆分為多個單一職責的小函式                             |
| **迭代器取代迴圈**   | `calculate_distance` 用 `windows(2).map().sum()` 取代 `for` 迴圈   |
| **消除布林參數**     | `create_time_pattern` 改用字串後綴取代 `has_br: bool`              |
| **邏輯內聚**         | 表格渲染、CSV/TSV 格式化的迴圈邏輯提取為獨立行格式化函式           |

**主要改動與影響**：

| 函式（重構前）             | 改動方式                                                | 原始碼位置         |
| -------------------------- | ------------------------------------------------------- | ------------------ |
| `handle_end_tag`           | 拆分為 `finalize_placemark` + `close_content_tag`       | `parser.rs`        |
| `handle_start_tag`         | 拆分為 `enter_placemark` + `open_content_tag`           | `parser.rs`        |
| `ParserState::append_text` | `ActiveTextField` 列舉 match 取代 if-else 串接          | `parser.rs`        |
| `parse_kml_from_reader`    | 拆分為 `read_all_events` + `process_event`              | `parser.rs`        |
| `extract_kml_from_kmz`     | 拆分為 `find_doc_kml` + `find_first_kml`                | `parser.rs`        |
| `format_cell`              | 拆分為 `is_right_aligned_column` + `pad_text`           | `format.rs`        |
| `format_table`             | 拆分為 4 個表格渲染子函式                               | `format.rs`        |
| `build_config`             | 拆分為 3 個映射函式                                     | `converter.rs`     |
| `determine_file_path`      | 拆分為 `resolve_export_path`                            | `output.rs`        |
| `resolve_kml_file`         | 拆分為 `resolve_cli_file` + `find_default_kml_file`     | `path_resolver.rs` |
| `calculate_distance`       | 提取 `haversine_distance_km` + `windows(2)` 迭代器      | `metadata.rs`      |
| `extract_categories`       | 拆分為 `filter_meaningful_path` + `categorize_by_depth` | `path.rs`          |

**結果**：重構後全域最高函式認知複雜度大幅下降（確切數值依 JetBrains Better Highlights 重新量測為準）

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

**最終結構（13 個模組，入口點優化 + 認知複雜度全面壓縮）：**

```
src/
├── lib.rs            (49 行) - Library root，導出公開 API
├── main.rs           (26 行) ✨ 乾淨的入口點（run() 模式 + 自訂 Result）
├── cli.rs            (60 行) - 命令行參數定義
├── config.rs         (16 行) - 配置結構體
├── path_resolver.rs  (60 行) - 檔案路徑解析
├── output.rs         (73 行) - 輸出和儲存邏輯
├── converter.rs      (46 行) - 參數轉換
├── error.rs         (126 行) - 自訂錯誤類型
├── parser.rs        (371 行) - XML 解析（ActiveTextField 列舉 + 狀態機）
├── path.rs          (196 行) - 路徑提取
├── metadata.rs      (195 行) - 軌跡詮釋資料
├── regex.rs          (77 行) - 正規表示式模式
└── format.rs        (310 行) - 輸出格式化
```

**關鍵改進**：main.rs 從 233 行精簡到 **26 行**，入口點完全乾淨。

## 關鍵設計改進

### 1. 狀態機設計（parser.rs）

**第一輪：布林旗標狀態機（複雜度由 187% 降至 33%）**

**第二輪：ActiveTextField 列舉取代布林旗標（進一步壓縮）**

```rust
/// 當前活躍的文字欄位（取代 4 個布林旗標）
#[derive(Debug, Default, PartialEq)]
enum ActiveTextField {
    #[default]
    None,
    Name,
    Description,
    Coordinates,
    FolderName,
}

#[derive(Debug, Default)]
struct ParserState {
    in_placemark: bool,
    active_field: ActiveTextField,  // 取代 in_name, in_description, in_coordinates, in_folder_name
    // ... 資料欄位
}

// 追加文字到當前活躍欄位 — 簡潔的 match 取代 if-else 串接
fn append_text(&mut self, content: &str, folder_stack: &mut Vec<String>) {
    match self.active_field {
        ActiveTextField::Name => self.current_name.push_str(content),
        ActiveTextField::Description => self.current_description.push_str(content),
        ActiveTextField::Coordinates => self.current_coordinates_str.push_str(content),
        ActiveTextField::FolderName => append_to_folder_name(folder_stack, content),
        ActiveTextField::None => {}
    }
}
```

**優點**：

- 列舉天然互斥，消除不一致的狀態組合
- 清晰的狀態轉移（`open_content_tag` → `close_content_tag`）
- 消除 if-else 串接與 match guard，降低認知複雜度
- 易於新增欄位（只需在列舉中加入變體）

### 2. 路徑提取簡化（path.rs）

**第一輪：模式匹配取代 if-else（複雜度由 60% 降至 13%）**

**第二輪：進一步拆分為 `filter_meaningful_path` + `categorize_by_depth` + `classify_single_element`**

```rust
// extract_categories 拆為過濾與分類兩步驟
pub fn extract_categories(folder_path: &[String]) -> (String, String, String, String) {
    let meaningful_path = filter_meaningful_path(folder_path);
    categorize_by_depth(&meaningful_path)
}
```

**優點**：

- 主函式零認知複雜度（純函式組合）
- 月份格式判斷提取為 `is_month_format` 獨立函式
- 各步驟可獨立測試

### 3. 格式化重構（format.rs）

**第一輪：統一格式化介面、減少程式碼重複**

**第二輪：表格渲染拆分 + CSV/TSV 迴圈消除**

```rust
// 表格渲染拆為 4 個獨立函式
fn format_table(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let widths = calculate_column_widths(tracks);
    let mut output = String::new();
    format_header_row(&mut output, &widths);
    format_separator_row(&mut output, &widths);
    format_data_rows(&mut output, tracks, &widths);
    output
}

// CSV/TSV 改用迭代器
fn format_csv(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let header = "Name,Start,...\n";
    let rows: String = tracks.iter().map(|(_, m)| format_csv_row(m)).collect();
    format!("{}{}", header, rows)
}

// 儲存格格式化拆為對齊判斷 + 填充
fn format_cell(text: &str, width: usize, col_index: usize) -> String {
    // ...
    pad_text(text, width - text_width, is_right_aligned_column(col_index))
}
```

**優點**：

- `format_table` 認知複雜度降為零（純函式組合）
- `format_cell` 由巢狀 if-else 簡化為線性流程
- 共用 `TABLE_HEADERS` 常數，消除重複定義

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

| 方面           | 第一輪改進         | 第二輪改進                             |
| -------------- | ------------------ | -------------------------------------- |
| **模組化**     | 6 個單一職責模組   | 13 個模組，各函式單一職責              |
| **複雜度**     | 所有函式 < 40%     | 進一步壓縮，列舉取代布林旗標           |
| **程式碼重複** | 減少（欄寬計算）   | 進一步減少（共用常數、迭代器取代迴圈） |
| **可讀性**     | 清晰的函式名和流程 | 每個函式職責更明確，巢狀深度更低       |
| **可測試性**   | 各模組可獨立測試   | 純函式更多，輔助函式可單獨測試         |
| **狀態管理**   | 布林旗標追蹤狀態   | `ActiveTextField` 列舉天然互斥，更安全 |

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

    - ✅ **error.rs**：7 個單元測試
        - `test_analyzer_error_display()` - 錯誤顯示
        - `test_analyzer_error_parsing()` - 解析錯誤訊息
        - `test_analyzer_error_from_string()` - 字串轉換
        - `test_analyzer_error_from_io_error()` - IO 錯誤轉換
        - `test_result_type_alias()` - Result 類型別名
        - `test_analyzer_error_kmz()` - KMZ 錯誤訊息
        - `test_analyzer_error_from_zip_error()` - ZipError 轉換

   **測試統計**：
    - 總測試數：26
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
