# 效能優化說明

## 效能改進成果

使用**流式 XML 解析器** (`quick-xml`) 替代原有的正規表示式方案，實現了劇烈的效能提升：

### 處理時間對比

| KML 檔案                 | 原始實現 | 優化後      | 改進倍數         |
| ------------------------ | -------- | ----------- | ---------------- |
| 48 MB / 2,164 Placemarks | ~4 分鐘  | ~0.3-0.9 秒 | **240-800 倍** ✅ |

### 各格式輸出時間（最新版本）

- **CSV (shell)**: 0.858s
- **表格 (shell)**: 0.529s
- **JSON (file)**: 0.392s
- **CSV (file)**: 0.330s

## 優化技術

### 1. 流式 XML 解析（核心改進）

**原始方案的問題**：

- 使用正規表示式搜尋 `<Placemark>` 標籤（每個 Placemark 掃描整個檔案近 50MB）
- 為了獲取 Folder 路徑，每個 Placemark 又要掃描一遍整個檔案找父元素
- **總掃描次數**：3,000 Placemarks × 2 次 = 6,000 次全文掃描
- **複雜度**：O(n² × m)，其中 n = Placemarks 數量，m = 檔案大小

**優化方案**：

```rust
// 使用 quick-xml 的事件驅動解析
// 只掃描一次檔案，自動追蹤 Folder 堆棧
// 邊解析邊處理，無須整個載入到記憶體

// 事件迴圈拆為 read_all_events() + process_event()，降低認知複雜度
fn read_all_events<R: BufRead>(xml_reader: &mut Reader<R>, ...) -> Result<()> {
    let mut buf = Vec::new();
    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => return Ok(()),
            Ok(event) => process_event(event, ...)?,
            Err(e) => { eprintln!("KML parsing error: {}", e); return Ok(()); }
        }
        buf.clear();
    }
}
```

**優點**：

- ✅ 只掃描檔案一次（O(n) 複雜度）
- ✅ 自動維護 Folder 堆棧，無須重複查找
- ✅ 流式處理，記憶體佔用恆定
- ✅ 原生支援 XML 層級和 CDATA 處理

### 2. 預編譯正規表示式

使用 `once_cell::sync::Lazy` 快取正規表示式，避免重複編譯：

```rust
static START_TIME_PATTERN: Lazy<Regex> = Lazy::new(||
    Regex::new(r"<b>\s*Start\s*:\s*</b>\s*(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})<br />").unwrap()
);
```

- 第一次調用時編譯一次，後續直接使用
- **節省**：原本每處理一個 Placemark 就會編譯一次正規表示式，現在全域共用一次

### 3. Unicode 寬字元支援

表格輸出中正確計算漢字寬度（2 倍顯示寬度）：

```rust
fn display_width(s: &str) -> usize {
    s.width()  // 使用 unicode-width crate
}
```

### 4. 數字欄位靠右對齊

表格輸出格式中的 Duration、Distance、Points 等數字欄位靠右對齊，提升可讀性。

## 依賴優化

從舊的依賴結構改為：

```toml
[dependencies]
chrono = "0.4"           # 日期時間處理
once_cell = "1.21"       # ⭐ 惰性靜態初始化（快取正規表示式）
quick-xml = "0.39"       # ⭐ 流式 XML 解析（核心改進）
regex = "1.12"           # 時間戳提取（預編譯）
unicode-width = "0.2"    # Unicode 寬字元計算
zip = "8.2"              # KMZ（ZIP）解壓縮
```

移除了低效的 `roxmltree`（DOM 解析方式），採用更高效的事件驅動模式。

## KMZ 檔案的記憶體策略

KMZ 是 ZIP 格式的壓縮檔，內含 KML 檔案。解析 KMZ 時的處理流程：

1. 開啟 ZIP 壓縮檔（`zip::ZipArchive`）
2. 優先尋找 `doc.kml`（KMZ 規範的預設主檔案），若不存在則取第一個 `.kml` 檔案
3. 將 KML 內容解壓至記憶體（`Vec<u8>`）
4. 以 `Cursor` 包裝後透過泛型 `BufRead` 介面送入與 KML 相同的流式解析器

> **單檔限制**：目前僅支援 KMZ 中的**單一 KML 內容**。若 KMZ 包含多個 KML 檔案，工具只會處理其中第一個（優先 `doc.kml`，否則取首個 `.kml` 檔）。

**記憶體影響**：KMZ 中的 KML 需完整解壓至記憶體後才能進行 XML 解析，因此不是完全的流式處理。對一般大小的 KMZ（數十 MB 解壓後）影響有限。若需支援 GB 級 KMZ，可考慮寫入暫存檔再流式讀取。

## 結論

使用流式 XML 解析將效能從 **4 分鐘** 提升至 **0.3 秒**，對於大型 KML 檔案（50MB+）的即時分析已達產品等級。

## 相關閱讀

- [README.md](./README.md) - 快速開始使用
- [ARCHITECTURE.md](./ARCHITECTURE.md) - 詳細的模組設計說明
- [REFACTORING.md](./REFACTORING.md) - 程式碼重構與品質改進
