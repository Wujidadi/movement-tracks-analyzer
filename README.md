# Movement Tracks Analyzer

## 功能概述

- 解析 KML 格式的 GPS 移動軌跡檔案
  > 個人自用的軌跡紀錄檔，一般名為 `移動軌跡.kml`
- 提取軌跡的開始/結束時間、距離、持續時間、座標點數量
- 區分為：分類（戶外運動、動力交通工具...）、活動（步行、自行車、飛機...）、年份、月份
- 支援多種輸出格式：JSON、CSV、TSV、表格（支援 Unicode 字元正確對齊）

## 快速開始

### 編譯

```bash
cargo build --release
```

### 運行

```bash
# 查看說明
./target/release/movement_tracks_analyzer -h

# 從預設路徑載入 KML 檔（優先級：移動軌跡.kml > Movement Tracks.kml）
./target/release/movement_tracks_analyzer

# 指定 KML 檔（短格式）
./target/release/movement_tracks_analyzer -f "path/to/file.kml"

# 指定 KML 檔（長格式）
./target/release/movement_tracks_analyzer --file "path/to/file.kml"
```

## 命令行參數

| 參數                      | 說明                                        |
|-------------------------|-------------------------------------------|
| `-f, --file <PATH>`     | 指定 KML 檔路徑                                |
| `-o, --output <TYPE>`   | 輸出類型：`shell`（命令行）或 `file`（檔案，預設）          |
| `-m, --format <FORMAT>` | 輸出格式：`json`、`csv`、`tsv`、`table`（默認：`csv`） |
| `-x, --export <PATH>`   | 輸出檔案路徑（支持目錄或完整檔案路徑，預設為當前目錄）               |
| `-h, --help`            | 顯示說明信息                                    |

## 輸出格式說明

### CSV / TSV / JSON

- 命令行或檔案輸出
- `table` 格式在 `-o file` 時自動轉為 CSV

### 表格（Table）

- 僅用於 `-o shell` 模式
- 自動計算欄寬，Unicode 寬字元正確對齊

## 輸出欄位

1. **Placemark Name** - 軌跡名稱
2. **Start Time** - 開始時間（YYYY-MM-DD HH:MM:SS）
3. **End Time** - 結束時間
4. **Duration (seconds)** - 持續時間（秒）
5. **Distance (meters)** - 里程（公尺）
6. **Coordinate Count** - 軌跡點數量
7. **Category** - 分類（戶外運動、動力交通工具...）
8. **Category Minor** - 活動（步行、自行車、飛機...）
9. **Year** - 年份（2013、2025...）
10. **Month** - 年月（2015-08、2026-03...）

## 使用範例

### 輸出表格到命令行

```bash
./target/release/movement_tracks_analyzer -f "Movement Tracks.kml" -o shell -m table
```

### 輸出 JSON 檔案

```bash
./target/release/movement_tracks_analyzer -f "Movement Tracks.kml" -o file -m json
# 生成 tracks_output.json
```

### 輸出 CSV 檔案（預設）

```bash
./target/release/movement_tracks_analyzer -f "Movement Tracks.kml"
# 生成 tracks_output.csv
```

### 指定輸出路徑

```bash
# 輸出到指定目錄（使用預設檔名）
./target/release/movement_tracks_analyzer -o file -m json -x /tmp
# 生成 /tmp/tracks_output.json

# 輸出到自訂檔名
./target/release/movement_tracks_analyzer -o file -m csv -x /tmp/my_data.csv
# 生成 /tmp/my_data.csv
```

## 技術細節

### 效能優化

採用**流式 XML 解析**替代正規表示式掃描，實現 800 倍效能提升。詳見 [PERFORMANCE.md](./PERFORMANCE.md)

### 距離計算與軌跡分析

- 使用**半正矢（Haversine）公式**計算地球表面大圓距離
- 自動追蹤 XML 層級獲取軌跡分類信息
- 支援完整的時間和座標解析

詳細實現見 [ARCHITECTURE.md](./ARCHITECTURE.md) 中的模組詳解。

### 程式碼品質

透過狀態機設計、模組化重構等技術，將認知複雜度降低 87%。詳見 [REFACTORING.md](./REFACTORING.md)

採用**自訂錯誤類型**取代 `Box<dyn Error>`，提供類型安全和清晰的錯誤消息。

## 更多資訊

| 文檔                                   | 用途                      |
|--------------------------------------|-------------------------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | 📐 專案結構、模組設計、資料流、統計數據   |
| [PERFORMANCE.md](./PERFORMANCE.md)   | ⚡ 效能優化技術、流式 XML 解析、效能數據 |
| [REFACTORING.md](./REFACTORING.md)   | 🔧 程式碼重構過程、複雜度改進、設計模式   |
