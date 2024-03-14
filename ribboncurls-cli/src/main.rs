mod cli;
mod operations;

use crate::cli::get_matches;
use crate::operations::render::render;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::exit;

const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
const BIN_NAME: &str = env!("CARGO_BIN_NAME");

fn main() -> Result<()> {
    let cli_matches = get_matches();

    match cli_matches.subcommand() {
        Some(("render", sub_matches)) => {
            let out_path_option: Option<PathBuf> =
                sub_matches.get_one::<String>("out").map(PathBuf::from);
            let mustache_input = sub_matches
                .get_one::<String>("mustache-file-path")
                .context("`mustache-file-path` is missing")?;
            let data_files = sub_matches
                .get_many::<String>("data-file")
                .unwrap_or_default()
                .map(|value| value.as_str())
                .collect::<Vec<&str>>();
            let cli_data = sub_matches
                .get_many::<String>("data")
                .unwrap_or_default()
                .map(|value| value.as_str())
                .collect::<Vec<&str>>();
            let data = if !cli_data.is_empty() {
                Some(cli_data.join("\n"))
            } else {
                None
            };

            render(mustache_input, data, data_files, out_path_option)?;
        }
        _ => {
            println!("Basic usage: {BIN_NAME} build <mustache-file-path> <yaml-data-file-path>");
            println!("For more information try `{BIN_NAME} --help` or visit: {HOMEPAGE}");
            exit(1);
        }
    }

    Ok(())
}
