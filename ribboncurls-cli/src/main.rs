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
            let is_inline = sub_matches
                .get_one::<bool>("inline")
                .context("Missing default value from cli input")?;
            let out_path_option: Option<PathBuf> =
                sub_matches.get_one::<String>("out").map(PathBuf::from);
            let mustache_path = sub_matches
                .get_one::<String>("mustache-file-path")
                .map(PathBuf::from)
                .context("`mustache-file-path` is missing")?;
            let yaml_data_path = sub_matches
                .get_one::<String>("yaml-data-file-path")
                .map(PathBuf::from)
                .context("`yaml-data-file-path` is missing")?;

            render(mustache_path, yaml_data_path, out_path_option, is_inline)?;
        }
        _ => {
            println!("Basic usage: {BIN_NAME} build <mustache-file-path> <yaml-data-file-path>");
            println!("For more information try `{BIN_NAME} --help` or visit: {HOMEPAGE}");
            exit(1);
        }
    }

    Ok(())
}
