/// 從 XML 資料夾路徑中提取軌跡分類資訊
///
/// 根據資料夾層級深度智能識別分類、活動、年份和月份。
///
/// # Arguments
///
/// * `folder_path` - XML 路徑中的資料夾名稱陣列（由上而下）
///
/// # Returns
///
/// 包含四個元素的元組：`(category, activity, year, month)`
/// - `category` (分類)：如「戶外運動」、「動力交通工具」
/// - `activity` (活動)：如「步行」、「飛機」
/// - `year` (年份)：如「2025」
/// - `month` (月份)：如「2025-03」
///
/// 若資訊不足，對應欄位為空字符串。
///
/// # Example
///
/// ```rust
/// use movement_tracks_analyzer::extract_categories;
///
/// let path = vec![
///     "移動軌跡".to_string(),
///     "戶外運動".to_string(),
///     "步行".to_string(),
///     "2025".to_string(),
///     "2025-03".to_string(),
/// ];
///
/// let (category, activity, year, month) = extract_categories(&path);
/// assert_eq!(category, "戶外運動");
/// assert_eq!(activity, "步行");
/// assert_eq!(year, "2025");
/// assert_eq!(month, "2025-03");
/// ```
pub fn extract_categories(folder_path: &[String]) -> (String, String, String, String) {
    let meaningful_path: Vec<&String> = folder_path
        .iter()
        .filter(|name| !name.contains("(Example)") && !name.contains("Movement Tracks"))
        .collect();

    match meaningful_path.len() {
        0 => empty_tuple(),
        1 => extract_single_element(&meaningful_path),
        2 => create_category_tuple(None, None, Some(0), Some(1), &meaningful_path),
        3 => create_category_tuple(None, Some(0), Some(1), Some(2), &meaningful_path),
        _ => {
            let len = meaningful_path.len();
            create_category_tuple(
                Some(len - 4),
                Some(len - 3),
                Some(len - 2),
                Some(len - 1),
                &meaningful_path,
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

/// 提取單個路徑元素（判斷是年份還是月份）
fn extract_single_element(path: &[&String]) -> (String, String, String, String) {
    let elem = path[0];
    if elem.contains('-') && elem.len() == 7 {
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
