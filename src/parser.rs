use crate::{extract_categories, Result, TrackMetadata, END_TIME_PATTERN, START_TIME_PATTERN};
use chrono::NaiveDateTime;
use quick_xml::{events::Event, Reader};
use std::{fs, io::BufReader, path::PathBuf};

/// 從 KML Description 中提取開始和結束時間
fn extract_times(description: &str) -> Option<(NaiveDateTime, NaiveDateTime)> {
    let start_match = START_TIME_PATTERN.captures(description)?;
    let end_match = END_TIME_PATTERN.captures(description)?;

    let start_str = start_match.get(1)?.as_str();
    let end_str = end_match.get(1)?.as_str();

    let start = NaiveDateTime::parse_from_str(start_str, "%Y-%m-%d %H:%M:%S").ok()?;
    let end = NaiveDateTime::parse_from_str(end_str, "%Y-%m-%d %H:%M:%S").ok()?;

    Some((start, end))
}

/// 從 KML 檔案中提取所有 Placemark 軌跡點
///
/// 使用流式 XML 解析器只掃描檔案一次，自動追蹤 XML 層級以提取軌跡分類資訊。
///
/// # Arguments
///
/// * `file_path` - KML 檔案的路徑
///
/// # Returns
///
/// 成功時返回 `Vec<(Vec<String>, TrackMetadata)>`，其中：
/// - `Vec<String>` 軌跡路徑（分類、活動、年份、月份）
/// - `TrackMetadata` 軌跡詳細資訊（名稱、時間、座標等）
///
/// # Errors
///
/// 若檔案不存在或 KML 格式無效，返回 `AnalyzerError`
///
/// # Performance
///
/// - 時間複雜度：O(n)（單次掃描）
/// - 空間複雜度：O(m)（m = Placemarks 數量）
/// - 適合處理大型檔案（50MB+）
///
/// # Example
///
/// ```ignore
/// use movement_tracks_analyzer::extract_placemarks_with_paths;
/// use std::path::PathBuf;
///
/// let placemarks = extract_placemarks_with_paths(&PathBuf::from("tracks.kml"))?;
/// for (path, metadata) in placemarks {
///     println!("{}: {} -> {}", metadata.name, path.join("/"), metadata.calculate_distance());
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn extract_placemarks_with_paths(
    file_path: &PathBuf,
) -> Result<Vec<(Vec<String>, TrackMetadata)>> {
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut xml_reader = Reader::from_reader(reader);

    let mut results = Vec::new();
    let mut buf = Vec::new();
    let mut folder_stack: Vec<String> = Vec::new();
    let mut parser_state = ParserState::default();

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(elem)) => {
                let tag_name = String::from_utf8_lossy(elem.name().as_ref()).to_string();
                handle_start_tag(&tag_name, &mut folder_stack, &mut parser_state);
            }
            Ok(Event::End(elem)) => {
                let tag_name = String::from_utf8_lossy(elem.name().as_ref()).to_string();
                handle_end_tag(
                    &tag_name,
                    &mut folder_stack,
                    &mut parser_state,
                    &mut results,
                )?;
            }
            Ok(Event::Text(text)) => {
                let content = String::from_utf8_lossy(text.as_ref()).to_string();
                parser_state.append_text(&content, &mut folder_stack);
            }
            Ok(Event::CData(cdata)) => {
                let content = String::from_utf8_lossy(cdata.as_ref()).to_string();
                if parser_state.in_description {
                    parser_state.current_description.push_str(&content);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("XML parsing error: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(results)
}

/// 解析器狀態機
#[derive(Debug, Default)]
struct ParserState {
    in_placemark: bool,
    in_name: bool,
    in_description: bool,
    in_coordinates: bool,
    in_folder_name: bool,
    current_name: String,
    current_description: String,
    current_coordinates_str: String,
}

impl ParserState {
    fn reset_placemark(&mut self) {
        self.current_name.clear();
        self.current_description.clear();
        self.current_coordinates_str.clear();
        self.in_name = false;
        self.in_description = false;
        self.in_coordinates = false;
    }

    fn append_text(&mut self, content: &str, folder_stack: &mut Vec<String>) {
        if self.in_name {
            self.current_name.push_str(content);
        } else if self.in_description {
            self.current_description.push_str(content);
        } else if self.in_coordinates {
            self.current_coordinates_str.push_str(content);
        } else if self.in_folder_name {
            if let Some(last) = folder_stack.last_mut() {
                last.push_str(content);
            }
        }
    }
}

/// 處理 XML 開始標籤
fn handle_start_tag(tag_name: &str, folder_stack: &mut Vec<String>, state: &mut ParserState) {
    match tag_name {
        "Folder" => {
            folder_stack.push(String::new());
        }
        "Placemark" => {
            state.in_placemark = true;
            state.reset_placemark();
        }
        "name" if state.in_placemark => {
            state.in_name = true;
        }
        "description" if state.in_placemark => {
            state.in_description = true;
        }
        "coordinates" if state.in_placemark => {
            state.in_coordinates = true;
        }
        "name"
            if !state.in_placemark
                && !folder_stack.is_empty()
                && folder_stack.last().map_or(false, |s| s.is_empty()) =>
        {
            state.in_folder_name = true;
        }
        _ => {}
    }
}

/// 處理 XML 結束標籤
fn handle_end_tag(
    tag_name: &str,
    folder_stack: &mut Vec<String>,
    state: &mut ParserState,
    results: &mut Vec<(Vec<String>, TrackMetadata)>,
) -> Result<()> {
    match tag_name {
        "Folder" => {
            if !folder_stack.is_empty() {
                folder_stack.pop();
            }
        }
        "Placemark" => {
            state.in_placemark = false;
            if let Some((start_time, end_time)) = extract_times(&state.current_description) {
                let coordinates = parse_coordinates(&state.current_coordinates_str)?;
                let (category, activity, year, month) = extract_categories(folder_stack);

                let metadata = TrackMetadata {
                    name: state.current_name.clone(),
                    start_time,
                    end_time,
                    coordinates,
                    category,
                    activity,
                    year,
                    month,
                };

                results.push((folder_stack.clone(), metadata));
            }
        }
        "name" if state.in_name => {
            state.in_name = false;
        }
        "description" if state.in_description => {
            state.in_description = false;
        }
        "coordinates" if state.in_coordinates => {
            state.in_coordinates = false;
        }
        "name" if state.in_folder_name => {
            state.in_folder_name = false;
        }
        _ => {}
    }
    Ok(())
}

/// 解析座標字串為 (lon, lat) 對
fn parse_coordinates(coords_str: &str) -> Result<Vec<(f64, f64)>> {
    Ok(coords_str
        .trim()
        .split_whitespace()
        .filter_map(|coord_str| {
            let parts: Vec<&str> = coord_str.split(',').collect();
            if parts.len() >= 2 {
                let lon = parts[0].parse().ok()?;
                let lat = parts[1].parse().ok()?;
                Some((lon, lat))
            } else {
                None
            }
        })
        .collect())
}
