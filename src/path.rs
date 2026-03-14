/// 從 KML 資料夾路徑中提取軌跡分類資訊
///
/// 根據目錄層級深度自動識別分類、活動、年份和月份。
/// 先以 `root_name`（來自檔案名稱）過濾掉根節點，再依剩餘深度進行分類。
///
/// # Arguments
///
/// * `folder_path` - KML 路徑中的資料夾名稱陣列（由上而下）
/// * `root_name` - KML/KMZ 檔案名稱（不含副檔名），用於過濾根節點
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
/// // 根節點名稱 "移動軌跡" 與檔名一致，會被過濾
/// let (category, activity, year, month) = extract_categories(&path, "移動軌跡");
/// assert_eq!(category, "戶外運動");
/// assert_eq!(activity, "步行");
/// assert_eq!(year, "2026");
/// assert_eq!(month, "2026-03");
/// ```
pub fn extract_categories(folder_path: &[String], root_name: &str) -> (String, String, String, String) {
    let meaningful_path = filter_meaningful_path(folder_path, root_name);
    categorize_by_depth(&meaningful_path)
}

/// 過濾掉根節點名稱，僅保留有效分類的路徑元素
fn filter_meaningful_path<'a>(folder_path: &'a [String], root_name: &str) -> Vec<&'a String> {
    folder_path
        .iter()
        .filter(|name| name.as_str() != root_name)
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

        let (cat, act, year, month) = extract_categories(&path, "移動軌跡");
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

        let (cat, act, year, month) = extract_categories(&path, "移動軌跡");
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

        let (cat, act, year, month) = extract_categories(&path, "移動軌跡");
        // 過濾 "移動軌跡" 後剩 3 個元素: ["戶外運動", "步行", "2026"]
        // 對應 pattern 3: (None, 0, 1, 2)
        assert_eq!(cat, "");
        assert_eq!(act, "戶外運動");
        assert_eq!(year, "步行");
        assert_eq!(month, "2026");
    }

    #[test]
    fn test_extract_categories_single_non_root_element() {
        let path = vec!["2026-03".to_string()];

        let (cat, act, year, month) = extract_categories(&path, "some_file");
        assert_eq!(cat, "");
        assert_eq!(act, "");
        assert_eq!(year, "");
        assert_eq!(month, "2026-03");
    }

    #[test]
    fn test_extract_categories_empty_path() {
        let path: Vec<String> = vec![];

        let (cat, act, year, month) = extract_categories(&path, "some_file");
        assert_eq!(cat, "");
        assert_eq!(act, "");
        assert_eq!(year, "");
        assert_eq!(month, "");
    }
}
