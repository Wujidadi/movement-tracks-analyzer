---
name: cli-development
description: 處理新增 CLI 參數、輸出格式設定與路徑解析的架構規則。
---

# 命令行介面開發規則

## 資料流向強制規範
新增或修改命令列參數時，必須嚴格遵守以下單向資料流：
`cli::Args` (定義) -> `converter::build_config()` (驗證與映射) -> `config::Config` (業務邏輯使用) -> `output::output_results()` (執行結果)。

## 各模組修改職責
1. **`src/cli.rs`**:
   - 僅負責使用 `Clap` derive 宏定義介面。
   - CLI 專用列舉必須加上 `Arg` 後綴 (如 `OutputFormatArg`)，以區別核心領域模型。
2. **`src/converter.rs`**:
   - 負責將 `cli::Args` 轉換為內部 `Config`。
   - 所有自動降級邏輯 (如 Table 輸出至檔案時自動轉 CSV) 必須寫在此處。
3. **`src/config.rs`**:
   - 應用程式的核心設定檔，絕對不可依賴 `Clap` 相關套件或 `cli::Args` 型別。
4. **`src/path_resolver.rs`**:
   - 處理未指定檔案時的 Fallback 邏輯 (預設檔案名稱與目錄搜尋)。
