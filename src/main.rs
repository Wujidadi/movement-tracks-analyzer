mod cli;
mod config;
mod converter;
mod output;
mod path_resolver;

use clap::Parser;
use cli::Args;
use converter::build_config;
use movement_tracks_analyzer::extract_placemarks_with_paths;
use output::output_results;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let config = build_config(args)?;
    let placemarks = extract_placemarks_with_paths(&config.kml_file)?;
    output_results(&placemarks, &config)
}
