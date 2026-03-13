---
name: kml-parsing
description: 本專案的 KML 檔案解析實現指南。說明流式 XML 解析的狀態機設計、座標點提取、軌跡詮釋資料結構化、性能優化與邊界情況處理。當使用者涉及 KML 解析改進、座標計算、時間處理或大型檔案性能時參照。
---

# KML 檔案解析指南

## 技術棧

| 工具      | 版本 | 用途                                                |
| --------- | ---- | --------------------------------------------------- |
| quick-xml | 0.39 | **流式 XML 解析**；狀態機模式，避免全檔案載入記憶體 |
| regex     | 1.12 | 正規表示式模式，用於座標與時間字串解析              |
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
- **性能**：線性掃描，無需多次遍歷
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

use quick_xml::events::{Event, BytesStart, BytesText};
use quick_xml::Reader;
use crate::metadata::TrackMetadata;
use crate::path::GpsPoint;
use crate::regex::{parse_coordinates, extract_metadata};
use std::io::BufReader;
use std::fs::File;

/// 從 KML 檔案解析軌跡資料
pub fn extract_placemarks_with_paths(file_path: &str) -> Result<Vec<TrackMetadata>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut xml_reader = Reader::from_reader(reader);

    let mut placemarks = Vec::new();
    let mut current_placemark = TrackMetadata::default();
    let mut in_placemark = false;
    let mut in_coordinates = false;
    let mut buf = Vec::new();

    loop {
        match xml_reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                match e.name().as_ref() {
                    b"Placemark" => {
                        in_placemark = true;
                        current_placemark = TrackMetadata::default();
                    }
                    b"coordinates" if in_placemark => {
                        in_coordinates = true;
                    }
                    _ => {}
                }
            }

            Event::End(ref e) => {
                match e.name().as_ref() {
                    b"Placemark" if in_placemark => {
                        if !current_placemark.points.is_empty() {
                            placemarks.push(current_placemark.clone());
                        }
                        in_placemark = false;
                    }
                    b"coordinates" if in_coordinates => {
                        in_coordinates = false;
                    }
                    _ => {}
                }
            }

            Event::Text(ref e) => {
                let text = e.unescape()?.into_owned();

                if in_placemark {
                    if in_coordinates {
                        // 解析座標
                        if let Ok(points) = parse_coordinates(&text) {
                            current_placemark.points.extend(points);
                        }
                    }
                }
            }

            Event::Eof => break,
            _ => {}
        }

        buf.clear();
    }

    Ok(placemarks)
}
```

### 狀態管理

- `in_placemark`：當前是否在 `<Placemark>` 內
- `in_coordinates`：當前是否在 `<coordinates>` 內
- `current_placemark`：累積中的軌跡資料
- `buf`：XML 事件緩衝區（提高性能）

---

## 座標與時間解析（regex.rs）

### 座標解析

```rust
// src/regex.rs

use regex::Regex;
use once_cell::sync::Lazy;
use crate::path::GpsPoint;

/// 座標分隔符正規表示式
static COORD_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(-?\d+\.\d+),(-?\d+\.\d+),(-?\d+(?:\.\d+)?)?").unwrap()
});

/// 解析座標字串為 GPS 點集合
pub fn parse_coordinates(text: &str) -> Result<Vec<GpsPoint>, String> {
    let mut points = Vec::new();

    for caps in COORD_PATTERN.captures_iter(text) {
        let lon: f64 = caps[1].parse()
            .map_err(|_| "經度解析失敗")?;
        let lat: f64 = caps[2].parse()
            .map_err(|_| "緯度解析失敗")?;
        let elevation: f64 = caps.get(3)
            .map(|m| m.as_str().parse())
            .transpose()
            .map_err(|_| "高度解析失敗")?
            .unwrap_or(0.0);

        points.push(GpsPoint {
            latitude: lat,
            longitude: lon,
            elevation,
        });
    }

    if points.is_empty() {
        return Err("未找到座標點".to_string());
    }

    Ok(points)
}

/// 從描述欄提取分類與活動
pub fn extract_metadata(description: &str) -> (Option<String>, Option<String>) {
    let mut category = None;
    let mut activity = None;

    for line in description.lines() {
        if let Some(value) = line.strip_prefix("分類:") {
            category = Some(value.trim().to_string());
        }
        if let Some(value) = line.strip_prefix("活動:") {
            activity = Some(value.trim().to_string());
        }
    }

    (category, activity)
}
```

### 時間字串解析

```rust
use chrono::NaiveDateTime;

/// 從軌跡名稱或時間戳解析日期時間
pub fn parse_timestamp(name: &str) -> Option<chrono::NaiveDateTime> {
    // 嘗試從名稱中提取日期（如 "2024-01-15 步行"）
    let date_pattern = Lazy::new(|| {
        Regex::new(r"(\d{4})-(\d{2})-(\d{2})\s+(\d{2}):?(\d{2})?").unwrap()
    });

    if let Some(caps) = date_pattern.captures(name) {
        if let Ok(dt) = NaiveDateTime::parse_from_str(
            &format!("{}-{}-{} {}:{}:00",
                     &caps[1], &caps[2], &caps[3],
                     caps.get(4).map(|m| m.as_str()).unwrap_or("00"),
                     caps.get(5).map(|m| m.as_str()).unwrap_or("00")
            ),
            "%Y-%m-%d %H:%M:%S"
        ) {
            return Some(dt);
        }
    }

    None
}
```

---

## 軌跡詮釋資料結構（metadata.rs）

### 資料結構設計

```rust
// src/metadata.rs

use chrono::NaiveDateTime;
use crate::path::GpsPoint;

/// 軌跡詮釋資料
#[derive(Debug, Clone)]
pub struct TrackMetadata {
    /// 軌跡名稱
    pub name: String,

    /// 分類（如「戶外運動」）
    pub category: Option<String>,

    /// 活動類型（如「步行」、「自行車」）
    pub activity: Option<String>,

    /// 開始時間戳
    pub start_time: Option<NaiveDateTime>,

    /// 結束時間戳
    pub end_time: Option<NaiveDateTime>,

    /// GPS 座標點集合
    pub points: Vec<GpsPoint>,

    /// 總距離（公里）
    pub distance: f64,

    /// 持續時間（秒）
    pub duration_seconds: u64,

    /// 座標點數量
    pub points_count: usize,
}

impl Default for TrackMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            category: None,
            activity: None,
            start_time: None,
            end_time: None,
            points: Vec::new(),
            distance: 0.0,
            duration_seconds: 0,
            points_count: 0,
        }
    }
}

impl TrackMetadata {
    /// 從座標點計算距離與持續時間
    pub fn compute_metrics(&mut self) {
        self.points_count = self.points.len();

        if self.points.len() < 2 {
            self.distance = 0.0;
            return;
        }

        // 計算總距離
        self.distance = compute_total_distance(&self.points);

        // 計算時間差（若有開始與結束時間）
        if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            self.duration_seconds = (end - start).num_seconds().max(0) as u64;
        }
    }

    /// 檢查軌跡是否有效
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.points.is_empty()
    }
}

/// 計算軌跡總距離
fn compute_total_distance(points: &[GpsPoint]) -> f64 {
    let mut total = 0.0;
    for i in 1..points.len() {
        total += points[i - 1].distance_to(&points[i]);
    }
    total
}
```

---

## 邊界情況與錯誤處理

### 常見問題

| 問題                | 處理方式                                        |
| ------------------- | ----------------------------------------------- |
| 空軌跡（無座標點）  | 驗證 `points.len() > 0`，跳過無效軌跡           |
| 座標格式異常        | 使用正規表示式驗證，捕捉解析異常                |
| 時間戳缺失          | 設為 `None`，後續邏輯判斷是否必須               |
| 超大檔案（GB 級別） | 流式解析確保恆定記憶體使用，詳見 PERFORMANCE.md |
| 非 UTF-8 編碼 KML   | 使用 BufReader 與錯誤恢復，或清楚提示使用者     |

### 錯誤處理範例

```rust
pub fn parse_kml_safe(file_path: &str) -> Result<Vec<TrackMetadata>, String> {
    let file = File::open(file_path)
        .map_err(|e| format!("無法開啟檔案: {}", e))?;

    let reader = BufReader::new(file);
    let mut xml_reader = Reader::from_reader(reader);

    let mut placemarks = Vec::new();

    // ...解析邏輯...

    if placemarks.is_empty() {
        return Err("檔案中未找到有效的軌跡".to_string());
    }

    Ok(placemarks)
}
```

---

## 性能優化建議

### 已實現的優化

1. **流式解析**：quick-xml 流式讀取，不載入全檔案
2. **緩衝 I/O**：BufReader 減少系統呼用
3. **正規表示式快取**：Lazy 單例化，避免重複編譯
4. **避免字串複製**：使用 `as_ref()` 與 `Cow<str>`

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
    let result = extract_placemarks_with_paths("tests/fixtures/tracks.kml");
    assert!(result.is_ok());

    let tracks = result.unwrap();
    assert!(!tracks.is_empty());
    assert!(tracks[0].is_valid());
}

#[test]
fn test_coordinates_parsing() {
    let coords = "121.5,25.0,100.0 121.6,25.1,105.0 121.7,25.2,110.0";
    let result = parse_coordinates(coords);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 3);
}
```

### 邊界情況測試

```rust
#[test]
fn test_empty_coordinates() {
    let result = parse_coordinates("");
    assert!(result.is_err());
}

#[test]
fn test_malformed_coordinates() {
    let result = parse_coordinates("invalid,data");
    assert!(result.is_err());
}
```

---

## 添加新的解析特性

### 步驟 1：擴展 TrackMetadata

```rust
pub struct TrackMetadata {
    // ...既有欄位...
    pub new_field: Option<String>,
}
```

### 步驟 2：在解析器中提取新資料

```rust
Event::Text( ref e) => {
let text = e.unescape() ?.into_owned();
if in_new_element {
current_placemark.new_field = Some(text);
}
}
```

### 步驟 3：在正規表示式或提取邏輯中處理

```rust
pub fn extract_new_data(text: &str) -> Option<String> {
    // 實現提取邏輯
}
```

### 步驟 4：編寫測試

```rust
#[test]
fn test_extract_new_data() {
    let result = extract_new_data("expected_value");
    assert_eq!(result, Some("expected_value".to_string()));
}
```

---

## 最佳實踐

- **流式處理**：對大型檔案優先使用流式解析
- **清楚的錯誤訊息**：協助使用者診斷 KML 檔案問題
- **完整的測試**：包含正常情況、邊界情況與錯誤情況
- **文件註解**：解釋複雜的解析邏輯
- **性能監控**：定期測試大型 KML 檔案的解析性能
