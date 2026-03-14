# Movement Tracks Analyzer

## 功能概述

- 解析 KML/KMZ 格式的 GPS 移動軌跡檔案
  > 個人自用的軌跡紀錄檔，一般名為 `移動軌跡.kml` 或 `移動軌跡.kmz`
- 提取軌跡的開始/結束時間、距離、持續時間、座標點數量
- 區分為：分類（戶外運動、動力交通工具...）、活動（步行、自行車、飛機...）、年份、月份
- 支援多種輸出格式：JSON、CSV、TSV、表格（支援 Unicode 字元正確對齊）

> **KMZ 支援限制**：目前僅支援解析 KMZ 檔案中的**單一 KML 內容**。若 KMZ 包含多個 KML 檔案，工具只會處理其中第一個（優先 `doc.kml`，否則取首個 `.kml` 檔）。

## 快速開始

### 編譯

```bash
cargo build --release
```

### 運行

```bash
# 查看說明
./target/release/movement_tracks_analyzer -h

# 從預設路徑載入 KML/KMZ 檔（優先級：移動軌跡.kml > Movement Tracks.kml > 移動軌跡.kmz > Movement Tracks.kmz）
./target/release/movement_tracks_analyzer

# 指定 KML 或 KMZ 檔（短格式）
./target/release/movement_tracks_analyzer -f "path/to/file.kml"
./target/release/movement_tracks_analyzer -f "path/to/file.kmz"

# 指定 KML 檔（長格式）
./target/release/movement_tracks_analyzer --file "path/to/file.kml"
```

## 命令行參數

| 參數                    | 說明                                                   |
| ----------------------- | ------------------------------------------------------ |
| `-f, --file <PATH>`     | 指定 KML/KMZ 檔路徑                                    |
| `-o, --output <TYPE>`   | 輸出類型：`shell`（命令行）或 `file`（檔案，預設）     |
| `-m, --format <FORMAT>` | 輸出格式：`json`、`csv`、`tsv`、`table`（默認：`csv`） |
| `-x, --export <PATH>`   | 輸出檔案路徑（支持目錄或完整檔案路徑，預設為當前目錄） |
| `-h, --help`            | 顯示說明信息                                           |

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

採用**自訂錯誤類型**取代 `Box<dyn Error>`，提供類型安全和清晰的錯誤訊息。

## 開發者指南

本專案採用**AI Agent 協作指引**架構，支援 GitHub Copilot、Claude Code、Cursor 等工具的智能協作。指引文件採取**四層分層設計**，在正確的時機提供正確的上下文。

### 協作指引結構

```
.github/
├── copilot-instructions.md         # 入口文件
├── AGENTS.md                       # 全域操作規範 ← 根目錄軟連結
├── instructions/
│   └── rust.instructions.md        # Rust 開發規範
└── skills/
    ├── testing/SKILL.md            # 測試技能
    ├── cli-development/SKILL.md    # CLI 開發技能
    └── kml-parsing/SKILL.md        # KML 解析技能
```

### 快速參考

| 內容                                                                                       | 用途                                    |
| ------------------------------------------------------------------------------------------ | --------------------------------------- |
| [`.github/copilot-instructions.md`](./.github/copilot-instructions.md)                     | AI Agent 協作入口與語言規則             |
| [`AGENTS.md`](./AGENTS.md)                                                                 | 全域操作規範、技術概覽、禁止事項        |
| [`.github/instructions/rust.instructions.md`](./.github/instructions/rust.instructions.md) | Rust 開發規範、架構模式、修改入口       |
| [`.github/skills/`](./.github/skills/)                                                     | 特定任務技能模組（測試、CLI、KML 解析） |

### 與 AI Agent 協作

新增功能或修復 Bug 時，請遵循 `.github/` 中的協作指引。Agent 會自動載入相應的上下文，提供：

- ✅ 一致的程式風格與命名規則
- ✅ 完整的測試驅動開發指南
- ✅ 模組化架構與最佳實踐
- ✅ 效能與最佳化建議

如需更新協作指引上下文，明確告知 Agent：「更新協作指引上下文」

### 使用本地協作指引

新對話時，Agent 會自動載入 `.github/copilot-instructions.md` 及相應層次的指引文件。若需手動指引 Agent，可參考：

1. **編輯程式碼時**：自動載入 `.github/instructions/rust.instructions.md`
2. **撰寫測試時**：自動載入 `.github/skills/testing/SKILL.md`
3. **開發 CLI 功能時**：自動載入 `.github/skills/cli-development/SKILL.md`
4. **改進 KML 解析時**：自動載入 `.github/skills/kml-parsing/SKILL.md`

## 更多資訊

| 文檔                                 | 用途                                    |
| ------------------------------------ | --------------------------------------- |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | 📐 專案結構、模組設計、資料流、統計數據  |
| [PERFORMANCE.md](./PERFORMANCE.md)   | ⚡ 效能優化技術、流式 XML 解析、效能數據 |
| [REFACTORING.md](./REFACTORING.md)   | 🔧 程式碼重構過程、複雜度改進、設計模式  |
