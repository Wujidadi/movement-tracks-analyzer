---
name: kml-parsing
description: 本專案的 KML/KMZ 檔案解析實現指南。說明流式 XML 解析的狀態機設計、座標點提取、軌跡詮釋資料結構化、KMZ 解壓縮策略與邊界情況處理。當使用者涉及 KML/KMZ 解析改進、座標計算、時間處理或大型檔案效能時參照。
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
                <description>分類: 戶外運動 / 活動: 步行</description>
                <LineString>
                    <coordinates>121.5,25.0,100.0 121.6,25.1,105.0</coordinates>
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

### 狀態機流程

```
[等待 Placemark] → [提取名稱] → [提取描述] → [解析座標] → [完成軌跡] → [輸出或儲存]
```

---

## 解析器實現（parser.rs）

### 核心入口

`extract_placemarks_with_paths(file_path: &PathBuf) -> Result<Vec<(Vec<String>, TrackMetadata)>>`：依副檔名判斷 KML 或 KMZ，分派至對應解析函式。

呼叫鏈：`extract_placemarks_with_paths()` → `parse_kmz_file()` / `parse_kml_file()` → `parse_kml_from_reader()` → `read_all_events()` → `process_event()`

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
```

`ParserState` 關鍵方法：

| 方法                  | 職責                                     |
| --------------------- | ---------------------------------------- |
| `enter_placemark()`   | 進入 Placemark，重設暫存欄位             |
| `open_content_tag()`  | 依標籤名稱設定 `active_field`            |
| `close_content_tag()` | 重設 `active_field = None`               |
| `append_text()`       | 依 `active_field` 分派文字內容至對應欄位 |
| `handle_cdata()`      | 僅在 Description 時追加 CDATA 內容       |
| `reset_placemark()`   | 清除暫存欄位                             |

事件處理：`handle_start_tag()` / `handle_end_tag()` 為獨立函式處理 XML 標籤開閉事件，`finalize_placemark()` 在 Placemark 結束後建立 `TrackMetadata`。

---

## 座標與時間解析

### 座標解析（parser.rs）

`parse_coordinates()` 使用字串切割（非正規表示式），透過 `split_whitespace()` + `filter_map(parse_single_coordinate)` 解析座標：

- 座標格式：`lon,lat,elevation`（空白分隔多個座標點）
- 忽略高度（elevation），僅取經度和緯度
- 無效座標點靜默跳過（`filter_map`）

### 時間提取（parser.rs + regex.rs）

時間從 `<description>` 中的 HTML 提取。`regex.rs` 定義 `START_TIME_PATTERN` 與 `END_TIME_PATTERN` 兩個 `Lazy<Regex>` 全域快取（使用 `once_cell::sync::Lazy`，僅編譯一次），匹配格式如 `<b>Start:</b> 2026-03-11 10:00:00<br />`。

`extract_times()` 從 Description 擷取 `Start` / `End` 時間戳，回傳 `Option<(NaiveDateTime, NaiveDateTime)>`，無法解析時該 Placemark 不寫入結果。Description 內容可能包含 CDATA，解析器已處理 `Event::CData` 事件。

---

## 軌跡詮釋資料（metadata.rs）

`TrackMetadata` 結構體欄位：`name`、`start_time`/`end_time`（`NaiveDateTime`）、`coordinates`（`Vec<(f64, f64)>`）、`category`、`activity`、`year`、`month`。

- `calculate_distance()`：使用 `windows(2)` 迭代器搭配 `haversine_distance_km()` 半正矢公式計算軌跡總距離
- `duration_seconds()`：計算軌跡持續時間

### 分類路徑提取（path.rs）

分類資訊從 Folder 堆棧中提取（而非 Description）：

`extract_categories()` → `filter_meaningful_path()` → `categorize_by_depth()`

- 回傳 `(category, activity, year, month)` 四元組
- 依路徑深度自動判斷各欄位對應位置
- 過濾掉根節點名稱（如 "Movement Tracks"）

---

## KMZ 檔案處理策略

KMZ 是 ZIP 格式的壓縮檔，處理流程：

1. 開啟 ZIP 壓縮檔（`zip::ZipArchive`）
2. 優先尋找 `doc.kml`（KMZ 規範的預設主檔案），若不存在則取**第一個** `.kml` 副檔名的檔案
3. 將 KML 內容讀入記憶體（`Vec<u8>`）
4. 以 `Cursor` 包裝後透過泛型 `BufRead` 介面送入與 KML 相同的流式解析器

> ⚠️ **單檔限制**：目前 `extract_kml_from_kmz()` **只處理 KMZ 中的第一個 KML 檔案**。若 KMZ 包含多個 KML，工具不會合併、迭代或提示使用者。若需支援多 KML 合併，應擴展該函式的邏輯。

---

## 邊界情況與錯誤處理

| 問題                | 處理方式                                                 |
| ------------------- | -------------------------------------------------------- |
| 空軌跡（無座標點）  | 目前不強制過濾；若時間可解析，仍會產生該筆軌跡           |
| 座標格式異常        | 使用字串切割 + `parse()`；無效點由 `filter_map` 略過     |
| 時間戳缺失          | `extract_times()` 回傳 `None`，該 Placemark 不會寫入結果 |
| 超大檔案（GB 級別） | 流式解析確保恆定記憶體使用，詳見 `PERFORMANCE.md`        |
| 非 UTF-8 編碼 KML   | 使用 BufReader 與錯誤恢復，或清楚提示使用者              |
| KMZ 中無 KML 檔案   | 回傳 `AnalyzerError::KmzError("No KML file found...")`   |
| KMZ 中多個 KML 檔案 | **僅處理第一個**（優先 `doc.kml`，否則取首個 `.kml`）    |

錯誤處理使用自訂 `AnalyzerError` 枚舉（定義於 `src/error.rs`），變體包含：`Io`、`ParsingError`、`TimeParsingError`、`CoordinateParsingError`、`FileNotFound`、`KmzError`、`Other`。

---

## 添加新的解析特性

1. **擴展 `TrackMetadata`**（`metadata.rs`）：新增欄位
2. **新增 `ActiveTextField` 變體**（`parser.rs`）：註冊新的 XML 標籤
3. **更新標籤處理**（`parser.rs`）：在 `open_content_tag()` / `handle_start_tag()` 中處理新標籤
4. **更新 `lib.rs` 導出**：若為公開 API，在 `pub use` 區塊中新增對應符號
5. **編寫測試**：參照 [`skills/testing/SKILL.md`](../testing/SKILL.md) 的測試策略

> 效能相關改動請參閱 `PERFORMANCE.md`。測試規範請參閱 [`skills/testing/SKILL.md`](../testing/SKILL.md)。
