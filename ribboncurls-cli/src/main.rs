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

    if let Some(("render", sub_matches)) = cli_matches.subcommand() {
        let out_path_option: Option<PathBuf> =
            sub_matches.get_one::<String>("out").map(PathBuf::from);
        let mustache_input = sub_matches
            .get_one::<String>("mustache-file-path")
            .context("`mustache-file-path` is missing")?;
        let data_files = sub_matches
            .get_many::<String>("data-file")
            .unwrap_or_default()
            .map(String::as_str)
            .collect::<Vec<&str>>();
        let cli_data = sub_matches
            .get_many::<String>("data")
            .unwrap_or_default()
            .map(String::as_str)
            .collect::<Vec<&str>>();
        let data = if cli_data.is_empty() {
            None
        } else {
            Some(cli_data.join("\n"))
        };
        let cli_partials_with_filepath = sub_matches
            .get_many::<String>("partial-file")
            .unwrap_or_default()
            .map(String::as_str)
            .collect::<Vec<&str>>();
        let cli_partials = sub_matches
            .get_many::<String>("partials")
            .unwrap_or_default()
            .collect::<Vec<&String>>();
        let partials_with_filepath = if cli_partials_with_filepath.is_empty() {
            None
        } else {
            Some(cli_partials_with_filepath.join("\n"))
        };

        render(
            mustache_input,
            data,
            data_files,
            cli_partials,
            partials_with_filepath,
            out_path_option,
        )?;
    } else {
        println!("Basic usage: {BIN_NAME} render <mustache-file-path> <yaml-data-file-path>");
        println!("For more information try `{BIN_NAME} --help` or visit: {HOMEPAGE}");
        exit(1);
    }

    Ok(())
}
