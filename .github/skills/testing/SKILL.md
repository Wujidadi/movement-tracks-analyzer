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

| 層次     | 位置              | 適用情境                                             |
| -------- | ----------------- | ---------------------------------------------------- |
| 單元測試 | `src/**` #[test]  | 純邏輯、計算、資料轉換，不涉及 I/O 或檔案系統        |
| 集成測試 | `tests/*.rs`      | 完整的功能流程、跨模組協作，涉及檔案解析或命令行呼用 |
| 測試夾具 | `tests/fixtures/` | 測試用的 KML 檔案與其他靜態測試資料                  |

---

## 單元測試結構範例

### 模組內測試

```rust
// src/metadata.rs

use chrono::NaiveDateTime;

impl TrackMetadata {
    /// 計算軌跡總距離（公尺），使用半正矢公式
    pub fn calculate_distance(&self) -> f64 {
        // ... 實作
    }

    /// 計算軌跡持續時間（秒）
    pub fn duration_seconds(&self) -> i64 {
        self.end_time
            .signed_duration_since(self.start_time)
            .num_seconds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> TrackMetadata {
        TrackMetadata {
            name: "Test Track".to_string(),
            start_time: NaiveDateTime::parse_from_str("2026-03-11 10:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            end_time: NaiveDateTime::parse_from_str("2026-03-11 11:30:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            coordinates: vec![(120.5, 24.7), (120.51, 24.71), (120.52, 24.72)],
            category: "戶外運動".to_string(),
            activity: "步行".to_string(),
            year: "2026".to_string(),
            month: "2026-03".to_string(),
        }
    }

    #[test]
    fn test_duration_seconds() {
        let metadata = create_test_metadata();
        assert_eq!(metadata.duration_seconds(), 5400); // 1.5 小時 = 5400 秒
    }

    #[test]
    fn test_calculate_distance_multiple_points() {
        let metadata = create_test_metadata();
        let distance = metadata.calculate_distance();
        assert!(distance > 0.0);
        assert!(distance < 10000.0); // 鄰近座標，應小於 10 km
    }

    #[test]
    fn test_calculate_distance_single_point() {
        let mut metadata = create_test_metadata();
        metadata.coordinates = vec![(120.5, 24.7)];
        assert_eq!(metadata.calculate_distance(), 0.0);
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

### 完整的 KML 解析測試

```rust
// tests/kml_parsing.rs

use movement_tracks_analyzer::extract_placemarks_with_paths;
use std::path::{Path, PathBuf};

#[test]
fn test_parse_sample_kml_file() {
    let fixture_path = PathBuf::from("tests/fixtures/tracks.kml");

    // Arrange
    assert!(Path::new(&fixture_path).exists(), "測試夾具檔案應存在");

    // Act
    let result = extract_placemarks_with_paths(&fixture_path);

    // Assert
    assert!(result.is_ok(), "應成功解析 KML 檔案");
    let tracks = result.unwrap();
    assert!(!tracks.is_empty(), "應解析至少一個軌跡");
}

#[test]
fn test_track_metadata_completeness() {
    let fixture_path = PathBuf::from("tests/fixtures/tracks.kml");
    let tracks = extract_placemarks_with_paths(&fixture_path).expect("應成功解析");

    for (_path, metadata) in &tracks {
        // 驗證每個軌跡的完整性
        assert!(!metadata.name.is_empty(), "軌跡名稱不應為空");
        assert!(!metadata.coordinates.is_empty(), "應至少有一個座標點");
    }
}

#[test]
fn test_distance_calculation_non_negative() {
    let fixture_path = PathBuf::from("tests/fixtures/tracks.kml");
    let tracks = extract_placemarks_with_paths(&fixture_path).expect("應成功解析");

    for (_path, metadata) in &tracks {
        assert!(metadata.calculate_distance() >= 0.0, "距離應為非負數");
    }
}
```

---

## 執行測試

### 基本命令

```bash
# 執行所有測試
cargo test

# 執行單個測試函數
cargo test test_same_point_distance_zero

# 執行單個模組中的所有測試
cargo test path::tests

# 執行單個集成測試檔案的所有測試
cargo test --test kml_parsing
```

### 詳細輸出

```bash
# 顯示 println! 與日誌輸出
cargo test -- --nocapture

# 順序執行（預設為並行執行）
cargo test -- --test-threads=1

# 詳細列出每個測試的結果
cargo test -- --show-output
```

### 選擇性執行

```bash
# 執行名稱包含 "distance" 的所有測試
cargo test distance

# 執行單元測試（排除集成測試）
cargo test --lib

# 執行集成測試（排除單元測試）
cargo test --test '*'
```

---

## 測試駐留夾具（Fixtures）

### 使用 KML 測試檔案

測試夾具存放於 `tests/fixtures/`，例如 `tracks.kml`：

```rust
// 在測試中引用
#[test]
fn test_with_fixture() {
    use movement_tracks_analyzer::extract_placemarks_with_paths;
    use std::path::PathBuf;

    let fixture = PathBuf::from("tests/fixtures/tracks.kml");
    let result = extract_placemarks_with_paths(&fixture);
    assert!(result.is_ok());
}
```

### 夾具特點

- KML 檔案應代表真實的 GPS 軌跡格式
- 應包含各種邊界情況：空軌跡、單點軌跡、多軌跡檔案等
- 夾具檔案應 UTF-8 編碼

---

## 測試驅動開發（TDD）流程

當添加新功能時，應遵循：

1. **編寫測試**：定義預期行為
   ```rust
   #[test]
   fn test_new_feature() {
       let input = prepare_input();
       let result = new_function(input);
       assert_eq!(result, expected_output);
   }
   ```

2. **執行測試**：確認測試失敗（因功能尚未實現）
   ```bash
   cargo test test_new_feature
   ```

3. **實現功能**：撰寫最小可行實現
   ```rust
   fn new_function(input: Input) -> Output {
       // ...實現...
   }
   ```

4. **執行測試**：確認測試通過
   ```bash
   cargo test test_new_feature
   ```

5. **重構**：改進程式碼品質，測試應持續通過

---

## 測試覆蓋與最佳實踐

### 應包含的測試場景

- **正常流程**：合法輸入產生預期輸出
- **邊界情況**：空集合、單一元素、最大值等
- **錯誤情況**：無效輸入、缺失檔案、格式錯誤等
- **效能測試**：大型 KML 檔案不應耗時過久（參見 `PERFORMANCE.md`）

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
