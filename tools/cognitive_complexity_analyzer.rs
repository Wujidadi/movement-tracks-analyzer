use std::env;
use std::fs;
use std::path::Path;
use std::process;

/// Better Highlights 認知複雜度分析器
struct BetterHighlightsCCAnalyzer;

impl BetterHighlightsCCAnalyzer {
    /// 分析函式的認知複雜度
    fn analyze_function(func_code: &str, func_name: &str) -> AnalysisResult {
        let mut complexity = 0;

        // 只分析函式體部分（{ 到 } 之間）
        let func_body = Self::extract_function_body(func_code);

        for line in func_body.lines() {
            // 移除註解
            let clean_line = if let Some(pos) = line.find("//") {
                &line[..pos]
            } else {
                line
            };
            let clean_line = clean_line.trim();

            if clean_line.is_empty() {
                continue;
            }

            // 如果行包含 ? 運算符，跳過複雜度計算（因為 ? 不計複雜度）
            if clean_line.contains("?") {
                continue;
            }

            // 計算巢狀深度
            let indent_level = line.len() - line.trim_start().len();
            let nesting_depth = indent_level / 4; // 假設 4 空格縮進

            let base_inc = 1 + nesting_depth.saturating_sub(1);

            // if let（計入為分支）
            if clean_line.contains("if let") {
                complexity += base_inc;
            }
            // else if
            else if clean_line.contains("else if") {
                complexity += base_inc;
            }
            // if 語句（不包含 else if）
            else if clean_line.contains("if ") && !clean_line.contains("else") {
                complexity += base_inc;
            }
            // else（不包含 let-else 的 else 或 else if，且排除方法調用如 .or_else()、.ok()、.get() 等）
            else if clean_line.contains("else")
                && !clean_line.contains("if")
                && !clean_line.contains("let")
                && !clean_line.contains(".")  // 排除所有方法調用（.or_else、.ok、.get 等）
                {
                complexity += base_inc;
            }
            // match
            else if clean_line.contains("match ") {
                complexity += base_inc;
            }

            // 循環
            if clean_line.contains("for ") || clean_line.contains("while ") || clean_line.contains("loop ") {
                complexity += base_inc;
            }

            // 邏輯運算符（但排除在 ? 運算符之後的情況）
            // 只計獨立的邏輯運算符，不計在方法鏈中的
            if !clean_line.contains("?") {
                let and_count = clean_line.matches("&&").count();
                let or_count = clean_line.matches("||").count();
                complexity += and_count + or_count;
            }
        }

        // 計算百分比
        let percentage = ((complexity as f32 / 15.0) * 100.0).round() as u32;
        let percentage = percentage.min(100);

        let label = Self::get_label(percentage);

        AnalysisResult {
            name: func_name.to_string(),
            cc: complexity,
            percentage,
            label,
        }
    }

    /// 根據百分比返回標籤
    fn get_label(percentage: u32) -> String {
        match percentage {
            0 => "極優".to_string(),
            1..=30 => "優秀".to_string(),
            31..=70 => "中等".to_string(),
            71..=90 => "複雜".to_string(),
            _ => "非常複雜".to_string(),
        }
    }

    /// 從函式代碼中提取只包含 { 到 } 的函式體（不含簽名）
    fn extract_function_body(func_code: &str) -> String {
        let chars: Vec<char> = func_code.chars().collect();
        let mut brace_start = None;
        let mut brace_count = 0;

        // 找到第一個 {
        for (i, &ch) in chars.iter().enumerate() {
            if ch == '{' {
                brace_start = Some(i);
                brace_count = 1;
                break;
            }
        }

        if let Some(start) = brace_start {
            // 找到配對的 }
            let mut pos = start + 1;
            while pos < chars.len() && brace_count > 0 {
                if chars[pos] == '{' {
                    brace_count += 1;
                } else if chars[pos] == '}' {
                    brace_count -= 1;
                }
                pos += 1;
            }

            // 返回 { 之後、第一個實際程式碼行開始
            // 找到 { 之後的第一個新行
            let mut body_start = start + 1;
            while body_start < chars.len() && chars[body_start] != '\n' {
                body_start += 1;
            }
            if body_start < chars.len() && chars[body_start] == '\n' {
                body_start += 1;
            }

            let body_end = if brace_count == 0 { pos - 1 } else { chars.len() };

            chars[body_start..body_end].iter().collect()
        } else {
            func_code.to_string()
        }
    }
}

#[derive(Debug)]
struct AnalysisResult {
    name: String,
    cc: usize,
    percentage: u32,
    label: String,
}

/// 從 Rust 檔案中提取所有函式
fn extract_functions_from_file(filepath: &str) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(filepath)?;
    let mut functions = Vec::new();

    // 逐字元掃描找出所有 fn 定義
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // 尋找 "fn " 關鍵字
        if i + 3 <= chars.len()
            && chars[i] == 'f'
            && chars[i + 1] == 'n'
            && chars[i + 2] == ' ' {

            // 找出函式名稱開始位置
            let mut name_start = i + 3;
            while name_start < chars.len() && chars[name_start].is_whitespace() {
                name_start += 1;
            }

            // 提取函式名稱
            let mut name_end = name_start;
            while name_end < chars.len() && (chars[name_end].is_alphanumeric() || chars[name_end] == '_') {
                name_end += 1;
            }

            if name_start >= name_end {
                i += 1;
                continue;
            }

            let func_name: String = chars[name_start..name_end].iter().collect();

            // 尋找開始的 { （跳過簽名部分）
            let mut brace_start = name_end;
            let mut paren_depth = 0;
            let mut angle_depth = 0;

            while brace_start < chars.len() {
                match chars[brace_start] {
                    '(' => paren_depth += 1,
                    ')' => {
                        paren_depth -= 1;
                        // 簽名結束，現在掃描返回型別和 where 子句
                        brace_start += 1;
                        // 跳過返回型別和 where 子句
                        while brace_start < chars.len() && chars[brace_start] != '{' {
                            brace_start += 1;
                        }
                        break;
                    }
                    '<' if paren_depth == 0 => angle_depth += 1,
                    '>' if paren_depth == 0 => angle_depth -= 1,
                    '{' if paren_depth == 0 && angle_depth == 0 => break,
                    _ => {}
                }
                brace_start += 1;
            }

            if brace_start >= chars.len() || chars[brace_start] != '{' {
                i += 1;
                continue;
            }

            // 尋找配對的 }
            let mut brace_count = 0;
            let mut brace_end = brace_start;

            while brace_end < chars.len() {
                if chars[brace_end] == '{' {
                    brace_count += 1;
                } else if chars[brace_end] == '}' {
                    brace_count -= 1;
                    if brace_count == 0 {
                        brace_end += 1;
                        break;
                    }
                }
                brace_end += 1;
            }

            if brace_count == 0 && brace_end > brace_start {
                let func_code: String = chars[i..brace_end].iter().collect();
                functions.push((func_name, func_code));
            }

            i = brace_end;
        } else {
            i += 1;
        }
    }

    Ok(functions)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("使用方式: cognitive_complexity_analyzer <source_file>");
        process::exit(1);
    }

    let filepath = &args[1];

    if !Path::new(filepath).exists() {
        eprintln!("❌ 檔案不存在: {}", filepath);
        process::exit(1);
    }

    match extract_functions_from_file(filepath) {
        Ok(functions) => {
            if functions.is_empty() {
                eprintln!("❌ 未找到任何函式");
                process::exit(1);
            }

            println!("\n📊 {} - 認知複雜度分析（Better Highlights 標準）", filepath);

            // 計算所需的最大列寬
            let max_name_len = functions.iter().map(|(name, _)| name.len()).max().unwrap_or(20);
            let name_width = std::cmp::max(20, std::cmp::min(max_name_len + 2, 60)); // 最少 20，最多 60
            let header_width = name_width + 25;

            println!("{}", "=".repeat(header_width));
            println!("{:<name_width$} {:>3} {:>5} {:<8}", "函式名稱", "CC", "百分比", "評級");
            println!("{}", "-".repeat(header_width));

            let mut total_percentage = 0u32;

            for (func_name, func_code) in &functions {
                let result =
                    BetterHighlightsCCAnalyzer::analyze_function(func_code, func_name);

                total_percentage += result.percentage;

                // 截斷過長的函式名
                let display_name = if result.name.len() > name_width - 2 {
                    format!("{}...", &result.name[..std::cmp::max(1, name_width - 5)])
                } else {
                    result.name.clone()
                };

                println!(
                    "{:<name_width$} {:>3}  {:>3}%  {:<8}",
                    display_name, result.cc, result.percentage, result.label
                );
            }

            if !functions.is_empty() {
                let avg_percentage = total_percentage / functions.len() as u32;
                println!("{}", "-".repeat(header_width));
                println!(
                    "{:<name_width$} {:>3} {:>3}%  {:<8}",
                    "平均複雜度",
                    "",
                    avg_percentage,
                    BetterHighlightsCCAnalyzer::get_label(avg_percentage)
                );
                println!("{}", "=".repeat(header_width));
            }
        }
        Err(e) => {
            eprintln!("❌ 錯誤: {}", e);
            process::exit(1);
        }
    }
}
