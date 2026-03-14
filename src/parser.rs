use crate::{
    extract_categories, AnalyzerError, Result, TrackMetadata, END_TIME_PATTERN, START_TIME_PATTERN,
};
use chrono::NaiveDateTime;
use quick_xml::{events::Event, Reader};
use std::{
    fs,
    io::{BufRead, BufReader, Cursor, Read},
    path::PathBuf,
};

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

/// 從 KML 或 KMZ 檔案中提取所有 Placemark 軌跡
///
/// 使用流式 XML 解析器只掃描檔案一次，自動追蹤 KML 層級以提取軌跡分類資訊。
/// 若輸入為 KMZ 檔案（`.kmz` 副檔名），會先從 ZIP 壓縮檔中提取 KML 內容再解析。
///
/// # Arguments
///
/// * `file_path` - KML 或 KMZ 檔案的路徑
///
/// # Returns
///
/// 成功時返回 `Vec<(Vec<String>, TrackMetadata)>`，其中：
/// - `Vec<String>` 軌跡路徑（分類、活動、年份、月份）
/// - `TrackMetadata` 軌跡詳細資訊（名稱、時間、座標等）
///
/// # Errors
///
/// 若檔案不存在、KML 格式無效或 KMZ 中找不到 KML 檔案，返回 `AnalyzerError`
///
/// # Performance
///
/// - 時間複雜度：O(n)（單次掃描）
/// - 空間複雜度：O(m)（m = Placemarks 數量）
/// - KML 檔案適合處理大型檔案（50MB+），採用流式解析
/// - KMZ 檔案需先將 KML 解壓至記憶體，再進行流式解析
///
/// # Example
///
/// ```
/// use movement_tracks_analyzer::extract_placemarks_with_paths;
/// use std::path::PathBuf;
///
/// // 解析 KML 檔案
/// let placemarks = extract_placemarks_with_paths(&PathBuf::from("tests/fixtures/tracks.kml"))?;
///
/// // 迴圈處理每個軌跡
/// for (path, metadata) in placemarks {
///     println!("Placemark: {}", metadata.name);
///     println!("Category: {}", metadata.category);
///     println!("Distance: {} m", metadata.calculate_distance());
///     println!("Duration: {} s", metadata.duration_seconds());
///     println!("Path: {}", path.join(" > "));
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn extract_placemarks_with_paths(
    file_path: &PathBuf,
) -> Result<Vec<(Vec<String>, TrackMetadata)>> {
    let is_kmz = file_path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("kmz"));

    if is_kmz {
        let kml_bytes = extract_kml_from_kmz(file_path)?;
        let cursor = Cursor::new(kml_bytes);
        let reader = BufReader::new(cursor);
        parse_kml_from_reader(reader)
    } else {
        let file = fs::File::open(file_path)?;
        let reader = BufReader::new(file);
        parse_kml_from_reader(reader)
    }
}

/// 從 KMZ（ZIP）檔案中提取第一個 KML 檔案的內容
///
/// 依照 KMZ 規範，優先尋找根目錄的 `doc.kml`，若不存在則取第一個 `.kml` 副檔名的條目。
fn extract_kml_from_kmz(file_path: &PathBuf) -> Result<Vec<u8>> {
    let file = fs::File::open(file_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // 優先尋找 doc.kml（KMZ 規範的預設主檔案）
    if let Ok(mut entry) = archive.by_name("doc.kml") {
        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf)?;
        return Ok(buf);
    }

    // 退而求其次，尋找第一個 .kml 副檔名的檔案
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.name().to_ascii_lowercase().ends_with(".kml") {
            let mut buf = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut buf)?;
            return Ok(buf);
        }
    }

    Err(AnalyzerError::KmzError(
        "No KML file found in KMZ archive".to_string(),
    ))
}

/// 從實作 BufRead 的來源解析 KML 內容
fn parse_kml_from_reader<R: BufRead>(reader: R) -> Result<Vec<(Vec<String>, TrackMetadata)>> {
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
                eprintln!("KML parsing error: {}", e);
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
