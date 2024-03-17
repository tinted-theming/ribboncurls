use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub fn render(
    template_path_str: &str,
    data_option: Option<String>,
    data_files: Vec<&str>,
    partials_paths: Vec<&String>,
    partials_content_option: Option<String>,
    out_path_option: Option<PathBuf>,
) -> Result<()> {
    // Combine content from --partials and --partials-file and read_from_string paths
    let partials = {
        let mut partials = String::new();

        for partial_path_str in partials_paths {
            let tmp_data = fs::read_to_string(partial_path_str).context(format!(
                "Unable to read partial, does it exist? \"{}\"",
                partial_path_str
            ))?;
            partials.push_str(&tmp_data);
            partials.push('\n');
        }

        if let Some(partials_content) = partials_content_option {
            partials.push_str(&partials_content);
        }

        load_partials(partials)?
    };

    // Combine data from data_option and data_files
    let data_string = {
        let data_vec = {
            let mut data_vec: Vec<String> = Vec::new();
            for data_file in data_files {
                let tmp_data = fs::read_to_string(data_file)
                    .context(format!("Unable to read data-file, does it exist? \"{}\"", data_file))?;
                data_vec.push(tmp_data);
            }

            data_vec
        };

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

    let output = ribboncurls::render(&template_result?, &data_string, Some(&partials))?;

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

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    file.write_all(contents.as_bytes())?;

    Ok(())
}

fn load_partials(partials: String) -> Result<String> {
    let partials: HashMap<String, PathBuf> = serde_yaml::from_str(&partials)?;

    let partials_with_content_result: Result<Vec<(String, String)>> = partials
        .iter()
        .map(|(key, value)| {
            fs::read_to_string(value)
                .map_err(anyhow::Error::new)
                .map(|content| (key.clone(), content))
        })
        .collect();

    let partials_with_content = partials_with_content_result?;

    let mut combined_content = String::new();
    for (key, content) in partials_with_content {
        FmtWrite::write_fmt(
            &mut combined_content,
            format_args!("{}: {}\n", key, content),
        )?;
    }

    Ok(combined_content.trim_end().to_string())
}
