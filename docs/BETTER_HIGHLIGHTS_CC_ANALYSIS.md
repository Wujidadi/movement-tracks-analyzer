# Better Highlights 複雜度計算差異分析

## 觀察

**Better Highlights 報告**：
- `main()` 函式：CC = 7%
- `run()` 函式：CC = 0%

**SonarQube 標準報告**：
- `main()` 函式：CC = 0（純粹的錯誤處理包裝）
- `run()` 函式：CC = 4（4 個順序操作）

## 原始代碼分析

```rust
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> movement_tracks_analyzer::Result<()> {
    let args = Args::parse();
    let config = build_config(args)?;
    let placemarks = extract_placemarks_with_paths(&config.kml_file)?;
    output_results(&placemarks, &config)
}
```

## 複雜度差異原因

### 1. `main()` 函式的複雜度差異

**Better Highlights (7%)**：
- 計入 `if let Err(e)` 分支：+1
- 可能將 `eprintln!` 和 `process::exit(1)` 視為額外的複雜性
- 可能用相對百分比表示（7% 相對於基準）

**SonarQube 標準 (0)**：
- `if let` 在標準 SonarQube 中不計複雜度（只有 `if` 和 `else if` 計算）
- `if let` 被視為單純的模式匹配，非控制流分支

**結論**：差異在於對 `if let` 的計算方式不同

### 2. `run()` 函式的複雜度差異

**Better Highlights (0%)**：
- 可能不計入順序操作（parse、build_config、extract、output）
- 將函式視為線性流程，無分支

**SonarQube 標準 (4)**：
- 計入 `?` 運算符的複雜度（錯誤傳播）
- 或將 4 行操作視為 4 個邏輯步驟

**結論**：Better Highlights 可能不計入 `?` 運算符

---

## Better Highlights 算法推測

根據觀察，Better Highlights 似乎採用了**簡化的複雜度計算**：

| 構造             | SonarQube        | Better Highlights |
| ---------------- | ---------------- | ----------------- |
| `if` / `else if` | +1               | +1                |
| `if let`         | 0                | +1（可能）        |
| `match`          | +1/分支          | +1/分支           |
| `?` 運算符       | +0（或視上下文） | 0（不計）         |
| 順序操作         | 0                | 0                 |
| `&&` / `         |                  | `                 | +1 | +1 |

---

## 驗證假設

### 假設 A：Better Highlights 計入 `if let`

```rust
fn main() {
    if let Err(e) = run() {  // +1（if let 視為分支）
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
```

**複雜度** = 1 → 可能顯示為 "7%"（相對基準）

### 假設 B：Better Highlights 不計入 `?` 運算符

```rust
fn run() -> movement_tracks_analyzer::Result<()> {
    let args = Args::parse();           // 無複雜度
    let config = build_config(args)?;   // ? 不計複雜度
    let placemarks = extract_placemarks_with_paths(&config.kml_file)?;  // ? 不計
    output_results(&placemarks, &config) // 無複雜度
}
```

**複雜度** = 0 → 顯示為 "0%"

---

## 結論

Better Highlights 與 SonarQube 的差異：

| 因素              | 說明                                                           |
| ----------------- | -------------------------------------------------------------- |
| **`if let` 計算** | Better Highlights 可能視 `if let` 為分支（+1），SonarQube 不計 |
| **`?` 運算符**    | Better Highlights 不計，SonarQube 視上下文計算                 |
| **百分比表示**    | Better Highlights 用相對百分比，不是絕對 CC 值                 |
| **線性流程**      | Better Highlights 對順序操作複雜度計算為 0                     |

---

## 建議

1. **確認 Better Highlights 的官方文檔** - 了解其確切計算方法
2. **根據專案需求選擇標準**：
    - 如果強調代碼簡潔性 → 使用 Better Highlights 的簡化標準
    - 如果遵循業界標準 → 使用 SonarQube 標準
3. **統一團隊規範** - 決定使用哪一套複雜度計算方法

---

**生成日期**：2026-03-14  
**分析對象**：Better Highlights vs SonarQube 複雜度計算差異  
**狀態**：基於觀察推測
