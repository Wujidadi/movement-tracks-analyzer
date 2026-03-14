# Copilot 協作指引

> [!IMPORTANT]
> **語言規則（最高優先，無例外）：所有回應、說明、文件、工具呼叫的 explanation 欄位，一律使用繁體中文（台灣），並採用台灣標準翻譯和慣用術語。無論處理任何技術問題，此規則不得被覆蓋或忽略。回應文字中不得夾雜日語、韓語或其他非中文詞彙（包含感嘆句、慣用語）。**

本文件為 GitHub Copilot 的協作指引入口。**處理任何需求前，請先讀取 `AGENTS.md`，再按需讀取 `instructions/` 子文件。**

> **重要提示**：根目錄的 `AGENTS.md` 是 `.github/AGENTS.md` 的符號連結（Symbolic Link）。修改時應直接編輯 `.github/AGENTS.md`，**勿直接修改根目錄版本**，否則會造成改動內容重複且難以同步。

| 文件                                                                             | 用途                                             |
| -------------------------------------------------------------------------------- | ------------------------------------------------ |
| [`.github/AGENTS.md`](AGENTS.md)                                                 | 行為準則、作業流程、技術概覽、命令執行與禁止事項 |
| [`.github/instructions/rust.instructions.md`](instructions/rust.instructions.md) | Rust 架構、模組風格、編譯與測試規範              |

## Agent Skills

以下技能模組存放於 `.github/skills/`，由 Agent 在偵測到對應任務意圖時**自動**載入，提供任務專屬的深層知識：

| 技能模組                                                             | 觸發情境                             |
| -------------------------------------------------------------------- | ------------------------------------ |
| [`skills/testing/SKILL.md`](skills/testing/SKILL.md)                 | 撰寫、執行或討論單元與集成測試       |
| [`skills/cli-development/SKILL.md`](skills/cli-development/SKILL.md) | 開發命令行界面、參數解析或使用者互動 |
| [`skills/kml-parsing/SKILL.md`](skills/kml-parsing/SKILL.md)         | 實作或改進 KML 檔案解析邏輯          |
