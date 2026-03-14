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
    if is_kmz_file(file_path) {
        parse_kmz_file(file_path)
    } else {
        parse_kml_file(file_path)
    }
}

/// 判斷檔案是否為 KMZ 格式
fn is_kmz_file(file_path: &PathBuf) -> bool {
    file_path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("kmz"))
}

/// 解析 KMZ 檔案
fn parse_kmz_file(file_path: &PathBuf) -> Result<Vec<(Vec<String>, TrackMetadata)>> {
    let kml_bytes = extract_kml_from_kmz(file_path)?;
    let cursor = Cursor::new(kml_bytes);
    let reader = BufReader::new(cursor);
    parse_kml_from_reader(reader)
}

/// 解析 KML 檔案
fn parse_kml_file(file_path: &PathBuf) -> Result<Vec<(Vec<String>, TrackMetadata)>> {
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    parse_kml_from_reader(reader)
}

/// 從 KMZ（ZIP）檔案中提取第一個 KML 檔案的內容
///
/// 依照 KMZ 規範，優先尋找根目錄的 `doc.kml`，若不存在則取第一個 `.kml` 副檔名的條目。
fn extract_kml_from_kmz(file_path: &PathBuf) -> Result<Vec<u8>> {
    let file = fs::File::open(file_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    find_doc_kml(&mut archive)
        .or_else(|| find_first_kml(&mut archive))
        .ok_or_else(|| AnalyzerError::KmzError("No KML file found in KMZ archive".to_string()))
}

/// 從 KMZ 壓縮檔中讀取 doc.kml（KMZ 規範的預設主檔案）
fn find_doc_kml(archive: &mut zip::ZipArchive<fs::File>) -> Option<Vec<u8>> {
    let mut entry = archive.by_name("doc.kml").ok()?;
    let mut buf = Vec::with_capacity(entry.size() as usize);
    entry.read_to_end(&mut buf).ok()?;
    Some(buf)
}

/// 從 KMZ 壓縮檔中尋找第一個 .kml 副檔名的檔案
fn find_first_kml(archive: &mut zip::ZipArchive<fs::File>) -> Option<Vec<u8>> {
    let len = archive.len();
    (0..len).find_map(|i| try_read_kml_entry(archive, i))
}

/// 嘗試讀取 KMZ 中指定索引的 KML 條目
fn try_read_kml_entry(archive: &mut zip::ZipArchive<fs::File>, index: usize) -> Option<Vec<u8>> {
    let mut entry = archive.by_index(index).ok()?;
    if !entry.name().to_ascii_lowercase().ends_with(".kml") {
        return None;
    }
    let mut buf = Vec::with_capacity(entry.size() as usize);
    entry.read_to_end(&mut buf).ok()?;
    Some(buf)
}

/// 當前活躍的文字欄位
#[derive(Debug, Default, PartialEq)]
enum ActiveTextField {
    #[default]
    None,
    /// Placemark 名稱
    Name,
    /// Placemark 描述
    Description,
    /// Placemark 座標
    Coordinates,
    /// 資料夾名稱
    FolderName,
}

/// 解析器狀態機
#[derive(Debug, Default)]
struct ParserState {
    in_placemark: bool,
    active_field: ActiveTextField,
    current_name: String,
    current_description: String,
    current_coordinates_str: String,
}

impl ParserState {
    /// 重設 Placemark 相關狀態
    fn reset_placemark(&mut self) {
        self.current_name.clear();
        self.current_description.clear();
        self.current_coordinates_str.clear();
        self.active_field = ActiveTextField::None;
    }

    /// 進入 Placemark
    fn enter_placemark(&mut self) {
        self.in_placemark = true;
        self.reset_placemark();
    }

    /// 處理內容標籤（name/description/coordinates）的開啟
    fn open_content_tag(&mut self, tag_name: &str, folder_stack: &[String]) {
        if self.in_placemark {
            self.open_placemark_tag(tag_name);
        } else if tag_name == "name" && self.is_at_unnamed_folder(folder_stack) {
            self.active_field = ActiveTextField::FolderName;
        }
    }

    /// 設定 Placemark 內的活躍文字欄位
    fn open_placemark_tag(&mut self, tag_name: &str) {
        self.active_field = match tag_name {
            "name" => ActiveTextField::Name,
            "description" => ActiveTextField::Description,
            "coordinates" => ActiveTextField::Coordinates,
            _ => return,
        };
    }

    /// 判斷當前是否處於未命名的資料夾
    fn is_at_unnamed_folder(&self, folder_stack: &[String]) -> bool {
        folder_stack.last().is_some_and(|s| s.is_empty())
    }

    /// 關閉當前活躍的文字欄位
    fn close_content_tag(&mut self) {
        self.active_field = ActiveTextField::None;
    }

    /// 追加文字到當前活躍欄位
    fn append_text(&mut self, content: &str, folder_stack: &mut Vec<String>) {
        match self.active_field {
            ActiveTextField::Name => self.current_name.push_str(content),
            ActiveTextField::Description => self.current_description.push_str(content),
            ActiveTextField::Coordinates => self.current_coordinates_str.push_str(content),
            ActiveTextField::FolderName => append_to_folder_name(folder_stack, content),
            ActiveTextField::None => {}
        }
    }

    /// 處理 CData 內容（僅在描述欄位中有效）
    fn handle_cdata(&mut self, content: &str) {
        if self.active_field == ActiveTextField::Description {
            self.current_description.push_str(content);
        }
    }
}

/// 追加文字到資料夾堆疊頂端的名稱
fn append_to_folder_name(folder_stack: &mut Vec<String>, content: &str) {
    if let Some(last) = folder_stack.last_mut() {
        last.push_str(content);
    }
}

/// 從實作 BufRead 的來源解析 KML 內容
fn parse_kml_from_reader<R: BufRead>(reader: R) -> Result<Vec<(Vec<String>, TrackMetadata)>> {
    let mut xml_reader = Reader::from_reader(reader);
    let mut results = Vec::new();
    let mut folder_stack: Vec<String> = Vec::new();
    let mut state = ParserState::default();

    read_all_events(&mut xml_reader, &mut folder_stack, &mut state, &mut results)?;

    Ok(results)
}

/// 讀取並處理所有 XML 事件
fn read_all_events<R: BufRead>(
    xml_reader: &mut Reader<R>,
    folder_stack: &mut Vec<String>,
    state: &mut ParserState,
    results: &mut Vec<(Vec<String>, TrackMetadata)>,
) -> Result<()> {
    let mut buf = Vec::new();
    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => return Ok(()),
            Ok(event) => process_event(event, folder_stack, state, results)?,
            Err(e) => {
                eprintln!("KML parsing error: {}", e);
                return Ok(());
            }
        }
        buf.clear();
    }
}

/// 處理單個 XML 事件
fn process_event(
    event: Event<'_>,
    folder_stack: &mut Vec<String>,
    state: &mut ParserState,
    results: &mut Vec<(Vec<String>, TrackMetadata)>,
) -> Result<()> {
    match event {
        Event::Start(elem) => {
            let tag = String::from_utf8_lossy(elem.name().as_ref()).to_string();
            handle_start_tag(&tag, folder_stack, state);
        }
        Event::End(elem) => {
            let tag = String::from_utf8_lossy(elem.name().as_ref()).to_string();
            handle_end_tag(&tag, folder_stack, state, results)?;
        }
        Event::Text(text) => {
            let content = String::from_utf8_lossy(text.as_ref()).to_string();
            state.append_text(&content, folder_stack);
        }
        Event::CData(cdata) => {
            let content = String::from_utf8_lossy(cdata.as_ref()).to_string();
            state.handle_cdata(&content);
        }
        _ => {}
    }
    Ok(())
}

/// 處理 XML 開始標籤
fn handle_start_tag(tag_name: &str, folder_stack: &mut Vec<String>, state: &mut ParserState) {
    match tag_name {
        "Folder" => folder_stack.push(String::new()),
        "Placemark" => state.enter_placemark(),
        "name" | "description" | "coordinates" => state.open_content_tag(tag_name, folder_stack),
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
            folder_stack.pop();
        }
        "Placemark" => finalize_placemark(state, folder_stack, results)?,
        "name" | "description" | "coordinates" => state.close_content_tag(),
        _ => {}
    }
    Ok(())
}

/// 完成 Placemark 解析，建立 TrackMetadata 並加入結果
fn finalize_placemark(
    state: &mut ParserState,
    folder_stack: &[String],
    results: &mut Vec<(Vec<String>, TrackMetadata)>,
) -> Result<()> {
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

        results.push((folder_stack.to_vec(), metadata));
    }
    Ok(())
}

/// 解析座標字串為 (lon, lat) 對
fn parse_coordinates(coords_str: &str) -> Result<Vec<(f64, f64)>> {
    Ok(coords_str
        .trim()
        .split_whitespace()
        .filter_map(parse_single_coordinate)
        .collect())
}

/// 解析單個座標字串
fn parse_single_coordinate(coord_str: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = coord_str.split(',').collect();
    if parts.len() >= 2 {
        let lon = parts[0].parse().ok()?;
        let lat = parts[1].parse().ok()?;
        Some((lon, lat))
    } else {
        None
    }
}
