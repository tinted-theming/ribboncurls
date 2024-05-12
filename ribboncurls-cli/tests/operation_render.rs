use anyhow::Result;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const COMMAND_PATH: &str = "../target/release/ribboncurls";
const SUBCOMMAND_PATH: &str = "render";

#[test]
fn test_operation_render_no_data() -> Result<()> {
    // -------
    // Arrange
    // -------
    let mustache_filepath = String::from("./tests/fixtures/variables.mustache");

    // ---
    // Act
    // ---
    let (_, stderr) = run_command(vec![
        COMMAND_PATH.to_string(),
        SUBCOMMAND_PATH.to_string(),
        mustache_filepath,
    ])
    .unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stderr.contains("the following required arguments were not provided"),
        "stderr does not contain the expected output"
    );
    assert!(
        stderr
            .contains("Usage: ribboncurls render <--data <YAML_STRING>|--data-file <FILE>> <FILE>"),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_operation_render_single_data() -> Result<()> {
    // -------
    // Arrange
    // -------
    let mustache_filepath = String::from("./tests/fixtures/variables.mustache");
    let yaml_data = "name: World";

    // ---
    // Act
    // ---
    let (stdout, stderr) = run_command(vec![
        COMMAND_PATH.to_string(),
        SUBCOMMAND_PATH.to_string(),
        format!("--data={}", yaml_data),
        mustache_filepath,
    ])
    .unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains("Hello, World !"),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_operation_render_multiple_data() -> Result<()> {
    // -------
    // Arrange
    // -------
    let mustache_filepath = String::from("./tests/fixtures/variables.mustache");
    let yaml_data_name = "name: Gillian";
    let yaml_data_lastname = "lastname: Doe";

    // ---
    // Act
    // ---
    let (stdout, stderr) = run_command(vec![
        COMMAND_PATH.to_string(),
        SUBCOMMAND_PATH.to_string(),
        format!("--data={}", yaml_data_name),
        format!("--data={}", yaml_data_lastname),
        mustache_filepath,
    ])
    .unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains("Hello, Gillian Doe!"),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_operation_render_single_datafile() -> Result<()> {
    // -------
    // Arrange
    // -------
    let mustache_filepath = String::from("./tests/fixtures/variables.mustache");

    // ---
    // Act
    // ---
    let (stdout, stderr) = run_command(vec![
        COMMAND_PATH.to_string(),
        SUBCOMMAND_PATH.to_string(),
        format!(
            "--data-file={}",
            PathBuf::from("./tests/fixtures/data-name.yaml").display()
        ),
        mustache_filepath,
    ])
    .unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains("Hello, Jessica !"),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_operation_render_multiple_datafile() -> Result<()> {
    // -------
    // Arrange
    // -------
    let mustache_filepath = String::from("./tests/fixtures/variables.mustache");

    // ---
    // Act
    // ---
    let (stdout, stderr) = run_command(vec![
        COMMAND_PATH.to_string(),
        SUBCOMMAND_PATH.to_string(),
        format!(
            "--data-file={}",
            PathBuf::from("./tests/fixtures/data-name.yaml").display()
        ),
        format!(
            "--data-file={}",
            PathBuf::from("./tests/fixtures/data-lastname.yaml").display()
        ),
        mustache_filepath,
    ])
    .unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains("Hello, Jessica Doe!"),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_operation_render_mixture_of_data_and_datafile() -> Result<()> {
    // -------
    // Arrange
    // -------
    let mustache_filepath = String::from("./tests/fixtures/variables.mustache");

    // ---
    // Act
    // ---
    let (stdout, stderr) = run_command(vec![
        COMMAND_PATH.to_string(),
        SUBCOMMAND_PATH.to_string(),
        format!(
            "--data-file={}",
            PathBuf::from("./tests/fixtures/data-name.yaml").display()
        ),
        "--data=lastname: Dodrio".to_string(),
        mustache_filepath,
    ])
    .unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains("Hello, Jessica Dodrio!"),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_operation_render_using_stdin_as_template() -> Result<()> {
    // -------
    // Arrange
    // -------
    let template = "Howzit, {{name}} {{lastname}}!";
    let command_str = format!(
        "echo '{}' | {} {} --data-file='{}' --data='lastname: Dodrio' -",
        template,
        COMMAND_PATH,
        SUBCOMMAND_PATH,
        PathBuf::from("./tests/fixtures/data-name.yaml").display(),
    );

    // ---
    // Act
    // ---
    let (stdout, stderr) = run_command_through_sh(&command_str).unwrap();

    // ------
    // Assert
    // ------
    assert!(
        stdout.contains("Howzit, Jessica Dodrio!"),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    Ok(())
}

#[test]
fn test_operation_render_out() -> Result<()> {
    // -------
    // Arrange
    // -------
    let mustache_filepath = String::from("./tests/fixtures/variables.mustache");
    let yaml_data = "name: World";
    let out_path = PathBuf::from("test_operation_render_out.txt");

    // ---
    // Act
    // ---
    let (stdout, stderr) = run_command(vec![
        COMMAND_PATH.to_string(),
        SUBCOMMAND_PATH.to_string(),
        format!("--data={}", yaml_data),
        format!("--out={}", out_path.display()),
        mustache_filepath,
    ])
    .unwrap();

    println!("stdout: {}", stdout);

    // ------
    // Assert
    // ------
    assert!(
        fs::read_to_string(&out_path)?.contains("Hello, World !"),
        "stdout does not contain the expected output"
    );
    assert!(
        stdout.contains(format!("Rendered output to: {}", out_path.display()).as_str()),
        "stdout does not contain the expected output"
    );
    assert!(
        stderr.is_empty(),
        "stderr does not contain the expected output"
    );

    Ok(())
}

pub fn run_command(command_vec: Vec<String>) -> Result<(String, String), Box<dyn Error>> {
    let output = Command::new(COMMAND_PATH)
        .args(&command_vec[1..])
        .output()
        .expect("Failed to execute command");

    if !output.stderr.is_empty() {
        println!(
            "Init command stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok((
        String::from_utf8(output.stdout)?,
        String::from_utf8(output.stderr)?,
    ))
}

pub fn run_command_through_sh(command_str: &str) -> Result<(String, String), Box<dyn Error>> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command_str)
        .output()
        .expect("Failed to execute command");

    if !output.stderr.is_empty() {
        println!(
            "Init command stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok((
        String::from_utf8(output.stdout)?,
        String::from_utf8(output.stderr)?,
    ))
}
