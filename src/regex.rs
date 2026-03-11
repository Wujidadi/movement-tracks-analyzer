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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_time_pattern_matches() {
        let html = r#"<b>Start: </b>2025-03-11 10:30:45<br />"#;
        assert!(START_TIME_PATTERN.is_match(html));
    }

    #[test]
    fn test_start_time_pattern_captures() {
        let html = r#"<b>Start: </b>2025-03-11 10:30:45<br />"#;
        let caps = START_TIME_PATTERN.captures(html);
        assert!(caps.is_some());
        let cap = caps.unwrap();
        assert_eq!(cap.get(1).unwrap().as_str(), "2025-03-11 10:30:45");
    }

    #[test]
    fn test_start_time_pattern_with_spaces() {
        let html = r#"<b>  Start  : </b>2025-03-11 10:30:45<br />"#;
        assert!(START_TIME_PATTERN.is_match(html));
    }

    #[test]
    fn test_end_time_pattern_matches() {
        let html = r#"<b>End: </b>2025-03-11 11:30:45"#;
        assert!(END_TIME_PATTERN.is_match(html));
    }

    #[test]
    fn test_end_time_pattern_captures() {
        let html = r#"<b>End: </b>2025-03-11 11:30:45"#;
        let caps = END_TIME_PATTERN.captures(html);
        assert!(caps.is_some());
        let cap = caps.unwrap();
        assert_eq!(cap.get(1).unwrap().as_str(), "2025-03-11 11:30:45");
    }

    #[test]
    fn test_end_time_pattern_without_br() {
        let html = r#"<b>End: </b>2025-03-11 11:30:45"#;
        assert!(END_TIME_PATTERN.is_match(html));

        // Note: END_TIME_PATTERN allows optional <br />, so it can match both
        let html_with_br = r#"<b>End: </b>2025-03-11 11:30:45<br />"#;
        // This will match because <br /> is not required
        assert!(END_TIME_PATTERN.is_match(html_with_br));
    }

    #[test]
    fn test_both_patterns_in_html() {
        let html = r#"<b>Start: </b>2025-03-11 10:30:45<br />
Some content
<b>End: </b>2025-03-11 11:30:45"#;
        assert!(START_TIME_PATTERN.is_match(html));
        assert!(END_TIME_PATTERN.is_match(html));
    }
}
