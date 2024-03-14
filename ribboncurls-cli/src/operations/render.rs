use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

pub fn render(
    template_path_str: &str,
    data_option: Option<String>,
    data_files: Vec<&str>,
    out_path_option: Option<PathBuf>,
) -> Result<()> {
    // Create Vec<String> of file content by reading from data_files
    let mut data_vec: Vec<String> = Vec::new();
    for data_file in data_files {
        let tmp_data = fs::read_to_string(data_file)
            .context(format!("Unable to read data-file: {}", data_file))?;
        data_vec.push(tmp_data);
    }

    // Combine data from data_option and data_files
    let data_string = {
        let data_inline = data_option.unwrap_or_default();
        if data_inline.is_empty() && data_vec.is_empty() {
            return Err(anyhow!(
                "No data has been provided or the provided data is empty"
            ));
        }

        let data = format!("{}\n{}", data_inline, data_vec.join("\n"));

        data
    };

    // Read template from stdin if value is `-` otherwise attemp to
    // locate and read from system file
    let template_result: Result<String> = if template_path_str == "-" {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;

        Ok(input)
    } else {
        let template_path = PathBuf::from(template_path_str);
        ensure_file_exists(&template_path)?;

        fs::read_to_string(&template_path).context(format!(
            "Unable to read from file: {}",
            &template_path.display()
        ))
    };

    let output = ribboncurls::render(&template_result?, &data_string, None)?;

    match out_path_option {
        Some(out_path) => {
            write_to_file(&out_path, &output)?;
            println!("Rendered output to: {}", out_path.display());
        }
        None => {
            let stdout = io::stdout();
            let mut stdout_handle = stdout.lock();

            writeln!(stdout_handle, "{}", output)?;
        }
    }

    Ok(())
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
