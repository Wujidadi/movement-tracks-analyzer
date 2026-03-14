use crate::{
    cli::{Args, OutputFormatArg, OutputTypeArg},
    config::{Config, OutputType},
    path_resolver::resolve_kml_file,
};
use movement_tracks_analyzer::{OutputFormat, Result};

/// 從 CLI 參數建立設定
pub fn build_config(args: Args) -> Result<Config> {
    let output_type = map_output_type(args.output);
    let format = resolve_format(args.format, output_type);

    Ok(Config {
        kml_file: resolve_kml_file(args.file)?,
        output_type,
        format,
        export_path: args.export,
    })
}

/// 將 CLI 輸出目標參數映射為內部型別
fn map_output_type(arg: OutputTypeArg) -> OutputType {
    match arg {
        OutputTypeArg::Shell => OutputType::Shell,
        OutputTypeArg::File => OutputType::File,
    }
}

/// 將 CLI 格式參數映射為內部型別
fn map_output_format(arg: OutputFormatArg) -> OutputFormat {
    match arg {
        OutputFormatArg::Json => OutputFormat::Json,
        OutputFormatArg::Csv => OutputFormat::Csv,
        OutputFormatArg::Tsv => OutputFormat::Tsv,
        OutputFormatArg::Table => OutputFormat::Table,
    }
}

/// 解析最終輸出格式（表格 + 檔案輸出時自動降級為 CSV）
fn resolve_format(format_arg: OutputFormatArg, output_type: OutputType) -> OutputFormat {
    if matches!(format_arg, OutputFormatArg::Table) && matches!(output_type, OutputType::File) {
        OutputFormat::Csv
    } else {
        map_output_format(format_arg)
    }
}
