# Movement Tracks Analyzer

## 功能概述

- 解析 KML 格式的 GPS 運動軌跡文件
- 提取軌跡的開始/結束時間、距離、持續時間、坐標點數
- 自動分類為：大類別、小類別、年份、月份
- 支持多種輸出格式：JSON、CSV、TSV、表格
- 自動識別汉字等寬字符，表格顯示對齊

## 快速開始

### 編譯

```bash
cargo build --release
```

### 運行

```bash
# 查看幫助
./target/release/movement_tracks_analyzer -h

# 自動查找默認 KML 文件（優先級：移動軌跡.kml > Movement Tracks.kml）
./target/release/movement_tracks_analyzer

# 指定 KML 文件（短格式）
./target/release/movement_tracks_analyzer -f "path/to/file.kml"

# 指定 KML 文件（長格式）
./target/release/movement_tracks_analyzer --file "path/to/file.kml"
```

## 命令行參數

| 參數 | 說明 |
|------|------|
| `-f, --file <PATH>` | 指定 KML 文件路徑 |
| `-o, --output <TYPE>` | 輸出類型：`shell`（命令行）或 `file`（文件，默認） |
| `-m, --format <FORMAT>` | 輸出格式：`json`、`csv`、`tsv`、`table`（默認：csv） |
| `-h, --help` | 顯示幫助信息 |

## 輸出格式說明

### CSV / TSV / JSON
- 命令行或文件輸出
- `table` 格式在 `-o file` 時自動轉為 CSV

### 表格（Table）
- 僅用於 `-o shell` 模式
- 自動計算列寬，漢字正確對齐

## 輸出欄位

1. **Placemark Name** - 地標名稱
2. **Start Time** - 開始時間（YYYY-MM-DD HH:MM:SS）
3. **End Time** - 結束時間
4. **Duration (seconds)** - 持續時間（秒）
5. **Distance (meters)** - 里程（米）
6. **Coordinate Count** - 軌跡點數量
7. **Category Major** - 大類別（如 戶外運動）
8. **Category Minor** - 小類別（如 步行）
9. **Year** - 年份（如 2025）
10. **Month** - 年月（如 2026-03）

## 使用範例

### 輸出表格到命令行
```bash
./target/release/movement_tracks_analyzer -f "Movement Tracks.kml" -o shell -m table
```

### 輸出 JSON 到文件
```bash
./target/release/movement_tracks_analyzer -f "Movement Tracks.kml" -o file -m json
# 生成 tracks_output.json
```

### 輸出 CSV 到文件（默認）
```bash
./target/release/movement_tracks_analyzer -f "Movement Tracks.kml"
# 生成 tracks_output.csv
```

## 技術細節

### 距離計算
- 使用 **Haversine 公式** 計算地球表面兩點間的大圓距離
- 地球半徑：6371 km
- 結果單位：米（m）

### 時間提取
- 從 KML Description 字段提取時間戳
- 格式：`YYYY-MM-DD HH:MM:SS`
- 使用正則表達式提取 Start 和 End 時間

### 路徑追踪
- 按 XML 嵌套順序追踪 Folder 層級
- 自動去除根節點（如 "Movement Tracks (Example)"）
- 取最後 4 個 Folder 作為分類信息

### 表格對齐
- 使用 `unicode-width` 庫正確計算漢字寬度
- 漢字寬度視為英文字符的 2 倍
- 自動計算列寬並進行左對齐填充

## 依賴庫

```toml
chrono = "0.4"        # 時間處理
regex = "1.12"        # 正則表達式
kml = "0.13"          # KML 格式支持
walkdir = "2.5"       # 目錄遍歷
unicode-width = "0.1" # 漢字寬度計算
```

## 常見問題

**Q: 程序找不到 KML 文件？**  
A: 確保文件名為 `移動軌跡.kml` 或 `Movement Tracks.kml`，或使用 `-f` 參數明確指定路徑。

**Q: 表格顯示不對齐？**  
A: 已解決，現在正確支持漢字顯示寬度。

**Q: 某些軌跡沒有顯示？**  
A: 只顯示包含完整 Start/End 時間戳的軌跡（格式：YYYY-MM-DD HH:MM:SS）。

**Q: 如何批量處理多個文件？**  
A: 目前需分別運行各文件，可考慮後續添加批量處理功能。

## 項目狀態

✅ 功能完整  
✅ 編譯成功  
✅ 全面測試通過  
✅ 生產就緒

