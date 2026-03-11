use crate::{
    cli::{Args, OutputFormatArg, OutputTypeArg},
    config::{Config, OutputType},
    path_resolver::resolve_kml_file,
};
use movement_tracks_analyzer::{OutputFormat, Result};

/// 從 CLI 參數建立設定
pub fn build_config(args: Args) -> Result<Config> {
    let output_type = match args.output {
        OutputTypeArg::Shell => OutputType::Shell,
        OutputTypeArg::File => OutputType::File,
    };

    let format = match args.format {
        OutputFormatArg::Json => OutputFormat::Json,
        OutputFormatArg::Csv => OutputFormat::Csv,
        OutputFormatArg::Tsv => OutputFormat::Tsv,
        OutputFormatArg::Table => OutputFormat::Table,
    };

    // 當 format="table" 且 output="file" 時，使用 csv 格式
    let format = if matches!(args.format, OutputFormatArg::Table)
        && matches!(output_type, OutputType::File)
    {
        OutputFormat::Csv
    } else {
        format
    };

    Ok(Config {
        kml_file: resolve_kml_file(args.file)?,
        output_type,
        format,
        export_path: args.export,
    })
}
