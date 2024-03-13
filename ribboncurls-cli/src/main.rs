mod cli;
mod operations;

use crate::cli::get_matches;
use crate::operations::build::build;
use anyhow::{Context, Result};
use std::path::PathBuf;

const REPO_URL: &str = env!("CARGO_PKG_HOMEPAGE");
const REPO_NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> Result<()> {
    let cli_matches = get_matches();

    match cli_matches.subcommand() {
        Some(("build", sub_matches)) => {
            let is_inline = sub_matches
                .get_one::<bool>("inline")
                .context("Missing default value from cli input")?;
            let out_path_option: Option<PathBuf> = sub_matches
                .get_one::<String>("out")
                .map(PathBuf::from);
            let mustache_path = sub_matches
                .get_one::<String>("mustache-file-path")
                .map(PathBuf::from)
                .context("`mustche-file-path` is missing")?;
            let yaml_data_path = sub_matches
                .get_one::<String>("yaml-data-file-path")
                .map(PathBuf::from)
                .context("`yaml-data-file-path` is missing")?;

            build(mustache_path, yaml_data_path, out_path_option, is_inline)?;
        }
        _ => {
            println!(
                "Basic usage: {} build <mustache-file-path> <yaml-data-file-path>",
                REPO_NAME
            );
            println!(
                "For more information try `{} --help` or visit: {}",
                REPO_NAME, REPO_URL
            );
        }
    }

    Ok(())
}
