---
name: testing
description: 執行與撰寫 Rust 單元測試與集成測試的規範與常用指令。
---

# 測試規範與執行規則

## 測試分層與位置
1. **單元測試**：寫在 `src/**/*.rs` 底部，使用 `#[cfg(test)] mod tests` 包裝，針對純邏輯測試。
2. **集成測試**：寫在 `tests/*.rs` 中，必須透過公開 API (`movement_tracks_analyzer::...`) 呼叫。
3. **測試夾具 (Fixtures)**：KML/KMZ 檔案統一放在 `tests/fixtures/`，測試中需使用相對路徑讀取。

## 撰寫規範
- **命名規則**：`test_[動作]_[預期結果]_[條件]` (例：`test_calculate_distance_returns_zero_for_same_point`)。
- **模式**：嚴格遵守 Arrange (準備) -> Act (執行) -> Assert (驗證) 結構。
- **錯誤處理**：測試函式不應隨意回傳 `Result`，直接 `unwrap()` 或 `assert!` 即可。

## Agent 執行指令參考
- 跑全部測試：`cargo test`
- 只跑單元測試：`cargo test --lib`
- 只跑特定模組：`cargo test path::tests`
- 印出除錯日誌：`cargo test -- --nocapture`
- 跑單個集成測試：`cargo test --test kml_parsing`
