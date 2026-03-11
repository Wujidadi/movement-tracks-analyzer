use crate::TrackMetadata;
use serde::Serialize;
use unicode_width::UnicodeWidthStr;

/// 時間格式化字串常數
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Json,
    Csv,
    Tsv,
    Table,
}

/// 單個軌跡記錄的 JSON 結構
#[derive(Serialize)]
struct TrackRecord {
    name: String,
    start_time: String,
    end_time: String,
    duration_seconds: i64,
    distance_meters: u64,
    coordinate_count: usize,
    category: String,
    activity: String,
    year: String,
    month: String,
}

/// 根據格式生成輸出
pub fn format_output(format: OutputFormat, tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    match format {
        OutputFormat::Json => format_json(tracks),
        OutputFormat::Csv => format_csv(tracks),
        OutputFormat::Tsv => format_tsv(tracks),
        OutputFormat::Table => format_table(tracks),
    }
}

/// 計算軌跡的數值指標（持續秒數、距離公尺數）
fn calculate_metrics(metadata: &TrackMetadata) -> (i64, u64) {
    (
        metadata.duration_seconds(),
        metadata.calculate_distance() as u64,
    )
}

/// 格式化時間戳
fn format_timestamp(metadata: &TrackMetadata) -> (String, String) {
    (
        metadata.start_time.format(TIME_FORMAT).to_string(),
        metadata.end_time.format(TIME_FORMAT).to_string(),
    )
}

/// 輸出 CSV 格式
fn format_csv(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let mut output = String::from(
        "Name,Start,End,Duration (seconds),Distance (meters),Points,Category,Activity,Year,Month\n"
    );

    for (_folder_path, metadata) in tracks {
        let (duration, distance) = calculate_metrics(metadata);
        let (start_time, end_time) = format_timestamp(metadata);

        output.push_str(&format!(
            "\"{}\",{},{},{},{},{},{},{},{},{}\n",
            metadata.name.replace("\"", "\"\""),
            start_time,
            end_time,
            duration,
            distance,
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
fn format_tsv(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let mut output = String::from(
        "Name\tStart\tEnd\tDuration (seconds)\tDistance (meters)\tPoints\tCategory\tActivity\tYear\tMonth\n"
    );

    for (_folder_path, metadata) in tracks {
        let (duration, distance) = calculate_metrics(metadata);
        let (start_time, end_time) = format_timestamp(metadata);

        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            metadata.name,
            start_time,
            end_time,
            duration,
            distance,
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
fn format_json(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let records: Vec<TrackRecord> = tracks
        .iter()
        .map(|(_, metadata)| {
            let (duration, distance) = calculate_metrics(metadata);
            let (start_time, end_time) = format_timestamp(metadata);

            TrackRecord {
                name: metadata.name.clone(),
                start_time,
                end_time,
                duration_seconds: duration,
                distance_meters: distance,
                coordinate_count: metadata.coordinates.len(),
                category: metadata.category.clone(),
                activity: metadata.activity.clone(),
                year: metadata.year.clone(),
                month: metadata.month.clone(),
            }
        })
        .collect();

    serde_json::to_string_pretty(&records).unwrap_or_default()
}

/// 輸出表格格式
fn format_table(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let widths = calculate_column_widths(tracks);
    let mut output = String::new();

    // 標題列
    let headers = [
        "Name",
        "Start",
        "End",
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
        output.push(' ');
    }
    output.push('\n');

    // 分隔線
    for width in &widths {
        output.push_str(&"-".repeat(*width));
        output.push(' ');
    }
    output.push('\n');

    // 資料列
    for (_folder_path, metadata) in tracks {
        let row_data = format_row_data(metadata);

        for (i, data) in row_data.iter().enumerate() {
            output.push_str(&format_cell(data, widths[i], i));
            output.push(' ');
        }
        output.push('\n');
    }

    output
}

/// 計算字串顯示寬度（考慮 Unicode 寬字元）
fn display_width(s: &str) -> usize {
    s.width()
}

/// 計算欄寬（用於表格輸出）
fn calculate_column_widths(tracks: &[(Vec<String>, TrackMetadata)]) -> [usize; 10] {
    let headers = [
        "Name",
        "Start",
        "End",
        "Duration (s)",
        "Distance (m)",
        "Points",
        "Category",
        "Activity",
        "Year",
        "Month",
    ];

    let mut widths: [usize; 10] = headers.map(display_width);

    for (_folder_path, metadata) in tracks {
        let values = format_row_data(metadata);
        for (i, value) in values.iter().enumerate() {
            widths[i] = widths[i].max(display_width(value));
        }
    }

    widths
}

/// 根據顯示寬度格式化字串
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

/// 格式化軌跡資料為字串陣列
///
/// 用於表格和欄寬計算中的共同邏輯
fn format_row_data(metadata: &TrackMetadata) -> [String; 10] {
    let duration = metadata.duration_seconds();
    let distance = metadata.calculate_distance() as u64;

    [
        metadata.name.clone(),
        metadata.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        metadata.end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        duration.to_string(),
        distance.to_string(),
        metadata.coordinates.len().to_string(),
        metadata.category.clone(),
        metadata.activity.clone(),
        metadata.year.clone(),
        metadata.month.clone(),
    ]
}
