---
name: kml-parsing
description: 本專案的 KML 檔案解析實現指南。說明流式 XML 解析的狀態機設計、座標點提取、軌跡詮釋資料結構化、效能優化與邊界情況處理。當使用者涉及 KML 解析改進、座標計算、時間處理或大型檔案效能時參照。
---

# KML 檔案解析指南

## 技術棧

| 工具      | 版本 | 用途                                                |
| --------- | ---- | --------------------------------------------------- |
| quick-xml | 0.39 | **流式 XML 解析**；狀態機模式，避免全檔案載入記憶體 |
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

use crate::{extract_categories, Result, TrackMetadata, END_TIME_PATTERN, START_TIME_PATTERN};
use chrono::NaiveDateTime;
use quick_xml::{events::Event, Reader};
use std::{fs, io::BufReader, path::PathBuf};

/// 從 KML 檔案中提取所有 Placemark 軌跡
pub fn extract_placemarks_with_paths(
    file_path: &PathBuf,
) -> Result<Vec<(Vec<String>, TrackMetadata)>> {
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut xml_reader = Reader::from_reader(reader);

    let mut results = Vec::new();
    let mut buf = Vec::new();
    let mut folder_stack: Vec<String> = Vec::new();
    let mut parser_state = ParserState::default();

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(elem)) => {
                let tag_name = String::from_utf8_lossy(elem.name().as_ref()).to_string();
                handle_start_tag(&tag_name, &mut folder_stack, &mut parser_state);
            }
            Ok(Event::End(elem)) => {
                let tag_name = String::from_utf8_lossy(elem.name().as_ref()).to_string();
                handle_end_tag(&tag_name, &mut folder_stack, &mut parser_state, &mut results)?;
            }
            Ok(Event::Text(text)) => {
                let content = String::from_utf8_lossy(text.as_ref()).to_string();
                parser_state.append_text(&content, &mut folder_stack);
            }
            Ok(Event::CData(cdata)) => {
                let content = String::from_utf8_lossy(cdata.as_ref()).to_string();
                if parser_state.in_description {
                    parser_state.current_description.push_str(&content);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("KML parsing error: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(results)
}
```

### 狀態機設計

解析器使用 `ParserState` 結構體集中管理所有狀態旗標與暫存資料：

```rust
#[derive(Debug, Default)]
struct ParserState {
    in_placemark: bool,
    in_name: bool,
    in_description: bool,
    in_coordinates: bool,
    in_folder_name: bool,
    current_name: String,
    current_description: String,
    current_coordinates_str: String,
}

impl ParserState {
    fn reset_placemark(&mut self) { /* 清除暫存欄位 */ }
    fn append_text(&mut self, content: &str, folder_stack: &mut Vec<String>) { /* 依狀態分派文字 */ }
}
```

- `ParserState`：集中管理所有解析狀態（布林旗標 + 暫存字串）
- `folder_stack`：追蹤當前 Folder 層級，用於提取分類路徑
- `handle_start_tag()` / `handle_end_tag()`：獨立函式處理 XML 標籤開閉事件
- `buf`：XML 事件緩衝區（提高效能）

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
        .filter_map(|coord_str| {
            let parts: Vec<&str> = coord_str.split(',').collect();
            if parts.len() >= 2 {
                let lon = parts[0].parse().ok()?;
                let lat = parts[1].parse().ok()?;
                Some((lon, lat))
            } else {
                None
            }
        })
        .collect())
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

fn create_time_pattern(label: &str, has_br: bool) -> String {
    let br = if has_br { r"<br />" } else { "" };
    format!(r"<b>\s*{}\s*:\s*</b>\s*{}{}", label, DATETIME_PATTERN, br)
}

pub static START_TIME_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&create_time_pattern("Start", true)).unwrap()
});

pub static END_TIME_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&create_time_pattern("End", false)).unwrap()
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
    /// 使用半正矢（Haversine）公式計算軌跡總距離（公尺）
    pub fn calculate_distance(&self) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;
        let mut total_distance = 0.0;

        for i in 0..self.coordinates.len() - 1 {
            let (lon1, lat1) = self.coordinates[i];
            let (lon2, lat2) = self.coordinates[i + 1];

            let lat1_rad = lat1.to_radians();
            let lat2_rad = lat2.to_radians();
            let delta_lat = (lat2 - lat1).to_radians();
            let delta_lon = (lon2 - lon1).to_radians();

            let a = (delta_lat / 2.0).sin().powi(2)
                + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
            let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

            total_distance += EARTH_RADIUS_KM * c;
        }

        total_distance * 1000.0 // 轉換為公尺
    }

    /// 計算軌跡持續時間（秒）
    pub fn duration_seconds(&self) -> i64 {
        self.end_time
            .signed_duration_since(self.start_time)
            .num_seconds()
    }
}
```

### 分類路徑提取（path.rs）

分類資訊從 Folder 堆棧（而非 Description）中提取：

```rust
// src/path.rs

/// 從 KML 資料夾路徑中提取軌跡分類資訊
pub fn extract_categories(folder_path: &[String]) -> (String, String, String, String) {
    let meaningful_path: Vec<&String> = folder_path
        .iter()
        .filter(|name| !name.contains("(Example)") && !name.contains("Movement Tracks"))
        .collect();

    match meaningful_path.len() {
        0 => (String::new(), String::new(), String::new(), String::new()),
        1 => extract_single_element(&meaningful_path),
        2 => create_category_tuple(None, None, Some(0), Some(1), &meaningful_path),
        3 => create_category_tuple(None, Some(0), Some(1), Some(2), &meaningful_path),
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

### 錯誤處理範例

專案使用自訂 `AnalyzerError` 枚舉（定義於 `src/error.rs`）搭配 `Result<T>` 型態別名：

```rust
use crate::error::{AnalyzerError, Result};

pub fn extract_placemarks_with_paths(
    file_path: &PathBuf,
) -> Result<Vec<(Vec<String>, TrackMetadata)>> {
    let file = fs::File::open(file_path)?;  // io::Error 自動轉換為 AnalyzerError::Io

    // ...解析邏輯...

    Ok(results)
}
```

`AnalyzerError` 變體包含：`Io`、`ParsingError`、`TimeParsingError`、`CoordinateParsingError`、`FileNotFound`、`Other`。

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

## 測試 KML 解析

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
fn test_coordinates_parsing() {
    // parse_coordinates 為 parser.rs 的私有函式，
    // 透過整合測試間接驗證：
    use movement_tracks_analyzer::extract_placemarks_with_paths;
    use std::path::PathBuf;

    let tracks = extract_placemarks_with_paths(&PathBuf::from("tests/fixtures/tracks.kml")).unwrap();
    for (_path, metadata) in &tracks {
        assert!(!metadata.coordinates.is_empty(), "每個軌跡應至少有一個座標點");
    }
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

### 步驟 2：在 ParserState 中新增暫存欄位（parser.rs）

```rust
#[derive(Debug, Default)]
struct ParserState {
    // ...既有欄位...
    in_new_element: bool,
    current_new_data: String,
}
```

### 步驟 3：在 handle_start_tag / handle_end_tag 中處理新標籤

```rust
fn handle_start_tag(tag_name: &str, folder_stack: &mut Vec<String>, state: &mut ParserState) {
    match tag_name {
        // ...既有匹配...
        "NewElement" if state.in_placemark => {
            state.in_new_element = true;
        }
        _ => {}
    }
}
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
