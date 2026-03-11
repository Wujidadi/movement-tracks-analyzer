use once_cell::sync::Lazy;
use regex::Regex;

const DATETIME_PATTERN: &str = r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})";

fn create_time_pattern(label: &str, has_br: bool) -> String {
    let br = if has_br { r"<br />" } else { "" };
    format!(r"<b>\s*{}\s*:\s*</b>\s*{}{}", label, DATETIME_PATTERN, br)
}

pub static START_TIME_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&create_time_pattern("Start", true)).unwrap()
});

pub static END_TIME_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&create_time_pattern("End", false)).unwrap()
});
