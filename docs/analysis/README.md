# 認知複雜度分析

此目錄包含使用 SonarQube 認知複雜度算法對 Movement Tracks Analyzer 專案進行的全面分析。

## 📋 檔案清單

### 報告檔案

1. **COGNITIVE_COMPLEXITY_REPORT.md**
    - 完整詳細分析報告（532 行）
    - 所有 80 個函式的認知複雜度統計
    - 9 個模組的詳細評估
    - 最複雜函式的深度解析
    - 重構成效驗證

2. **COMPLEXITY_ANALYSIS_INDEX.md**
    - 快速導航索引（284 行）
    - 核心統計速查表
    - 常見問題解答
    - 後續行動建議

3. **COMPLEXITY_ANALYSIS_SUMMARY.md**
    - 快速參考摘要
    - 主要指標概覽

### 數據檔案

- **cognitive_complexity_data.json**
    - 結構化 JSON 格式數據（420 行）
    - 機器可讀，適合工具集成
    - 完整的品質分佈和模組統計

### 工具檔案

- **cognitive_complexity_analyzer.py**
    - 認知複雜度計算工具
    - 基於 SonarQube 算法實現
    - 可用於後續分析

### 分析檔案

- **BETTER_HIGHLIGHTS_CC_ANALYSIS.md**
    - Better Highlights vs SonarQube 複雜度計算差異分析
    - 原因分析與驗證假設

## 🎯 快速開始

1. **需要快速了解** → 閱讀 COMPLEXITY_ANALYSIS_SUMMARY.md
2. **需要完整分析** → 閱讀 COGNITIVE_COMPLEXITY_REPORT.md
3. **需要數據集成** → 使用 cognitive_complexity_data.json
4. **需要查詢特定信息** → 參考 COMPLEXITY_ANALYSIS_INDEX.md
5. **對 Better Highlights 差異感興趣** → 參考 BETTER_HIGHLIGHTS_CC_ANALYSIS.md

## 📊 核心統計

| 指標            | 數值  | 評級       |
| --------------- | ----- | ---------- |
| 總認知複雜度    | 110   | ✅          |
| 總函式數        | 80    | ✅          |
| 平均複雜度      | 1.38  | ✅ 業界優秀 |
| 優秀比例 (CC≤3) | 92.5% | ✅          |

## 📅 生成日期

2026-03-14

## 🔗 相關文件

- [../REFACTORING.md](../../REFACTORING.md) - 重構歷史
- [../ARCHITECTURE.md](../../ARCHITECTURE.md) - 架構設計
- [../PERFORMANCE.md](../../PERFORMANCE.md) - 效能優化
