---
name: kml-parsing
description: 本專案的 KML/KMZ 檔案解析實現指南。說明流式 XML 解析的狀態機設計、座標點提取、軌跡詮釋資料結構化、KMZ 解壓縮策略、效能優化與邊界情況處理。當使用者涉及 KML/KMZ 解析改進、座標計算、時間處理或大型檔案效能時參照。
---

# KML/KMZ 檔案解析指南

## 技術棧

| 工具      | 版本 | 用途                                                |
| --------- | ---- | --------------------------------------------------- |
| quick-xml | 0.39 | **流式 XML 解析**；狀態機模式，避免全檔案載入記憶體 |
| zip       | 8.2  | KMZ（ZIP）檔案解壓縮；讀取壓縮檔中的 KML 內容       |
| regex     | 1.12 | 正規表示式模式，用於 Description 內的時間字串解析   |
| chrono    | 0.4  | 時間戳與日期時間處理                                |
| serde     | 1.0  | 序列化/反序列化軌跡資料結構                         |

---

## KML 格式概覽

### 典型的 KML 軌跡結構

```xml
<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
    <Document>
        <Folder>
            <name>運動軌跡</name>
            <Placemark>
                <name>2024-01-15 步行</name>
                <description>
                    分類: 戶外運動
                    活動: 步行
                </description>
                <LineString>
                    <coordinates>
                        121.5,25.0,100.0 121.6,25.1,105.0 121.7,25.2,110.0
                    </coordinates>
                </LineString>
            </Placemark>
        </Folder>
    </Document>
</kml>
```

### 關鍵元素

| 元素            | 說明                                    |
| --------------- | --------------------------------------- |
| `<Placemark>`   | 單個軌跡（一個完整的活動記錄）          |
| `<name>`        | 軌跡名稱（通常包含日期與活動類型）      |
| `<description>` | 軌跡詳細信息（通常包含分類與活動說明）  |
| `<LineString>`  | 座標序列容器                            |
| `<coordinates>` | 座標點清單（格式：`lon,lat,elevation`） |

---

## 架構：流式解析與狀態機

### 為什麼使用流式解析

- **記憶體效率**：大型 KML 檔案（數百 MB）無需全部載入記憶體
- **效能**：線性掃描，無需多次遍歷
- **可擴展性**：支援任意大小的檔案

### 狀態機設計

```
開始
  ↓
[等待 Placemark] ← 掃描 XML 事件
  ↓
[提取名稱] ← 在 <name> 內
  ↓
[提取描述] ← 在 <description> 內
  ↓
[解析座標] ← 在 <coordinates> 內
  ↓
[完成軌跡] ← Placemark 結束
  ↓
[輸出或儲存]
```

---

## 解析器實現（parser.rs）

### 核心結構

```rust
// src/parser.rs

use crate::{extract_categories, AnalyzerError, Result, TrackMetadata, END_TIME_PATTERN, START_TIME_PATTERN};
use chrono::NaiveDateTime;
use quick_xml::{events::Event, Reader};
use std::{
    fs,
    io::{BufRead, BufReader, Cursor, Read},
    path::PathBuf,
};

/// 從 KML 或 KMZ 檔案中提取所有 Placemark 軌跡
pub fn extract_placemarks_with_paths(
    file_path: &PathBuf,
) -> Result<Vec<(Vec<String>, TrackMetadata)>> {
    if is_kmz_file(file_path) {
        parse_kmz_file(file_path)
    } else {
        parse_kml_file(file_path)
    }
}

/// 從 KMZ（ZIP）檔案中提取第一個 KML 檔案的內容
fn extract_kml_from_kmz(file_path: &PathBuf) -> Result<Vec<u8>> {
    let file = fs::File::open(file_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // 優先尋找 doc.kml，退而求其次取第一個 .kml 檔案
    find_doc_kml(&mut archive)
        .or_else(|| find_first_kml(&mut archive))
        .ok_or_else(|| AnalyzerError::KmzError("No KML file found in KMZ archive".to_string()))
}

/// 從實作 BufRead 的來源解析 KML 內容
fn parse_kml_from_reader<R: BufRead>(reader: R) -> Result<Vec<(Vec<String>, TrackMetadata)>> {
    let mut xml_reader = Reader::from_reader(reader);
    // ... 初始化狀態 ...
    read_all_events(&mut xml_reader, &mut folder_stack, &mut state, &mut results)?;
    Ok(results)
}
```

### 狀態機設計

解析器使用 `ActiveTextField` 列舉取代布林旗標，搭配 `ParserState` 結構體管理狀態：

```rust
/// 當前活躍的文字欄位（天然互斥，取代 4 個布林旗標）
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
    active_field: ActiveTextField,
    current_name: String,
    current_description: String,
    current_coordinates_str: String,
}

impl ParserState {
    fn reset_placemark(&mut self) { /* 清除暫存欄位，重設 active_field = None */ }
    fn enter_placemark(&mut self) { /* in_placemark = true + reset */ }
    fn open_content_tag(&mut self, tag_name: &str, folder_stack: &[String]) { /* 設定活躍欄位 */ }
    fn close_content_tag(&mut self) { /* active_field = None */ }
    fn append_text(&mut self, content: &str, folder_stack: &mut Vec<String>) { /* 依 active_field 分派 */ }
    fn handle_cdata(&mut self, content: &str) { /* 僅在 Description 時追加 */ }
}
```

- `ActiveTextField` 列舉：天然互斥，消除不一致狀態組合
- `folder_stack`：追蹤當前 Folder 層級，用於提取分類路徑
- `handle_start_tag()` / `handle_end_tag()`：獨立函式處理 XML 標籤開閉事件
- `read_all_events()` → `process_event()`：事件迴圈與事件處理分離
- `finalize_placemark()`：Placemark 完成後建立 `TrackMetadata`

---

## 座標與時間解析

### 座標解析（parser.rs）

座標解析使用字串切割（非正規表示式），定義於 `parser.rs` 的 `parse_coordinates()` 私有函式：

```rust
// src/parser.rs

/// 解析座標字串為 (lon, lat) 對
fn parse_coordinates(coords_str: &str) -> Result<Vec<(f64, f64)>> {
    Ok(coords_str
        .trim()
        .split_whitespace()
        .filter_map(parse_single_coordinate)
        .collect())
}

/// 解析單個座標字串
fn parse_single_coordinate(coord_str: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = coord_str.split(',').collect();
    if parts.len() >= 2 {
        let lon = parts[0].parse().ok()?;
        let lat = parts[1].parse().ok()?;
        Some((lon, lat))
    } else {
        None
    }
}
```

- 座標格式：`lon,lat,elevation`（空白分隔多個座標點）
- 忽略高度（elevation），僅取經度和緯度
- 無效座標點靜默跳過（`filter_map`）

### 時間提取（parser.rs + regex.rs）

時間從 `<description>` 中的 HTML 提取，使用預編譯正規表示式：

```rust
// src/regex.rs

use once_cell::sync::Lazy;
use regex::Regex;

const DATETIME_PATTERN: &str = r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})";

fn create_time_pattern(label: &str, suffix: &str) -> String {
    format!(r"<b>\s*{}\s*:\s*</b>\s*{}{}", label, DATETIME_PATTERN, suffix)
}

pub static START_TIME_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&create_time_pattern("Start", r"<br />")).unwrap()
});

pub static END_TIME_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&create_time_pattern("End", "")).unwrap()
});
```

```rust
// src/parser.rs

/// 從 KML Description 中提取開始和結束時間
fn extract_times(description: &str) -> Option<(NaiveDateTime, NaiveDateTime)> {
    let start_match = START_TIME_PATTERN.captures(description)?;
    let end_match = END_TIME_PATTERN.captures(description)?;

    let start_str = start_match.get(1)?.as_str();
    let end_str = end_match.get(1)?.as_str();

    let start = NaiveDateTime::parse_from_str(start_str, "%Y-%m-%d %H:%M:%S").ok()?;
    let end = NaiveDateTime::parse_from_str(end_str, "%Y-%m-%d %H:%M:%S").ok()?;

    Some((start, end))
}
```

- 正規表示式使用 `once_cell::sync::Lazy` 全域快取，僅編譯一次
- Description 內容可能包含 CDATA，解析器已處理 `Event::CData` 事件

---

## 軌跡詮釋資料結構（metadata.rs）

### 資料結構設計

```rust
// src/metadata.rs

use chrono::NaiveDateTime;

/// 軌跡 Placemark 詮釋資料結構
#[derive(Debug, Clone)]
pub struct TrackMetadata {
    /// 軌跡名稱
    pub name: String,
    /// 開始時間
    pub start_time: NaiveDateTime,
    /// 結束時間
    pub end_time: NaiveDateTime,
    /// 座標點（經度、緯度）
    pub coordinates: Vec<(f64, f64)>,
    /// 分類（如「戶外運動」）
    pub category: String,
    /// 活動（如「步行」）
    pub activity: String,
    /// 年度（如「2026」）
    pub year: String,
    /// 月份（如「2026-03」）
    pub month: String,
}

impl TrackMetadata {
    /// 使用 windows(2) 迭代器搭配半正矢公式計算軌跡總距離（公尺）
    pub fn calculate_distance(&self) -> f64 {
        self.coordinates
            .windows(2)
            .map(|pair| haversine_distance_km(pair[0], pair[1]))
            .sum::<f64>()
            * 1000.0 // 轉換為公尺
    }

    /// 計算軌跡持續時間（秒）
    pub fn duration_seconds(&self) -> i64 {
        self.end_time
            .signed_duration_since(self.start_time)
            .num_seconds()
    }
}

/// 使用半正矢（Haversine）公式計算地球表面上兩點間的大圓距離（公里）
fn haversine_distance_km(point1: (f64, f64), point2: (f64, f64)) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;
    // ... Haversine 公式實作 ...
}
```

### 分類路徑提取（path.rs）

分類資訊從 Folder 堆棧（而非 Description）中提取：

```rust
// src/path.rs

/// 從 KML 資料夾路徑中提取軌跡分類資訊
pub fn extract_categories(folder_path: &[String]) -> (String, String, String, String) {
    let meaningful_path = filter_meaningful_path(folder_path);
    categorize_by_depth(&meaningful_path)
}

fn filter_meaningful_path(folder_path: &[String]) -> Vec<&String> {
    folder_path
        .iter()
        .filter(|name| !name.contains("(Example)") && !name.contains("Movement Tracks"))
        .collect()
}

fn categorize_by_depth(meaningful_path: &[&String]) -> (String, String, String, String) {
    match meaningful_path.len() {
        0 => empty_tuple(),
        1 => classify_single_element(meaningful_path[0]),
        2 => create_category_tuple(None, None, Some(0), Some(1), meaningful_path),
        3 => create_category_tuple(None, Some(0), Some(1), Some(2), meaningful_path),
        _ => { /* 使用最後四個元素 */ }
    }
}
```

- 回傳 `(category, activity, year, month)` 四元組
- 依路徑深度自動判斷各欄位對應位置
- 過濾掉根節點名稱（如 "Movement Tracks"）

---

## 邊界情況與錯誤處理

### 常見問題

| 問題                | 處理方式                                                 |
| ------------------- | -------------------------------------------------------- |
| 空軌跡（無座標點）  | 目前不強制過濾；若時間可解析，仍會產生該筆軌跡           |
| 座標格式異常        | 使用字串切割 + `parse()`；無效點由 `filter_map` 略過     |
| 時間戳缺失          | `extract_times()` 回傳 `None`，該 Placemark 不會寫入結果 |
| 超大檔案（GB 級別） | 流式解析確保恆定記憶體使用，詳見 PERFORMANCE.md          |
| 非 UTF-8 編碼 KML   | 使用 BufReader 與錯誤恢復，或清楚提示使用者              |
| KMZ 中無 KML 檔案   | 回傳 `AnalyzerError::KmzError("No KML file found...")`   |
| KMZ 中多個 KML 檔案 | **僅處理第一個**（優先 `doc.kml`，否則取首個 `.kml`）    |

### 錯誤處理範例

專案使用自訂 `AnalyzerError` 枚舉（定義於 `src/error.rs`）搭配 `Result<T>` 型態別名：

```rust
use crate::error::{AnalyzerError, Result};

pub fn extract_placemarks_with_paths(
    file_path: &PathBuf,
) -> Result<Vec<(Vec<String>, TrackMetadata)>> {
    let file = fs::File::open(file_path)?;  // io::Error 自動轉換為 AnalyzerError::Io

    // KMZ 相關錯誤（ZipError）自動轉換為 AnalyzerError::KmzError
    // KML 解析錯誤透過 ? 操作符回傳

    Ok(results)
}
```

`AnalyzerError` 變體包含：`Io`、`ParsingError`、`TimeParsingError`、`CoordinateParsingError`、`FileNotFound`、`KmzError`、`Other`。

---

## KMZ 檔案處理策略

### 解壓縮流程

KMZ 是 ZIP 格式的壓縮檔，內含 KML 檔案。解析 KMZ 時的處理流程：

1. 開啟 ZIP 壓縮檔（`zip::ZipArchive`）
2. 優先尋找 `doc.kml`（KMZ 規範的預設主檔案），若不存在則取**第一個** `.kml` 副檔名的檔案
3. 將 KML 內容讀入記憶體（`Vec<u8>`）
4. 以 `Cursor` 包裝後透過泛型 `BufRead` 介面送入與 KML 相同的流式解析器

### 單檔限制

> ⚠️ **重要限制**：目前 `extract_kml_from_kmz()` **只處理 KMZ 中的第一個 KML 檔案**。若 KMZ 包含多個 KML，工具不會合併、迭代或提示使用者。

此限制符合 KMZ 規範的主檔案概念（根目錄的 `doc.kml`），適用於大多數實際場景。若需支援多 KML 合併，應擴展該函式的邏輯。

---

## 效能優化建議

### 已實現的優化

1. **流式解析**：quick-xml 流式讀取，不載入全檔案
2. **緩衝 I/O**：BufReader 減少系統呼用
3. **正規表示式快取**：Lazy 單例化，避免重複編譯

### 進一步優化空間

- 多執行緒座標計算（若檔案超大）
- 座標點降採樣（若點數超多）
- 詳見 `PERFORMANCE.md`

---

## 測試 KML/KMZ 解析

### 使用測試夾具

```rust
#[test]
fn test_parse_sample_kml() {
    use movement_tracks_analyzer::extract_placemarks_with_paths;
    use std::path::PathBuf;

    let result = extract_placemarks_with_paths(&PathBuf::from("tests/fixtures/tracks.kml"));
    assert!(result.is_ok());

    let tracks = result.unwrap();
    assert!(!tracks.is_empty());

    let (_path, metadata) = &tracks[0];
    assert!(!metadata.name.is_empty());
    assert!(metadata.calculate_distance() >= 0.0);
}

#[test]
fn test_parse_sample_kmz() {
    use movement_tracks_analyzer::extract_placemarks_with_paths;
    use std::path::PathBuf;

    let result = extract_placemarks_with_paths(&PathBuf::from("tests/fixtures/tracks.kmz"));
    assert!(result.is_ok());

    let tracks = result.unwrap();
    assert!(!tracks.is_empty());
}

#[test]
fn test_kmz_no_kml_inside() {
    use movement_tracks_analyzer::{extract_placemarks_with_paths, AnalyzerError};
    use std::path::PathBuf;

    let result = extract_placemarks_with_paths(&PathBuf::from("tests/fixtures/empty.kmz"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AnalyzerError::KmzError(_)));
}
```

### 邊界情況測試

```rust
#[test]
fn test_nonexistent_file() {
    use movement_tracks_analyzer::extract_placemarks_with_paths;
    use std::path::PathBuf;

    let result = extract_placemarks_with_paths(&PathBuf::from("nonexistent.kml"));
    assert!(result.is_err());
}
```

---

## 添加新的解析特性

### 步驟 1：擴展 TrackMetadata（metadata.rs）

```rust
pub struct TrackMetadata {
    // ...既有欄位...
    pub new_field: String,
}
```

### 步驟 2：在 ActiveTextField 列舉中新增變體（parser.rs）

```rust
#[derive(Debug, Default, PartialEq)]
enum ActiveTextField {
    #[default]
    None,
    Name,
    Description,
    Coordinates,
    FolderName,
    NewElement,  // 新增
}
```

### 步驟 3：在 open_content_tag / handle_start_tag 中處理新標籤

```rust
// 在 ParserState::open_placemark_tag() 中新增匹配
fn open_placemark_tag(&mut self, tag_name: &str) {
    self.active_field = match tag_name {
        // ...既有匹配...
        "NewElement" => ActiveTextField::NewElement,
        _ => return,
    };
}

// 在 handle_start_tag() 中若新標籤非 name/description/coordinates，
// 需新增獨立的 match arm
```

### 步驟 4：更新 lib.rs 導出（若為公開 API）

若新欄位需透過函式庫 crate 導出，在 `src/lib.rs` 的 `pub use` 區塊中新增對應符號。

### 步驟 5：編寫測試

```rust
#[test]
fn test_extract_new_field() {
    use movement_tracks_analyzer::extract_placemarks_with_paths;
    use std::path::PathBuf;

    let tracks = extract_placemarks_with_paths(&PathBuf::from("tests/fixtures/tracks.kml")).unwrap();
    for (_path, metadata) in &tracks {
        // 驗證新欄位
        assert!(!metadata.new_field.is_empty());
    }
}
```

---

## 最佳實踐

- **流式處理**：對大型檔案優先使用流式解析
- **清楚的錯誤訊息**：協助使用者診斷 KML 檔案問題
- **完整的測試**：包含正常情況、邊界情況與錯誤情況
- **文件註解**：解釋複雜的解析邏輯
- **效能監控**：定期測試大型 KML 檔案的解析效能
