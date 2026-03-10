use std::fs;
use std::env;
use std::path::PathBuf;
use std::io::Write;
use chrono::NaiveDateTime;
use regex::Regex;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
struct TrackMetadata {
    name: String,
    start_time: NaiveDateTime,
    end_time: NaiveDateTime,
    coordinates: Vec<(f64, f64)>,
    category_major: String,      // 大类别（如 戶外運動）
    category_minor: String,      // 小类别（如 步行）
    year: String,                // 年份（如 2025）
    month: String,               // 年月（如 2026-03）
}

impl TrackMetadata {
    /// 从 KML Description 字段中提取开始和结束时间
    fn extract_times(description: &str) -> Option<(NaiveDateTime, NaiveDateTime)> {
        // 使用正则表达式提取时间格式: YYYY-MM-DD HH:MM:SS
        let time_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})").ok()?;

        let matches: Vec<&str> = time_pattern
            .find_iter(description)
            .map(|m| m.as_str())
            .collect();

        if matches.len() >= 2 {
            let start = NaiveDateTime::parse_from_str(matches[0], "%Y-%m-%d %H:%M:%S").ok()?;
            let end = NaiveDateTime::parse_from_str(matches[1], "%Y-%m-%d %H:%M:%S").ok()?;
            Some((start, end))
        } else {
            None
        }
    }

    /// 计算轨迹距离（米）- 使用 Haversine 公式
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

            let a = (delta_lat / 2.0).sin().powi(2)
                + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
            let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

            total_distance += EARTH_RADIUS_KM * c;
        }

        total_distance * 1000.0  // 转换为米
    }

    /// 计算轨迹持续时间（秒）
    fn duration_seconds(&self) -> i64 {
        (self.end_time - self.start_time).num_seconds()
    }
}

/// 从 KML 内容中提取所有 Placemark
fn extract_placemarks_with_paths(kml_content: &str) -> Vec<(Vec<String>, TrackMetadata)> {
    let mut results = Vec::new();

    // 用 (?s) 标志让 . 匹配换行符（包括 \n）
    let placemark_pattern = Regex::new(
        r#"(?s)<Placemark[^>]*>.*?<name>(.*?)</name>.*?<description>(.*?)</description>.*?<coordinates>(.*?)</coordinates>.*?</Placemark>"#
    ).unwrap();

    for cap in placemark_pattern.captures_iter(kml_content) {
        if let (Some(name_match), Some(desc_match), Some(coords_match)) =
            (cap.get(1), cap.get(2), cap.get(3)) {

            let name = name_match.as_str().trim().to_string();
            let description = desc_match.as_str();
            let coordinates_str = coords_match.as_str();

            if let Some((start_time, end_time)) = TrackMetadata::extract_times(description) {
                let coordinates: Vec<(f64, f64)> = coordinates_str
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

                let folder_path = extract_folder_path_for_placemark(kml_content, &name);

                // 从文件夹路径中提取分类信息
                let (category_major, category_minor, year, month) = extract_categories(&folder_path);

                let metadata = TrackMetadata {
                    name,
                    start_time,
                    end_time,
                    coordinates,
                    category_major,
                    category_minor,
                    year,
                    month,
                };

                results.push((folder_path, metadata));
            }
        }
    }

    results
}

/// 为某个 Placemark 提取其所在的 Folder 路径
fn extract_folder_path_for_placemark(kml_content: &str, placemark_name: &str) -> Vec<String> {
    let mut path = Vec::new();

    // 寻找该 Placemark 的位置
    if let Some(placemark_pos) = kml_content.find(&format!("<name>{}</name>", placemark_name)) {
        let before = &kml_content[..placemark_pos];

        // 使用栈来追踪folder嵌套
        let mut folder_stack: Vec<String> = Vec::new();
        let folder_open_pattern = Regex::new(r"<Folder[^>]*>\s*<name>(.*?)</name>").unwrap();
        let _folder_close_pattern = Regex::new(r"</Folder>").unwrap();

        // 遍历KML内容，同时追踪开和闭
        let mut pos = 0;
        let _char_indices: Vec<usize> = before.char_indices().map(|(i, _)| i).collect();


        // 更简单的方法：计算位置
        while pos < before.len() {
            // 找下一个 <Folder> 或 </Folder>
            if let Some(found_open) = before[pos..].find("<Folder") {
                let open_pos = pos + found_open;
                if let Some(found_close) = before[pos..].find("</Folder>") {
                    let close_pos = pos + found_close;

                    if open_pos < close_pos {
                        // 有开，处理它
                        if let Some(cap) = folder_open_pattern.captures(&before[open_pos..]) {
                            if let Some(name) = cap.get(1) {
                                folder_stack.push(name.as_str().to_string());
                            }
                        }
                        pos = open_pos + 7; // "<Folder".len()
                    } else {
                        // 有闭，处理它
                        if !folder_stack.is_empty() {
                            folder_stack.pop();
                        }
                        pos = close_pos + 9; // "</Folder>".len()
                    }
                } else {
                    // 只有开，没有闭
                    if let Some(cap) = folder_open_pattern.captures(&before[open_pos..]) {
                        if let Some(name) = cap.get(1) {
                            folder_stack.push(name.as_str().to_string());
                        }
                    }
                    pos = open_pos + 7;
                }
            } else if let Some(found_close) = before[pos..].find("</Folder>") {
                if !folder_stack.is_empty() {
                    folder_stack.pop();
                }
                pos += found_close + 9;
            } else {
                break;
            }
        }

        path = folder_stack;
    }

    path
}

/// 从文件夹路径中提取分类信息（大类别、小类别、年份、月份）
fn extract_categories(folder_path: &[String]) -> (String, String, String, String) {
    // 跳过顶级的 "Movement Tracks (Example)" 等根节点
    let meaningful_path: Vec<&String> = folder_path.iter()
        .filter(|name| !name.contains("(Example)") && !name.contains("Movement Tracks"))
        .collect();

    let mut category_major = String::new();
    let mut category_minor = String::new();
    let mut year = String::new();
    let mut month = String::new();

    // 从路径末尾向前取，最多4个元素
    let path_len = meaningful_path.len();

    if path_len >= 4 {
        // 有4个或以上元素，取最后4个
        category_major = meaningful_path[path_len - 4].clone();
        category_minor = meaningful_path[path_len - 3].clone();
        year = meaningful_path[path_len - 2].clone();
        month = meaningful_path[path_len - 1].clone();
    } else if path_len == 3 {
        // 3个元素：跳过category_major
        category_minor = meaningful_path[0].clone();
        year = meaningful_path[1].clone();
        month = meaningful_path[2].clone();
    } else if path_len == 2 {
        // 2个元素：跳过category_major和category_minor
        year = meaningful_path[0].clone();
        month = meaningful_path[1].clone();
    } else if path_len == 1 {
        // 1个元素：作为月份或年份，需要判断格式
        let elem = meaningful_path[0];
        // 如果包含 "-" 且长度为7（如 2013-08），认为是month；否则认为是year
        if elem.contains("-") && elem.len() == 7 {
            month = elem.to_string();
        } else {
            year = elem.to_string();
        }
    }

    (category_major, category_minor, year, month)
}

/// 获取 KML 文件路径
/// 优先级：
/// 1. 命令行 -f/--file 参数指定的路径
/// 2. 执行文件同目录的 "移動軌跡.kml"
/// 3. 执行文件同目录的 "Movement Tracks.kml"
/// 4. 当前工作目录的 "移動軌跡.kml"
/// 5. 当前工作目录的 "Movement Tracks.kml"
fn get_kml_file_path() -> Result<PathBuf, String> {
    let args: Vec<String> = env::args().collect();

    // 检查命令行参数
    for i in 0..args.len() {
        if (args[i] == "-f" || args[i] == "--file") && i + 1 < args.len() {
            let path = PathBuf::from(&args[i + 1]);
            if path.exists() {
                println!("📄 Using KML file from command line: {}", path.display());
                return Ok(path);
            } else {
                return Err(format!("KML file not found: {}", path.display()));
            }
        }
    }

    // 尝试获取执行文件所在目录
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // 优先查找 "移動軌跡.kml"
            let path1 = exe_dir.join("移動軌跡.kml");
            if path1.exists() {
                println!("📄 Using default KML file: {}", path1.display());
                return Ok(path1);
            }

            // 其次查找 "Movement Tracks.kml"
            let path2 = exe_dir.join("Movement Tracks.kml");
            if path2.exists() {
                println!("📄 Using default KML file: {}", path2.display());
                return Ok(path2);
            }
        }
    }

    // 尝试当前工作目录
    let current_dir = env::current_dir().map_err(|e| e.to_string())?;

    // 优先查找 "移動軌跡.kml"
    let path1 = current_dir.join("移動軌跡.kml");
    if path1.exists() {
        println!("📄 Using default KML file: {}", path1.display());
        return Ok(path1);
    }

    // 其次查找 "Movement Tracks.kml"
    let path2 = current_dir.join("Movement Tracks.kml");
    if path2.exists() {
        println!("📄 Using default KML file: {}", path2.display());
        return Ok(path2);
    }

    Err("No KML file found. Please specify with -f/--file or place 移動軌跡.kml or Movement Tracks.kml in the current directory.".to_string())
}

/// 打印使用说明
fn print_usage() {
    println!("Movement Tracks Analyzer");
    println!("\nUsage: movement_tracks_analyzer [OPTIONS]");
    println!("\nOptions:");
    println!("  -f, --file <PATH>      Specify the KML file path");
    println!("  -o, --output <TYPE>    Output type: 'shell' or 'file' (default: file)");
    println!("  -m, --format <FORMAT>  Output format: 'json', 'csv', 'tsv', 'table' (default: csv)");
    println!("  -h, --help             Show this help message");
}

/// 从命令行参数中获取值
fn get_arg_value(args: &[String], short: &str, long: &str) -> Option<String> {
    for i in 0..args.len() {
        if (args[i] == short || args[i] == long) && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

/// 生成CSV格式的输出
fn format_as_csv(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let mut output = String::new();

    // CSV 头部
    output.push_str("Placemark Name,Start Time,End Time,Duration (seconds),Distance (meters),Coordinate Count,Category Major,Category Minor,Year,Month\n");

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
            metadata.category_major,
            metadata.category_minor,
            metadata.year,
            metadata.month
        ));
    }

    output
}

/// 生成TSV格式的输出
fn format_as_tsv(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let mut output = String::new();

    // TSV 头部
    output.push_str("Placemark Name\tStart Time\tEnd Time\tDuration (seconds)\tDistance (meters)\tCoordinate Count\tCategory Major\tCategory Minor\tYear\tMonth\n");

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
            metadata.category_major,
            metadata.category_minor,
            metadata.year,
            metadata.month
        ));
    }

    output
}

/// 生成JSON格式的输出
fn format_as_json(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let mut output = String::from("[\n");

    for (idx, (_folder_path, metadata)) in tracks.iter().enumerate() {
        let duration = metadata.duration_seconds();
        let distance = metadata.calculate_distance();

        output.push_str(&format!(
            "  {{\n    \"name\": \"{}\",\n    \"start_time\": \"{}\",\n    \"end_time\": \"{}\",\n    \"duration_seconds\": {},\n    \"distance_meters\": {},\n    \"coordinate_count\": {},\n    \"category_major\": \"{}\",\n    \"category_minor\": \"{}\",\n    \"year\": \"{}\",\n    \"month\": \"{}\"\n  }}",
            metadata.name.replace("\"", "\\\""),
            metadata.start_time.format("%Y-%m-%d %H:%M:%S"),
            metadata.end_time.format("%Y-%m-%d %H:%M:%S"),
            duration,
            distance as u64,
            metadata.coordinates.len(),
            metadata.category_major,
            metadata.category_minor,
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

/// 计算字符串的显示宽度（考虑汉字等宽字符）
fn display_width(s: &str) -> usize {
    s.width()
}

/// 计算列宽（用于表格输出）
fn calculate_column_widths(tracks: &[(Vec<String>, TrackMetadata)]) -> Vec<usize> {
    let mut widths = vec![
        display_width("Placemark Name"),
        display_width("Start Time"),
        display_width("End Time"),
        display_width("Duration (s)"),
        display_width("Distance (m)"),
        display_width("Points"),
        display_width("Category Major"),
        display_width("Category Minor"),
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
        widths[6] = widths[6].max(display_width(&metadata.category_major));
        widths[7] = widths[7].max(display_width(&metadata.category_minor));
        widths[8] = widths[8].max(display_width(&metadata.year));
        widths[9] = widths[9].max(display_width(&metadata.month));
    }

    widths
}

/// 根据显示宽度进行字符串格式化（左对齐，用空格填充）
fn format_cell(text: &str, width: usize) -> String {
    let text_width = display_width(text);
    if text_width >= width {
        text.to_string()
    } else {
        let padding = width - text_width;
        format!("{}{}", text, " ".repeat(padding))
    }
}

/// 生成表格格式的输出
fn format_as_table(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let widths = calculate_column_widths(tracks);
    let mut output = String::new();

    // 表头
    let headers = vec![
        "Placemark Name",
        "Start Time",
        "End Time",
        "Duration (s)",
        "Distance (m)",
        "Points",
        "Category Major",
        "Category Minor",
        "Year",
        "Month",
    ];

    for (i, header) in headers.iter().enumerate() {
        output.push_str(&format_cell(header, widths[i]));
        output.push_str(" ");
    }
    output.push_str("\n");

    // 分隔线
    for width in &widths {
        output.push_str(&"-".repeat(*width));
        output.push_str(" ");
    }
    output.push_str("\n");

    // 数据行
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
            metadata.category_major.clone(),
            metadata.category_minor.clone(),
            metadata.year.clone(),
            metadata.month.clone(),
        ];

        for (i, data) in row_data.iter().enumerate() {
            output.push_str(&format_cell(data, widths[i]));
            output.push_str(" ");
        }
        output.push_str("\n");
    }

    output
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // 处理帮助参数
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_usage();
        return Ok(());
    }

    // 获取 KML 文件路径
    let kml_path = get_kml_file_path()?;

    // 解析输出参数
    let output_type = get_arg_value(&args, "-o", "--output").unwrap_or_else(|| "file".to_string());
    let format_type = get_arg_value(&args, "-m", "--format").unwrap_or_else(|| "csv".to_string());

    // 验证参数
    if !["shell", "file"].contains(&output_type.as_str()) {
        eprintln!("Error: Invalid output type '{}'. Must be 'shell' or 'file'", output_type);
        return Ok(());
    }

    if !["json", "csv", "tsv", "table"].contains(&format_type.as_str()) {
        eprintln!("Error: Invalid format '{}'. Must be 'json', 'csv', 'tsv' or 'table'", format_type);
        return Ok(());
    }

    // 当 format='table' 且 output='file' 时，使用 csv 格式
    let actual_format = if format_type == "table" && output_type == "file" {
        "csv".to_string()
    } else {
        format_type.clone()
    };

    // 读取 KML 文件
    let kml_content = fs::read_to_string(&kml_path)?;

    // 提取所有 Placemarks
    let placemarks = extract_placemarks_with_paths(&kml_content);

    // 生成输出
    let output = match actual_format.as_str() {
        "json" => format_as_json(&placemarks),
        "csv" => format_as_csv(&placemarks),
        "tsv" => format_as_tsv(&placemarks),
        "table" => format_as_table(&placemarks),
        _ => format_as_csv(&placemarks),
    };

    // 输出内容
    if output_type == "shell" {
        print!("{}", output);
    } else {
        // 文件输出
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
