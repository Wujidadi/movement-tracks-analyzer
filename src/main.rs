use std::{env, fs, io::{Write, BufReader}, path::PathBuf};
use chrono::NaiveDateTime;
use once_cell::sync::Lazy;
use quick_xml::{events::Event, Reader};
use regex::Regex;
use unicode_width::UnicodeWidthStr;

/// 開始時間正規表示式
static START_TIME_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"<b>\s*Start\s*:\s*</b>\s*(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})<br />").unwrap());

/// 結束時間正規表示式
static END_TIME_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"<b>\s*End\s*:\s*</b>\s*(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})").unwrap());

#[derive(Debug, Clone)]
struct TrackMetadata {
    /// 軌跡名稱
    name: String,
    /// 開始時間
    start_time: NaiveDateTime,
    /// 結束時間
    end_time: NaiveDateTime,
    /// 座標點
    coordinates: Vec<(f64, f64)>,
    /// 分類
    category: String, // 戶外運動、動力交通工具...
    /// 活動
    activity: String, // 步行、跑步、機車、飛機...
    /// 年度
    year: String, // 2013、2025...
    /// 月份
    month: String, // 2015-02、2026-03...
}

impl TrackMetadata {
    /// 從 KML Description 中取得開始和結束時間
    fn extract_times(description: &str) -> Option<(NaiveDateTime, NaiveDateTime)> {
        let start_match = START_TIME_PATTERN.captures(description)?;
        let end_match = END_TIME_PATTERN.captures(description)?;

        let start_str = start_match.get(1)?.as_str();
        let end_str = end_match.get(1)?.as_str();

        let start = NaiveDateTime::parse_from_str(start_str, "%Y-%m-%d %H:%M:%S").ok()?;
        let end = NaiveDateTime::parse_from_str(end_str, "%Y-%m-%d %H:%M:%S").ok()?;

        Some((start, end))
    }

    /// 計算軌跡距離（公尺）- 使用半正矢（Haversine）公式
    fn calculate_distance(&self) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;

        let mut total_distance = 0.0;

        for i in 0..self.coordinates.len() - 1 {
            let (lon1, lat1) = self.coordinates[i];
            let (lon2, lat2) = self.coordinates[i + 1];

            let lat1_rad = lat1.to_radians();
            let lat2_rad = lat2.to_radians();
            let delta_lat = (lat2 - lat1).to_radians();
            let delta_lon = (lon2 - lon1).to_radians();

            let a = (delta_lat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
            let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

            total_distance += EARTH_RADIUS_KM * c;
        }

        total_distance * 1000.0  // 轉換為公尺
    }

    /// 計算軌跡持續時間（秒）
    fn duration_seconds(&self) -> i64 {
        (self.end_time - self.start_time).num_seconds()
    }
}

/// 使用流式 XML 解析器從 KML 中提取所有 Placemark（高效、只掃描一次）
fn extract_placemarks_with_paths(file_path: &PathBuf) -> Result<Vec<(Vec<String>, TrackMetadata)>, Box<dyn std::error::Error>> {
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut xml_reader = Reader::from_reader(reader);

    let mut results = Vec::new();
    let mut buf = Vec::new();

    // 用於追蹤當前的 folder 路徑堆棧
    let mut folder_stack: Vec<String> = Vec::new();

    // 用於臨時儲存當前 Placemark 的資訊
    let mut current_name = String::new();
    let mut current_description = String::new();
    let mut current_coordinates_str = String::new();
    let mut in_placemark = false;
    let mut in_name = false;
    let mut in_description = false;
    let mut in_coordinates = false;
    let mut in_folder_name = false;

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(elem)) => {
                let tag_name = String::from_utf8_lossy(elem.name().as_ref()).to_string();

                match tag_name.as_str() {
                    "Folder" => {
                        folder_stack.push(String::new()); // 暫時佔位
                    }
                    "Placemark" => {
                        in_placemark = true;
                        current_name.clear();
                        current_description.clear();
                        current_coordinates_str.clear();
                    }
                    "name" if in_placemark => {
                        in_name = true;
                    }
                    "description" if in_placemark => {
                        in_description = true;
                    }
                    "coordinates" if in_placemark => {
                        in_coordinates = true;
                    }
                    "name" if !in_placemark && !folder_stack.is_empty() && folder_stack.last().map_or(false, |s| s.is_empty()) => {
                        in_folder_name = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(elem)) => {
                let tag_name = String::from_utf8_lossy(elem.name().as_ref()).to_string();

                match tag_name.as_str() {
                    "Folder" => {
                        if !folder_stack.is_empty() {
                            folder_stack.pop();
                        }
                    }
                    "Placemark" => {
                        in_placemark = false;

                        // 處理當前 Placemark
                        if let Some((start_time, end_time)) = TrackMetadata::extract_times(&current_description) {
                            let coordinates: Vec<(f64, f64)> = current_coordinates_str
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
                                .collect();

                            let (category, activity, year, month) = extract_categories(&folder_stack);

                            let metadata = TrackMetadata {
                                name: current_name.clone(),
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
                    "name" if in_name => {
                        in_name = false;
                    }
                    "description" if in_description => {
                        in_description = false;
                    }
                    "coordinates" if in_coordinates => {
                        in_coordinates = false;
                    }
                    "name" if in_folder_name => {
                        in_folder_name = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(text)) => {
                let content = String::from_utf8_lossy(text.as_ref()).to_string();

                if in_name {
                    current_name.push_str(&content);
                } else if in_description {
                    current_description.push_str(&content);
                } else if in_coordinates {
                    current_coordinates_str.push_str(&content);
                } else if in_folder_name {
                    if let Some(last) = folder_stack.last_mut() {
                        last.push_str(&content);
                    }
                }
            }
            Ok(Event::CData(cdata)) => {
                let content = String::from_utf8_lossy(cdata.as_ref()).to_string();
                if in_description {
                    current_description.push_str(&content);
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

/// 從路徑中提取分類、活動、年度、月份
fn extract_categories(folder_path: &[String]) -> (String, String, String, String) {
    // 跳過頂級根節點
    let meaningful_path: Vec<&String> = folder_path.iter()
        .filter(|name| !name.contains("(Example)") && !name.contains("Movement Tracks"))
        .collect();

    let mut category = String::new();
    let mut activity = String::new();
    let mut year = String::new();
    let mut month = String::new();

    // 從路徑末尾向前取最多 4 個元素
    let path_len = meaningful_path.len();
    if path_len >= 4 {
        // 有 4 個或以上元素，取最後4個
        category = meaningful_path[path_len - 4].clone();
        activity = meaningful_path[path_len - 3].clone();
        year = meaningful_path[path_len - 2].clone();
        month = meaningful_path[path_len - 1].clone();
    } else if path_len == 3 {
        // 3 個元素：跳過 category
        activity = meaningful_path[0].clone();
        year = meaningful_path[1].clone();
        month = meaningful_path[2].clone();
    } else if path_len == 2 {
        // 2 個元素：跳過 category 和 activity
        year = meaningful_path[0].clone();
        month = meaningful_path[1].clone();
    } else if path_len == 1 {
        // 1 個元素：依格式判斷是年或月
        let elem = meaningful_path[0];
        // 包含 "-" 且長度等於 7（如 2013-08）則為月份；否則為年度
        if elem.contains("-") && elem.len() == 7 {
            month = elem.to_string();
        } else {
            year = elem.to_string();
        }
    }

    (category, activity, year, month)
}

/// 獲取 KML 檔案路徑
///
/// 優先級：
/// 1. 命令行 `-f` / `--file` 參數指定的路徑
/// 2. 執行檔同目錄的 `移動軌跡.kml`
/// 3. 執行檔同目錄的 `Movement Tracks.kml`
/// 4. 當前工作目錄的 `移動軌跡.kml`
/// 5. 當前工作目錄的 `Movement Tracks.kml`
fn get_kml_file_path() -> Result<PathBuf, String> {
    let args: Vec<String> = env::args().collect();

    // 檢查命令行參數
    for i in 0..args.len() {
        if (args[i] == "-f" || args[i] == "--file") && i + 1 < args.len() {
            let path = PathBuf::from(&args[i + 1]);
            if path.exists() {
                println!("Using KML file from command line: {}", path.display());
                return Ok(path);
            } else {
                return Err(format!("KML file not found: {}", path.display()));
            }
        }
    }

    // 嘗試獲取執行檔所在目錄
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // 優先查找 "移動軌跡.kml"
            let path1 = exe_dir.join("移動軌跡.kml");
            if path1.exists() {
                println!("Using default KML file: {}", path1.display());
                return Ok(path1);
            }

            // 其次查找 "Movement Tracks.kml"
            let path2 = exe_dir.join("Movement Tracks.kml");
            if path2.exists() {
                println!("Using default KML file: {}", path2.display());
                return Ok(path2);
            }
        }
    }

    // 嘗試當前工作目錄
    let current_dir = env::current_dir().map_err(|e| e.to_string())?;

    // 優先查找 "移動軌跡.kml"
    let path1 = current_dir.join("移動軌跡.kml");
    if path1.exists() {
        println!("Using default KML file: {}", path1.display());
        return Ok(path1);
    }

    // 其次查找 "Movement Tracks.kml"
    let path2 = current_dir.join("Movement Tracks.kml");
    if path2.exists() {
        println!("Using default KML file: {}", path2.display());
        return Ok(path2);
    }

    Err("No KML file found. Please specify with -f / --file or place 移動軌跡.kml or Movement Tracks.kml in the current directory.".to_string())
}

/// 列印使用說明
fn print_usage() {
    println!("Movement Tracks Analyzer");
    println!("\nUsage: movement_tracks_analyzer [OPTIONS]");
    println!("\nOptions:");
    println!("  -f, --file <PATH>      Specify the KML file path");
    println!("  -o, --output <TYPE>    Output type: 'shell' or 'file' (default: file)");
    println!("  -m, --format <FORMAT>  Output format: 'json', 'csv', 'tsv', 'table' (default: csv)");
    println!("  -h, --help             Show this help message");
}

/// 從命令行參數中解析並獲取值
fn get_arg_value(args: &[String], short: &str, long: &str) -> Option<String> {
    for i in 0..args.len() {
        if (args[i] == short || args[i] == long) && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

/// 輸出 CSV 格式
fn format_as_csv(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let mut output = String::new();

    // CSV 標題列
    output.push_str("Placemark Name,Start Time,End Time,Duration (seconds),Distance (meters),Coordinate Count,Category,Activity,Year,Month\n");

    for (_folder_path, metadata) in tracks {
        let duration = metadata.duration_seconds();
        let distance = metadata.calculate_distance();

        output.push_str(&format!(
            "\"{}\",{},{},{},{},{},{},{},{},{}\n",
            metadata.name.replace("\"", "\"\""),
            metadata.start_time.format("%Y-%m-%d %H:%M:%S"),
            metadata.end_time.format("%Y-%m-%d %H:%M:%S"),
            duration,
            distance as u64,
            metadata.coordinates.len(),
            metadata.category,
            metadata.activity,
            metadata.year,
            metadata.month
        ));
    }

    output
}

/// 輸出 TSV 格式
fn format_as_tsv(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let mut output = String::new();

    // TSV 標題列
    output.push_str("Placemark Name\tStart Time\tEnd Time\tDuration (seconds)\tDistance (meters)\tCoordinate Count\tCategory\tActivity\tYear\tMonth\n");

    for (_folder_path, metadata) in tracks {
        let duration = metadata.duration_seconds();
        let distance = metadata.calculate_distance();

        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            metadata.name,
            metadata.start_time.format("%Y-%m-%d %H:%M:%S"),
            metadata.end_time.format("%Y-%m-%d %H:%M:%S"),
            duration,
            distance as u64,
            metadata.coordinates.len(),
            metadata.category,
            metadata.activity,
            metadata.year,
            metadata.month
        ));
    }

    output
}

/// 輸出 JSON 格式
fn format_as_json(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let mut output = String::from("[\n");

    for (idx, (_folder_path, metadata)) in tracks.iter().enumerate() {
        let duration = metadata.duration_seconds();
        let distance = metadata.calculate_distance();

        output.push_str(&format!(
            "  {{\n    \"name\": \"{}\",\n    \"start_time\": \"{}\",\n    \"end_time\": \"{}\",\n    \"duration_seconds\": {},\n    \"distance_meters\": {},\n    \"coordinate_count\": {},\n    \"category\": \"{}\",\n    \"activity\": \"{}\",\n    \"year\": \"{}\",\n    \"month\": \"{}\"\n  }}",
            metadata.name.replace("\"", "\\\""),
            metadata.start_time.format("%Y-%m-%d %H:%M:%S"),
            metadata.end_time.format("%Y-%m-%d %H:%M:%S"),
            duration,
            distance as u64,
            metadata.coordinates.len(),
            metadata.category,
            metadata.activity,
            metadata.year,
            metadata.month
        ));

        if idx < tracks.len() - 1 {
            output.push_str(",");
        }
        output.push_str("\n");
    }

    output.push_str("]\n");
    output
}

/// 計算字串顯示寬度（考慮 Unicode 寬字元）
fn display_width(s: &str) -> usize {
    s.width()
}

/// 計算欄寬（用於表格輸出）
fn calculate_column_widths(tracks: &[(Vec<String>, TrackMetadata)]) -> Vec<usize> {
    let mut widths = vec![
        display_width("Placemark Name"),
        display_width("Start Time"),
        display_width("End Time"),
        display_width("Duration (s)"),
        display_width("Distance (m)"),
        display_width("Points"),
        display_width("Category"),
        display_width("Activity"),
        display_width("Year"),
        display_width("Month"),
    ];

    for (_folder_path, metadata) in tracks {
        let duration = metadata.duration_seconds();
        let distance = metadata.calculate_distance();

        widths[0] = widths[0].max(display_width(&metadata.name));
        widths[1] = widths[1].max(display_width(&metadata.start_time.format("%Y-%m-%d %H:%M:%S").to_string()));
        widths[2] = widths[2].max(display_width(&metadata.end_time.format("%Y-%m-%d %H:%M:%S").to_string()));
        widths[3] = widths[3].max(display_width(&duration.to_string()));
        widths[4] = widths[4].max(display_width(&(distance as u64).to_string()));
        widths[5] = widths[5].max(display_width(&metadata.coordinates.len().to_string()));
        widths[6] = widths[6].max(display_width(&metadata.category));
        widths[7] = widths[7].max(display_width(&metadata.activity));
        widths[8] = widths[8].max(display_width(&metadata.year));
        widths[9] = widths[9].max(display_width(&metadata.month));
    }

    widths
}

/// 根據顯示寬度格式化字串
///
/// 對於數字欄位靠右對齊；其他欄位靠左對齊
fn format_cell(text: &str, width: usize, col_index: usize) -> String {
    let text_width = display_width(text);
    if text_width >= width {
        text.to_string()
    } else {
        let padding = width - text_width;
        // 數字欄位（Duration, Distance, Points）靠右對齊
        if col_index == 3 || col_index == 4 || col_index == 5 {
            format!("{}{}", " ".repeat(padding), text)
        } else {
            // 其他欄位靠左對齊
            format!("{}{}", text, " ".repeat(padding))
        }
    }
}

/// 輸出表格格式
fn format_as_table(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let widths = calculate_column_widths(tracks);
    let mut output = String::new();

    // 標題列
    let headers = vec![
        "Placemark Name",
        "Start Time",
        "End Time",
        "Duration (s)",
        "Distance (m)",
        "Points",
        "Category",
        "Activity",
        "Year",
        "Month",
    ];

    for (i, header) in headers.iter().enumerate() {
        output.push_str(&format_cell(header, widths[i], i));
        output.push_str(" ");
    }
    output.push_str("\n");

    // 分隔線
    for width in &widths {
        output.push_str(&"-".repeat(*width));
        output.push_str(" ");
    }
    output.push_str("\n");

    // 資料列
    for (_folder_path, metadata) in tracks {
        let duration = metadata.duration_seconds();
        let distance = metadata.calculate_distance();

        let row_data = vec![
            metadata.name.clone(),
            metadata.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            metadata.end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            duration.to_string(),
            (distance as u64).to_string(),
            metadata.coordinates.len().to_string(),
            metadata.category.clone(),
            metadata.activity.clone(),
            metadata.year.clone(),
            metadata.month.clone(),
        ];

        for (i, data) in row_data.iter().enumerate() {
            output.push_str(&format_cell(data, widths[i], i));
            output.push_str(" ");
        }
        output.push_str("\n");
    }

    output
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // 顯示使用說明
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_usage();
        return Ok(());
    }

    // 獲取 KML 檔案路徑
    let kml_path = get_kml_file_path()?;

    // 解析輸出參數
    let output_type = get_arg_value(&args, "-o", "--output").unwrap_or_else(|| "file".to_string());
    let format_type = get_arg_value(&args, "-m", "--format").unwrap_or_else(|| "csv".to_string());

    // 驗證參數
    if !["shell", "file"].contains(&output_type.as_str()) {
        eprintln!("Error: Invalid output type '{}'. Must be 'shell' or 'file'", output_type);
        return Ok(());
    }

    if !["json", "csv", "tsv", "table"].contains(&format_type.as_str()) {
        eprintln!("Error: Invalid format '{}'. Must be 'json', 'csv', 'tsv' or 'table'", format_type);
        return Ok(());
    }

    // 當 format='table' 且 output='file' 時，使用 csv 格式
    let actual_format = if format_type == "table" && output_type == "file" {
        "csv".to_string()
    } else {
        format_type.clone()
    };

    // 提取所有 Placemarks（流式解析，只掃描一次）
    let placemarks = extract_placemarks_with_paths(&kml_path)?;

    // 生成輸出
    let output = match actual_format.as_str() {
        "json" => format_as_json(&placemarks),
        "csv" => format_as_csv(&placemarks),
        "tsv" => format_as_tsv(&placemarks),
        "table" => format_as_table(&placemarks),
        _ => format_as_csv(&placemarks),
    };

    // 輸出內容
    if output_type == "shell" {
        print!("{}", output);
    } else {
        // 輸出檔案
        let file_extension = match actual_format.as_str() {
            "json" => "json",
            "csv" => "csv",
            "tsv" => "tsv",
            "table" => "csv",
            _ => "csv",
        };

        let output_filename = format!("tracks_output.{}", file_extension);
        let mut file = fs::File::create(&output_filename)?;
        file.write_all(output.as_bytes())?;
        println!("Output saved to: {}", output_filename);
    }

    Ok(())
}

