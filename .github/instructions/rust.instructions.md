# Rust 開發規範與架構地圖

本文件定義本專案的架構模式與修改入口。

## 技術棧約束
- **Edition**: Rust 2024
- **CLI**: Clap 4.x (derive 模式)
- **XML**: quick-xml 0.39 (流式狀態機，嚴禁全檔載入記憶體)
- **ZIP**: zip 8.2 (KMZ 解壓縮)
- **時間**: chrono 0.4
- **正規表示**: regex 1.12
- **全域快取**: once_cell 1.x (`Lazy` 模式)
- **序列化**: serde + serde_json (TrackMetadata 序列化)
- **表格寬度**: unicode-width 0.2 (CJK 字元對齊)

## 架構地圖
```
.
├── src/
│   ├── lib.rs            # 公開 API 導出
│   ├── main.rs           # CLI 主程式 (run() 模式, 自訂 Result)
│   ├── cli.rs            # 命令行參數定義 (Clap)
│   ├── config.rs         # 應用配置結構體
│   ├── path_resolver.rs  # 預設檔案自動尋找 (KML/KMZ)
│   ├── converter.rs      # CLI args 轉 Config
│   ├── output.rs         # 輸出目標判定 (shell / file)
│   ├── error.rs          # AnalyzerError 枚舉
│   ├── regex.rs          # Regex 全域快取 (Lazy)
│   ├── parser.rs         # XML 流式解析與狀態機
│   ├── path.rs           # 軌跡分類提取邏輯
│   ├── metadata.rs       # 軌跡資料結構 (TrackMetadata)
│   └── format.rs         # 輸出格式化
└── tests/                # 獨立的集成測試檔案與 fixtures
```

## 程式碼風格與慣例
1. **錯誤處理**：使用 `Result<T, AnalyzerError>`，嚴禁隨意使用 `unwrap()` 或 `panic!()`。
2. **命名規則**：模組與函式 `snake_case`，結構體 `PascalCase`，常數 `SCREAMING_SNAKE_CASE`。
3. **可見性**：謹慎使用 `pub`，模組內部共享使用 `pub(crate)`。
4. **註解規範**：公開 API 必須有 `///` 註解。行內 `//` 註解只解釋「為什麼這樣寫」，不解釋「這行在做什麼」。

## 常見修改入口
- **新增 CLI 參數**：`cli.rs` (定義) -> `converter.rs` (轉換) -> `config.rs` (儲存)。
- **修改解析邏輯**：`parser.rs` (XML 狀態機) / `metadata.rs` (運算) / `path.rs` (分類)。
- **新增輸出格式**：`format.rs` (格式化) -> `cli.rs` (參數) -> `converter.rs` (映射)。
