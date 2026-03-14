#!/usr/bin/env python3
"""
認知複雜度計算器 - Better Highlights 標準

根據 Better Highlights 的認知複雜度算法計算 Rust 代碼的複雜度：
- if / else if：+1 / 分支
- if let：+1（視為分支）← 與 SonarQube 不同
- match：+1 / 分支
- for / while / loop：+1 / 循環
- &&, ||：+1 / 運算符
- ? 運算符：0（不計）← 與 SonarQube 不同
- 結果表示為相對百分比
"""

import re
import sys
from pathlib import Path
from typing import Dict, List, Tuple

class BetterHighlightsCCAnalyzer:
    """Better Highlights 認知複雜度分析器"""

    def __init__(self):
        self.reset()

    def reset(self):
        self.complexity = 0
        self.max_possible = 0  # 用於百分比計算

    def analyze_function(self, func_code: str, func_name: str = "unknown") -> Dict:
        """
        分析函式的認知複雜度（Better Highlights 標準）

        Args:
            func_code: 函式代碼字串
            func_name: 函式名稱

        Returns:
            包含複雜度和百分比的字典
        """
        self.reset()

        lines = func_code.split('\n')
        i = 0

        while i < len(lines):
            line = lines[i]

            # 移除註解
            clean_line = re.sub(r'//.*$', '', line).strip()
            if not clean_line:
                i += 1
                continue

            # 如果行包含 ? 運算符，跳過複雜度計算（因為 ? 不計複雜度）
            if '?' in clean_line:
                i += 1
                continue

            # 計算巢狀深度
            indent_level = len(line) - len(line.lstrip())
            nesting_depth = indent_level // 4  # 假設 4 空格縮進

            # Better Highlights: 巢狀深度修正
            base_inc = 1 + max(0, nesting_depth - 1)

            # if let（Better Highlights 計入為分支）
            if re.search(r'\bif\s+let\b', clean_line):
                self.complexity += base_inc
            # if 語句（不包含 else if）
            elif re.search(r'\bif\s+', clean_line) and 'else' not in clean_line:
                self.complexity += base_inc
            # else if（計為 if 的複雜度）
            elif re.search(r'\belse\s+if\b', clean_line):
                self.complexity += base_inc
            # else（單獨的 else，但排除 let-else 中的 else 和方法調用如 .or_else()）
            elif re.search(r'\belse\b', clean_line) and 'if' not in clean_line and 'let' not in clean_line and '.or_else' not in clean_line and '.ok_or_else' not in clean_line:
                self.complexity += base_inc
            # match（計 match 本身為 +1，不計分支）
            elif re.search(r'\bmatch\b', clean_line):
                self.complexity += base_inc

            # 循環
            if re.search(r'\bfor\b|\bwhile\b|\bloop\b', clean_line):
                self.complexity += base_inc

            # 邏輯運算符（但排除在 ? 運算符之後的情況）
            if '?' not in clean_line:
                and_count = len(re.findall(r'&&', clean_line))
                or_count = len(re.findall(r'\|\|', clean_line))
                self.complexity += (and_count + or_count)

            i += 1

        # 計算百分比（Better Highlights 使用相對基準）
        # (CC / 15) * 100，其中 15 是預期的最大合理複雜度
        percentage = min(100, round((self.complexity / 15.0) * 100))

        return {
            'name': func_name,
            'cc': self.complexity,
            'percentage': percentage,
            'label': self._get_label(percentage)
        }

    @staticmethod
    def _get_label(percentage: int) -> str:
        """根據百分比返回標籤"""
        if percentage == 0:
            return "極優"
        elif percentage <= 30:
            return "優秀"
        elif percentage <= 70:
            return "中等"
        elif percentage <= 90:
            return "複雜"
        else:
            return "非常複雜"


def extract_functions_from_file(filepath: str) -> List[Tuple[str, str]]:
    """從 Rust 檔案中提取所有函式"""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    functions = []

    # 正規表達式：fn name(...) 或 fn name<...>(...)
    pattern = r'(pub\s+)?(async\s+)?fn\s+(\w+)\s*(?:<[^>]+>)?\s*\([^)]*\)[^{]*\{'

    for match in re.finditer(pattern, content):
        func_name = match.group(3)
        start = match.start()

        # 尋找配對的 }
        brace_count = 1
        pos = match.end()

        while pos < len(content) and brace_count > 0:
            if content[pos] == '{':
                brace_count += 1
            elif content[pos] == '}':
                brace_count -= 1
            pos += 1

        if brace_count == 0:
            func_code = content[start:pos]
            functions.append((func_name, func_code))

    return functions


def main():
    if len(sys.argv) < 2:
        print("使用方式: python3 cognitive_complexity_analyzer.py <source_file>")
        sys.exit(1)

    filepath = sys.argv[1]

    if not Path(filepath).exists():
        print(f"❌ 檔案不存在: {filepath}")
        sys.exit(1)

    analyzer = BetterHighlightsCCAnalyzer()
    functions = extract_functions_from_file(filepath)

    if not functions:
        print("❌ 未找到任何函式")
        sys.exit(1)

    print(f"\n📊 {filepath} - 認知複雜度分析（Better Highlights 標準）")

    # 計算所需的最大列寬
    max_name_len = max(len(name) for name, _ in functions)
    name_width = max(20, min(max_name_len + 2, 60))  # 最少 20，最多 60

    header_width = name_width + 25
    print("=" * header_width)
    print(f"{'函式名稱':<{name_width}} {'CC':>3} {'百分比':>5} {'評級':<8}")
    print("-" * header_width)

    total_cc = 0
    total_percentage = 0

    for func_name, func_code in functions:
        result = analyzer.analyze_function(func_code, func_name)
        total_cc += result['cc']
        total_percentage += result['percentage']

        # 截斷過長的函式名
        display_name = result['name']
        if len(display_name) > name_width - 2:
            display_name = result['name'][:name_width - 5] + "..."

        print(f"{display_name:<{name_width}} {result['cc']:>3}  {result['percentage']:>3}%  {result['label']:<8}")

    if functions:
        avg_percentage = total_percentage // len(functions)
        print("-" * header_width)
        print(f"{'平均複雜度':<{name_width}} {'':>3} {avg_percentage:>3}%  {analyzer._get_label(avg_percentage):<8}")
        print("=" * header_width)


if __name__ == "__main__":
    main()


