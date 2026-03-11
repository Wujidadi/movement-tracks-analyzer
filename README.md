# Movement Tracks Analyzer

## 功能概述

- 解析 KML 格式的 GPS 移動軌跡檔案
  > 個人自用的檔案，一般名為 `移動軌跡.kml`
- 提取軌跡的開始/結束時間、距離、持續時間、座標點數量
- 區分為：分類（戶外運動、動力交通工具...）、活動（步行、自行車、飛機...）、年份、月份
- 支持多種輸出格式：JSON、CSV、TSV、表格
- 支持 Unicode 正確對齊

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

## 技術細節

### 性能優化

採用 **流式 XML 解析**（quick-xml）替代正規表達式掃描：
- 只掃描 KML 檔案一次，自動追蹤 XML 層級
- 預編譯正規表達式，避免重複編譯開銷
- **結果**：O(n²) → O(n) 複雜度，800 倍性能提升

詳見 [PERFORMANCE.md](./PERFORMANCE.md)

### 距離計算

- 使用**半正矢（Haversine）公式**計算地球表面兩點間的大圓距離
- 地球半徑：6371 km
- 結果單位：公尺（m）

### 時間提取

- 從 KML Description 提取時間戳
- 格式：`YYYY-MM-DD HH:MM:SS`
- 使用正規表示法提取 Start 和 End 時間

### 路徑追蹤

- 按 XML 嵌套順序追蹤 Folder 層級
- 自動去除根節點（如 "移動軌跡"）
- 取最後 4 個 Folder 作為分類路徑（分類、活動、年度、月份）

### 表格對齊

- 使用 `unicode-width` 庫正確計算漢字寬度（漢字 = 2 寬）
- 自動計算欄寬
- 數字欄位（Duration、Distance、Points）靠右對齊，其他欄位靠左對齊
