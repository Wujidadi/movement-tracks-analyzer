---
name: kml-parsing
description: 處理 KML/KMZ 解析、狀態機擴展、座標提取與時間處理的硬性規則。
---

# KML/KMZ 解析實作規則

## 1. 流式解析強制規定 (parser.rs)
- **嚴格禁止**將大型 KML 檔案全部載入記憶體或使用 DOM 解析。
- 必須透過 `quick-xml` 的 `Event` 迴圈搭配 `ParserState` 狀態機處理。
- 狀態管理：使用 `ActiveTextField` 列舉管理當前標籤狀態，不應為每個標籤新增獨立的布林旗標。

## 2. 座標與時間處理
- **座標 (parser.rs)**：使用空白分割字串後解析 `lon,lat,elevation`，靜默忽略高度欄位，並跳過無效座標。
- **時間 (regex.rs)**：使用 `Lazy<Regex>` 提取 HTML Description 中的時間戳，快取編譯結果。若解析失敗回傳 `None`，並捨棄該筆 Placemark。

## 3. 分類路徑提取 (path.rs)
- 從 `<Folder>` 堆疊提取分類，不依賴 Description。
- 回傳 `(category, activity, year, month)` 四元組，必須過濾非有效分類節點（如 "Movement Tracks"、含 "(Example)" 的資料夾）。

## 4. KMZ 處理邊界條件
- `extract_kml_from_kmz()` **僅處理第一筆 KML**。
- 優先尋找 `doc.kml`，若無則取首個 `.kml`。
- 若需支援多 KML 檔案合併，必須顯式修改此函式並處理記憶體與疊加邏輯。
