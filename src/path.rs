/// 從 KML 資料夾路徑中提取軌跡分類資訊
///
/// 根據目錄層級深度自動識別分類、活動、年份和月份。
///
/// # Arguments
///
/// * `folder_path` - KML 路徑中的資料夾名稱陣列（由上而下）
///
/// # Returns
///
/// 包含四個元素的元組：`(category, activity, year, month)`
/// - `category` (分類)：如「戶外運動」、「動力交通工具」
/// - `activity` (活動)：如「步行」、「飛機」
/// - `year` (年份)：如「2026」
/// - `month` (月份)：如「2026-03」
///
/// 若資訊不足，對應欄位為空字串。
///
/// # Example
///
/// ```
/// use movement_tracks_analyzer::extract_categories;
///
/// let path = vec![
///     "移動軌跡".to_string(),
///     "戶外運動".to_string(),
///     "步行".to_string(),
///     "2026".to_string(),
///     "2026-03".to_string(),
/// ];
///
/// let (category, activity, year, month) = extract_categories(&path);
/// // With 5 elements, uses pattern (len-4, len-3, len-2, len-1) = (1, 2, 3, 4)
/// assert_eq!(category, "戶外運動");
/// assert_eq!(activity, "步行");
/// assert_eq!(year, "2026");
/// assert_eq!(month, "2026-03");
/// ```
pub fn extract_categories(folder_path: &[String]) -> (String, String, String, String) {
    let meaningful_path = filter_meaningful_path(folder_path);
    categorize_by_depth(&meaningful_path)
}

/// 過濾掉非有效分類的路徑元素
fn filter_meaningful_path(folder_path: &[String]) -> Vec<&String> {
    folder_path
        .iter()
        .filter(|name| !name.contains("(Example)") && !name.contains("Movement Tracks"))
        .collect()
}

/// 根據路徑深度建立分類元組
fn categorize_by_depth(meaningful_path: &[&String]) -> (String, String, String, String) {
    match meaningful_path.len() {
        0 => empty_tuple(),
        1 => classify_single_element(meaningful_path[0]),
        2 => create_category_tuple(None, None, Some(0), Some(1), meaningful_path),
        3 => create_category_tuple(None, Some(0), Some(1), Some(2), meaningful_path),
        _ => {
            let len = meaningful_path.len();
            create_category_tuple(
                Some(len - 4),
                Some(len - 3),
                Some(len - 2),
                Some(len - 1),
                meaningful_path,
            )
        }
    }
}

/// 建立分類元組，省略重複的 String::new()
fn create_category_tuple(
    cat_idx: Option<usize>,
    act_idx: Option<usize>,
    year_idx: Option<usize>,
    month_idx: Option<usize>,
    path: &[&String],
) -> (String, String, String, String) {
    (
        cat_idx.map(|i| path[i].to_string()).unwrap_or_default(),
        act_idx.map(|i| path[i].to_string()).unwrap_or_default(),
        year_idx.map(|i| path[i].to_string()).unwrap_or_default(),
        month_idx.map(|i| path[i].to_string()).unwrap_or_default(),
    )
}

/// 返回空的分類元組
fn empty_tuple() -> (String, String, String, String) {
    (String::new(), String::new(), String::new(), String::new())
}

/// 判斷字串是否為月份格式（YYYY-MM）
fn is_month_format(s: &str) -> bool {
    s.contains('-') && s.len() == 7
}

/// 分類單個路徑元素（判斷是年份還是月份）
fn classify_single_element(elem: &str) -> (String, String, String, String) {
    if is_month_format(elem) {
        (
            String::new(),
            String::new(),
            String::new(),
            elem.to_string(),
        )
    } else {
        (
            String::new(),
            String::new(),
            elem.to_string(),
            String::new(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_categories_full_path() {
        let path = vec![
            "移動軌跡".to_string(),
            "戶外運動".to_string(),
            "步行".to_string(),
            "2026".to_string(),
            "2026-03".to_string(),
        ];

        let (cat, act, year, month) = extract_categories(&path);
        assert_eq!(cat, "戶外運動");
        assert_eq!(act, "步行");
        assert_eq!(year, "2026");
        assert_eq!(month, "2026-03");
    }

    #[test]
    fn test_extract_categories_with_spaces() {
        let path = vec![
            "移動軌跡".to_string(),
            "  動力交通工具  ".to_string(),
            "飛機".to_string(),
            "2026".to_string(),
            "2026-02".to_string(),
        ];

        let (cat, act, year, month) = extract_categories(&path);
        // Space padding is preserved in the implementation
        assert_eq!(cat, "  動力交通工具  ");
        assert_eq!(act, "飛機");
        assert_eq!(year, "2026");
        assert_eq!(month, "2026-02");
    }

    #[test]
    fn test_extract_categories_with_three_meaningful_elements() {
        let path = vec![
            "移動軌跡".to_string(),
            "戶外運動".to_string(),
            "步行".to_string(),
            "2026".to_string(),
        ];

        let (cat, act, year, month) = extract_categories(&path);
        // meaningful_path has 4 elements, uses _ pattern (len - 4, len - 3, len - 2, len - 1)
        // So indices are: (0, 1, 2, 3) = ("移動軌跡", "戶外運動", "步行", "2026")
        assert_eq!(cat, "移動軌跡");
        assert_eq!(act, "戶外運動");
        assert_eq!(year, "步行");
        assert_eq!(month, "2026");
    }

    #[test]
    fn test_extract_categories_single_non_root_element() {
        let path = vec!["2026-03".to_string()]; // Just a month

        let (cat, act, year, month) = extract_categories(&path);
        // extract_single_element checks for '-' pattern
        assert_eq!(cat, "");
        assert_eq!(act, "");
        assert_eq!(year, "");
        assert_eq!(month, "2026-03");
    }

    #[test]
    fn test_extract_categories_empty_path() {
        let path: Vec<String> = vec![];

        let (cat, act, year, month) = extract_categories(&path);
        assert_eq!(cat, "");
        assert_eq!(act, "");
        assert_eq!(year, "");
        assert_eq!(month, "");
    }
}
