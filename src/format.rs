use crate::TrackMetadata;
use serde::Serialize;
use unicode_width::UnicodeWidthStr;

/// 時間格式化字串常數
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// 表格標題
const TABLE_HEADERS: [&str; 10] = [
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

/// 根據指定格式產生輸出字串
///
/// 支援多種輸出格式，適應不同的使用場景。
///
/// # Arguments
///
/// * `format` - 輸出格式（JSON、CSV、TSV、Table）
/// * `tracks` - 軌跡資料陣列，包含路徑和詮釋資料
///
/// # Returns
///
/// 格式化後的字串，可直接用於顯示或保存到檔案
///
/// # Format Details
///
/// - **Json**：結構化的 JSON 格式，適合程式化處理
/// - **Csv**：逗號分隔值，適合 Excel 等電子表格
/// - **Tsv**：Tab 分隔值，避免資料中的逗號引起混亂
/// - **Table**：命令行表格格式，支援 Unicode 字元對齊（漢字寬度=2）
///
/// # Example
///
/// ```
/// use movement_tracks_analyzer::{extract_placemarks_with_paths, format_output, OutputFormat};
/// use std::path::PathBuf;
///
/// let placemarks = extract_placemarks_with_paths(&PathBuf::from("tests/fixtures/tracks.kml"))?;
///
/// // 輸出為 JSON
/// let json_output = format_output(OutputFormat::Json, &placemarks);
/// assert!(json_output.contains("2026-03"));
///
/// // 輸出為 CSV
/// let csv_output = format_output(OutputFormat::Csv, &placemarks);
/// assert!(csv_output.contains("Start,End"));
///
/// // 輸出為表格（命令行展示）
/// let table_output = format_output(OutputFormat::Table, &placemarks);
/// assert!(table_output.contains("Distance"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
    let header =
        "Name,Start,End,Duration (seconds),Distance (meters),Points,Category,Activity,Year,Month\n";
    let rows: String = tracks
        .iter()
        .map(|(_, metadata)| format_csv_row(metadata))
        .collect();
    format!("{}{}", header, rows)
}

/// 格式化單筆 CSV 資料列
fn format_csv_row(metadata: &TrackMetadata) -> String {
    let (duration, distance) = calculate_metrics(metadata);
    let (start_time, end_time) = format_timestamp(metadata);
    format!(
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
    )
}

/// 輸出 TSV 格式
fn format_tsv(tracks: &[(Vec<String>, TrackMetadata)]) -> String {
    let header = "Name\tStart\tEnd\tDuration (seconds)\tDistance (meters)\tPoints\tCategory\tActivity\tYear\tMonth\n";
    let rows: String = tracks
        .iter()
        .map(|(_, metadata)| format_tsv_row(metadata))
        .collect();
    format!("{}{}", header, rows)
}

/// 格式化單筆 TSV 資料列
fn format_tsv_row(metadata: &TrackMetadata) -> String {
    let (duration, distance) = calculate_metrics(metadata);
    let (start_time, end_time) = format_timestamp(metadata);
    format!(
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
    )
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
    format_header_row(&mut output, &widths);
    format_separator_row(&mut output, &widths);
    format_data_rows(&mut output, tracks, &widths);
    output
}

/// 格式化表格標題列
fn format_header_row(output: &mut String, widths: &[usize; 10]) {
    for (i, header) in TABLE_HEADERS.iter().enumerate() {
        output.push_str(&format_cell(header, widths[i], i));
        output.push(' ');
    }
    output.push('\n');
}

/// 格式化表格分隔線
fn format_separator_row(output: &mut String, widths: &[usize; 10]) {
    for width in widths {
        output.push_str(&"-".repeat(*width));
        output.push(' ');
    }
    output.push('\n');
}

/// 格式化表格資料列
fn format_data_rows(
    output: &mut String,
    tracks: &[(Vec<String>, TrackMetadata)],
    widths: &[usize; 10],
) {
    for (_folder_path, metadata) in tracks {
        format_single_data_row(output, metadata, widths);
    }
}

/// 格式化單筆表格資料列
fn format_single_data_row(output: &mut String, metadata: &TrackMetadata, widths: &[usize; 10]) {
    let row_data = format_row_data(metadata);
    for (i, data) in row_data.iter().enumerate() {
        output.push_str(&format_cell(data, widths[i], i));
        output.push(' ');
    }
    output.push('\n');
}

/// 計算字串顯示寬度（考慮 Unicode 寬字元）
fn display_width(s: &str) -> usize {
    s.width()
}

/// 計算欄寬（用於表格輸出）
fn calculate_column_widths(tracks: &[(Vec<String>, TrackMetadata)]) -> [usize; 10] {
    let mut widths: [usize; 10] = TABLE_HEADERS.map(display_width);
    for (_folder_path, metadata) in tracks {
        update_widths_from_row(&mut widths, metadata);
    }
    widths
}

/// 根據資料列更新各欄位寬度
fn update_widths_from_row(widths: &mut [usize; 10], metadata: &TrackMetadata) {
    let values = format_row_data(metadata);
    for (i, value) in values.iter().enumerate() {
        widths[i] = widths[i].max(display_width(value));
    }
}

/// 判斷是否為數值欄位（靠右對齊）
fn is_right_aligned_column(col_index: usize) -> bool {
    matches!(col_index, 3 | 4 | 5)
}

/// 根據顯示寬度格式化字串
fn format_cell(text: &str, width: usize, col_index: usize) -> String {
    let text_width = display_width(text);
    if text_width >= width {
        return text.to_string();
    }
    pad_text(text, width - text_width, is_right_aligned_column(col_index))
}

/// 依據對齊方式填充文字
fn pad_text(text: &str, padding_size: usize, right_align: bool) -> String {
    let padding = " ".repeat(padding_size);
    if right_align {
        format!("{}{}", padding, text)
    } else {
        format!("{}{}", text, padding)
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
