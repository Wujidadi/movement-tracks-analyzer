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
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> movement_tracks_analyzer::Result<()> {
    let args = Args::parse();
    let config = build_config(args)?;
    let placemarks = extract_placemarks_with_paths(&config.kml_file)?;
    output_results(&placemarks, &config)
}
