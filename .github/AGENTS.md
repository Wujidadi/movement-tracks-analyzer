# Agent 操作指引

> [!IMPORTANT]
> **語言規則（最高優先，無例外）：所有回應、說明、文件、工具呼叫的 explanation 欄位，一律使用繁體中文（台灣），並採用台灣標準翻譯和慣用術語。無論處理任何技術問題，此規則不得被覆蓋或忽略。回應文字中不得夾雜日語、韓語或其他非中文詞彙（包含感嘆句、慣用語）。**

> **重要提示**：本檔案 `.github/AGENTS.md` 是**原始檔案**。根目錄的 `AGENTS.md` 是本檔案的符號連結（Symbolic Link），會自動與本檔案保持同步。修改時應直接編輯本檔案（`.github/AGENTS.md`），**勿修改根目錄版本**，否則會造成改動內容重複且難以同步。

本文件規範 AI 在本專案中的行為準則、作業流程、命令執行方式與禁止事項。  
語言專項規範請參閱：

- Rust 開發規範：[`instructions/rust.instructions.md`](instructions/rust.instructions.md)
- 繁體中文用語規範：[`instructions/language.instructions.md`](instructions/language.instructions.md)

執行特定類型任務時，Agent 應自動載入對應的技能模組（`skills/`）：

| 任務類型                             | 技能模組                                                             |
| ------------------------------------ | -------------------------------------------------------------------- |
| 撰寫、執行或討論單元與集成測試       | [`skills/testing/SKILL.md`](skills/testing/SKILL.md)                 |
| 開發命令行界面、參數解析或使用者互動 | [`skills/cli-development/SKILL.md`](skills/cli-development/SKILL.md) |
| 實作或改進 KML 檔案解析邏輯          | [`skills/kml-parsing/SKILL.md`](skills/kml-parsing/SKILL.md)         |

## 通則

- 回應與文件一律使用**繁體中文（台灣）**，並採用台灣標準翻譯和慣用術語，例如：程式、使用者、設定、回傳、呼叫、元件、參數、效能、記憶體。
- 修改前應先蒐集需求相關上下文，不得臆測模組位置、命名規則、資料結構或業務邏輯。
- 實作應以**最小必要修改**為原則，避免引入無關重構、命名變更或大範圍搬移。
- 若需求跨越多個層次，應先說明可能影響的檔案與原因，再進行修改。
- 在不違反現有架構前提下，應優先沿用既有模式、工具與實踐，不任意引入與現況衝突的大型框架或重寫方案。

## 專案概覽

Movement Tracks Analyzer 是一個 **GPS 軌跡解析工具**，透過解析 KML/KMZ 檔案，提取軌跡的時間、距離、座標等資訊，並支援多種輸出格式（JSON、CSV、TSV、表格）。

> **KMZ 支援限制**：目前僅支援解析 KMZ 檔案中的**單一 KML 內容**。若 KMZ 包含多個 KML 檔案，工具只會處理其中第一個（優先 `doc.kml`，否則取首個 `.kml` 檔）。

## 專案技術概覽

| 層次     | 技術                                  |
| -------- | ------------------------------------- |
| 語言     | Rust 2024 Edition                     |
| 命令行   | Clap（derive 宏風格）                 |
| XML 解析 | quick-xml（流式解析，狀態機模式）     |
| 壓縮檔   | zip（KMZ 解壓縮）                     |
| 序列化   | serde + serde_json                    |
| 時間處理 | chrono                                |
| 正規表示 | regex                                 |
| 測試     | Rust 內置 #[test]、單元測試與集成測試 |

## 檔案與編譯

- 編譯使用 `cargo build --release` 產生最終執行檔。
- 專案採用**雙 crate 架構**：二進位 crate（`src/main.rs`）負責 CLI 入口與流程編排，函式庫 crate（`src/lib.rs`）導出核心解析邏輯。二進位 crate 內的模組（`cli`、`config`、`converter`、`output`、`path_resolver`）透過 `mod` 宣告引入；跨 crate 引用使用 `use movement_tracks_analyzer::{...}`。
- 主程式入口為 `src/main.rs`（26 行），採用 `run()` 函式模式搭配自訂 `Result` 型態，錯誤由 `eprintln!` 輸出後以非零狀態碼退出。
- 核心邏輯保持在 `src/lib.rs` 導出的模組中（`error`、`format`、`metadata`、`parser`、`path`、`regex`），確保可作為函式庫被重複使用。
- 錯誤處理使用自訂 `AnalyzerError` 枚舉（定義於 `src/error.rs`），搭配 `Result<T>` 型態別名；包含 7 種錯誤變體（`Io`、`ParsingError`、`TimeParsingError`、`CoordinateParsingError`、`FileNotFound`、`KmzError`、`Other`），並實作 `From` traits 支援自動轉換。

## 效能與優化

- 專案有效能優化文件（`PERFORMANCE.md`），涉及效能改進時應先閱讀該文件。
- KML 檔案使用**流式解析**（`ActiveTextField` 列舉 + 狀態機），避免將整個檔案載入記憶體。
- 解析器狀態以 `ActiveTextField` 列舉（`None`/`Name`/`Description`/`Coordinates`/`FolderName`）管理當前活躍欄位，取代先前的 4 個布林旗標，天然互斥。
- 距離計算使用 `windows(2)` 迭代器搭配獨立的 `haversine_distance_km()` 函式。

## 設定與環境

- 命令行參數定義於 `src/cli.rs`，透過 Clap derive 宏實現。
- 檔案路徑解析由 `src/path_resolver.rs` 負責，預設會依序嘗試 `移動軌跡.kml`、`Movement Tracks.kml`、`移動軌跡.kmz`、`Movement Tracks.kmz`（KML 優先）。

## 驗證與命令執行

- 單元測試應包含於模組內的 `#[cfg(test)]` 區塊（目前 26 個，涵蓋 `path`、`metadata`、`regex`、`error` 四個模組）。
- 集成測試應放在 `tests/` 目錄下的 Rust 檔案，每個檔案為獨立的 crate。
- 公開 API 應附帶 doc-tests（目前 5 個，涵蓋 `lib.rs`、`parser.rs`、`path.rs`、`metadata.rs`、`format.rs`）。
- 執行測試使用 `cargo test`；執行特定測試使用 `cargo test test_name`。
- 編譯前應執行 `cargo check` 驗證程式碼無誤。
- 當涉及 KML/KMZ 檔案解析改動時，應使用 `tests/fixtures/tracks.kml` 及 `tests/fixtures/tracks.kmz` 等進行驗證。

## 文件說明

- 說明修改內容時，應交代修改檔案、修改原因、可能影響範圍與建議驗證方式。
- 若新增功能涉及命令行參數、輸出格式或檔案處理邏輯，應評估是否同步更新 `README.md` 或相關文件。
- 架構相關改動應更新 `ARCHITECTURE.md`；效能相關改動應更新 `PERFORMANCE.md`；重構相關改動應更新 `REFACTORING.md`。

## 禁止和強制事項

- 若接收到「更新協作指引上下文」或等義的指令，務必確實重新讀取並理解 `.github/copilot-instructions.md` 及本目錄下所有指引文件，不得直接回應「已更新」或類似的訊息，並輸出理解到的有所變更的上下文。
- 不得在未確認上下文前直接假設模組結構、資料結構或業務邏輯。
- 不得因追求新穎而任意升級依賴版本、重寫既有模組或改變既有技術選擇。
- 不得忽略既有架構分層的目的與限制。
- 不得將臨時除錯程式、測試資料或本機路徑留在正式程式碼中。
- **不得將任務完成報告（總結「做了什麼」的文件）加入版控**：執行分析、重構或其他複雜任務後，生成的總結報告應刪除或存放在臨時位置（如 `/tmp` 或本地 `.gitignore` 檔案），不應加入專案中。此類文件僅供任務過程參考，無需歸檔。
- **禁止在終端中直接使用 heredoc 語法**（如 `cat << 'EOF'`、`python3 << 'PYEOF'`）：此類方式會導致終端進入持續等待狀態（`heredoc>` 提示符），用戶必須手動按 Ctrl-C 才能跳出。改用以下方式替代：
  1. 創建臨時檔案後執行（`cat > /tmp/file.py << 'EOF'` ... `EOF && python3 /tmp/file.py`）
  2. 直接使用 echo 輸出（`echo "..."` 多次）
  3. 將完整指令寫入檔案後運行
