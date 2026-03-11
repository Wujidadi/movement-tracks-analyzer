/// 從路徑中提取分類、活動、年度、月份
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
