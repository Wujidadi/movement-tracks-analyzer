# 工具庫

此目錄存放與專案相關的分析和輔助工具。

## 工具清單

### cognitive_complexity_analyzer
- **用途**：計算 Rust 源代碼的認知複雜度
- **算法**：Better Highlights 標準（完全相同）
- **輸出格式**：相對百分比（0-100%）
- **語言**：Rust（已編譯為二進位，無需 Python）

### 使用方式

**Rust 版本**（推薦）：
```bash
# 直接編譯並運行
cargo run --release --bin cognitive_complexity_analyzer -- src/converter.rs

# 或運行已編譯的二進位
./target/release/cognitive_complexity_analyzer src/converter.rs
```

**Python 版本**（備用）：
```bash
python3 tools/cognitive_complexity_analyzer.py src/converter.rs
```

## Better Highlights 算法詳解

此工具完全實現 Better Highlights 的認知複雜度計算算法：

| 構造             | 計分  | 備註                                        |
| ---------------- | ----- | ------------------------------------------- |
| `if`             | +1    | 純 if 語句                                  |
| `if let`         | +1    | Rust 模式匹配分支                           |
| `let-else`       | 0     | 守衛語句（不計複雜度）                      |
| `else if`        | +1    | 視為新分支                                  |
| `else`           | +1    | 單獨的 else 分支（不包含 let-else 的 else） |
| `match`          | +1    | match 本身計 1，不計分支數                  |
| `for/while/loop` | +1    | 每個循環                                    |
| `&&` / `\|\|`    | +1    | 每個邏輯運算符                              |
| 巢狀修正         | +1/層 | 超過 1 層縮進的分支                         |

### 計算基準

複雜度使用相對百分比表示，其中：
- **基準上限**：15（預期的最大合理複雜度）
- **百分比公式**：(CC / 15) × 100
- **四舍五入**：使用 `round()` 方法
- **範圍**：0-100%

### 主要特性

- ✅ 計入 `else` 為獨立分支（與 SonarQube 不同）
- ✅ `match` 本身計為 1，不計分支數（簡化複雜度）
- ✅ `?` 運算符不計複雜度（Rust 風格）
- ✅ 自動函式提取和分析
- ✅ 相對百分比輸出

### 使用方式

**Rust 版本**（推薦，編譯為二進位，無依賴）：
```bash
# 直接編譯並運行
cargo run --release --bin cognitive_complexity_analyzer -- src/converter.rs

# 或運行已編譯的二進位
./target/release/cognitive_complexity_analyzer src/converter.rs
```

**Python 版本**（備用）：
```bash
python3 tools/cognitive_complexity_analyzer.py src/converter.rs
```

### 版本說明

| 版本   | 優點                                   | 備註     |
| ------ | -------------------------------------- | -------- |
| Rust   | 編譯為二進位，無依賴，運行快，計算準確 | 推薦使用 |
| Python | 易於修改和調試                         | 備用     |

### 計算準確性

- ✅ **Python 版本和 Rust 版本計算結果完全一致**
- ✅ 結果與 Better Highlights 的複雜度計算算法對齊
- ✅ 正確排除了方法調用如 `.or_else()`、`.ok_or_else()` 的誤識別
- ✅ 正確排除了 `?` 運算符的複雜度計算（Rust 特性）
- ✅ 對長函式名稱的自動截斷和格式化
