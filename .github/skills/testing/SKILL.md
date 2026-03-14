---
name: testing
description: 本專案的單元測試與集成測試策略與規範。說明如何撰寫 Rust 單元測試與集成測試、各層次的測試重點與命名慣例，以及執行測試的完整規則。當使用者要撰寫、執行、調整或討論測試（含 cargo test 語法）時參照。
---

# 單元與集成測試策略

## 測試框架

| 框架              | 用途                                       |
| ----------------- | ------------------------------------------ |
| Rust #[test]      | **標準內置測試**；所有單元與集成測試均使用 |
| cargo test        | 測試執行工具，支援選擇性執行與詳細輸出     |
| #[cfg(test)] 區塊 | 模組內單元測試，條件編譯，發行版本中排除   |

---

## 測試分層

| 層次     | 位置              | 適用情境                                                                                |
| -------- | ----------------- | --------------------------------------------------------------------------------------- |
| 單元測試 | `src/**` #[test]  | 純邏輯、計算、資料轉換，不涉及 I/O 或檔案系統                                           |
| 集成測試 | `tests/*.rs`      | 完整的功能流程、跨模組協作，涉及檔案解析或命令行呼用                                    |
| 文件測試 | 公開 API doc-test | 公開函式的使用範例（涵蓋 `lib.rs`、`parser.rs`、`path.rs`、`metadata.rs`、`format.rs`） |
| 測試夾具 | `tests/fixtures/` | 測試用的 KML 檔案與其他靜態測試資料                                                     |

---

## 單元測試結構範例

```rust
// src/metadata.rs
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> TrackMetadata { /* 建立測試用 TrackMetadata */ }

    #[test]
    fn test_duration_seconds() {
        let metadata = create_test_metadata();
        assert_eq!(metadata.duration_seconds(), 5400);
    }
}
```

### 測試編寫規則

1. **命名**：`test_` 前綴 + 清楚的描述，例如 `test_same_point_distance_zero`
2. **結構**：Arrange（準備資料）→ Act（執行）→ Assert（驗證）
3. **獨立性**：每個測試應獨立，無依賴順序
4. **清晰度**：優先清楚而非簡潔，讓他人快速理解測試意圖

---

## 集成測試結構範例

```rust
// tests/kml_parsing.rs

use movement_tracks_analyzer::extract_placemarks_with_paths;
use std::path::{Path, PathBuf};

#[test]
fn test_parse_sample_kml_file() {
    let fixture_path = PathBuf::from("tests/fixtures/tracks.kml");
    assert!(Path::new(&fixture_path).exists(), "測試夾具檔案應存在");

    let result = extract_placemarks_with_paths(&fixture_path);
    assert!(result.is_ok(), "應成功解析 KML 檔案");

    let tracks = result.unwrap();
    assert!(!tracks.is_empty(), "應解析至少一個軌跡");
}
```

---

## 執行測試

```bash
# 執行所有測試
cargo test

# 執行單個測試函數
cargo test test_same_point_distance_zero

# 執行單個模組中的所有測試
cargo test path::tests

# 執行單個集成測試檔案的所有測試
cargo test --test kml_parsing

# 顯示 println! 與日誌輸出
cargo test -- --nocapture

# 執行名稱包含 "distance" 的所有測試
cargo test distance

# 僅執行單元測試 / 僅執行集成測試
cargo test --lib
cargo test --test '*'
```

---

## 測試夾具（Fixtures）

測試夾具存放於 `tests/fixtures/`：

| 檔案                | 用途                       |
| ------------------- | -------------------------- |
| `tracks.kml`        | 標準 KML 測試檔案          |
| `tracks.kmz`        | KMZ 測試檔案（含 doc.kml） |
| `tracks_no_doc.kmz` | KMZ 測試檔案（非 doc.kml） |
| `empty.kmz`         | 空 KMZ 測試檔案            |

- KML 檔案應代表真實的 GPS 軌跡格式
- 應包含各種邊界情況：空軌跡、單點軌跡、多軌跡檔案等
- 夾具檔案應 UTF-8 編碼

---

## 測試驅動開發（TDD）流程

1. **編寫測試**：定義預期行為 → `cargo test test_new_feature`（確認失敗）
2. **實現功能**：撰寫最小可行實現
3. **執行測試**：`cargo test test_new_feature`（確認通過）
4. **重構**：改進程式碼品質，測試應持續通過

---

## 測試覆蓋與最佳實踐

### 應包含的測試場景

- **正常流程**：合法輸入產生預期輸出
- **邊界情況**：空集合、單一元素、最大值等
- **錯誤情況**：無效輸入、缺失檔案、格式錯誤等

### 測試命名約定

- 使用清楚的動詞：`test_returns_`、`test_handles_`、`test_throws_`
- 說明測試的條件與預期結果：`test_calculate_distance_returns_zero_for_same_point`
- 避免含糊的名稱如 `test_it_works` 或 `test_basic`

### 避免的模式

- **避免過度測試**：不需要測試語言或標準函式庫的行為
- **避免測試實現細節**：測試公開 API，而非內部私有函數
- **避免共享狀態**：測試應獨立，不應相互依賴或按特定順序執行
- **避免複製程式碼**：提取共同的測試設置為輔助函數

---

## 持續整合

- 提交程式碼前，應執行 `cargo test` 確保所有測試通過
- 新增功能時應同步添加對應的單元與集成測試
- 修復 Bug 時應先編寫失敗的測試，再實現修復
