use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use ribboncurls::render;

pub fn build(
    template_path: PathBuf,
    yaml_data_path: PathBuf,
    out_path_option: Option<PathBuf>,
    is_inline: &bool,
) -> Result<()> {
    ensure_file_exists(&template_path)?;
    ensure_file_exists(&yaml_data_path)?;

    let default_out_path = append_to_filename(&template_path, ".output")?;
    let out_path = out_path_option.unwrap_or(default_out_path);
    let template = fs::read_to_string(template_path)?;
    let data = fs::read_to_string(yaml_data_path)?;
    let output = render(&template, &data, None)?;

    if *is_inline {
        println!("{}", output);
    } else {
        write_to_file(&out_path, &output)?;
        println!("Rendered output to: {}", out_path.display());
    }

    Ok(())
}

fn append_to_filename(original_path: &Path, addition: &str) -> Result<PathBuf> {
    let mut new_path = original_path.to_path_buf();

    if let Some(file_name) = original_path.file_name() {
        let new_file_name = file_name.to_string_lossy().into_owned() + addition;

        new_path.set_file_name(new_file_name);
    }

    Ok(new_path)
}

fn ensure_file_exists(file_path: &Path) -> Result<()> {
    if !file_path.is_file() {
        return Err(anyhow!(format!(
            "{} is not a valid file",
            file_path.display()
        )));
    }

    Ok(())
}

pub fn write_to_file(path: &Path, contents: &str) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("Unable to remove file: {}", path.display()))?;
    }

    let mut file =
        File::create(path).with_context(|| format!("Unable to create file: {}", path.display()))?;

    file.write_all(contents.as_bytes())?;

    Ok(())
}
