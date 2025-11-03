use std::fmt::Write;
use std::io::Write as _;
use std::path::Path;

#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct SpecFile {
    pub overview: String,
    pub tests: Vec<Test>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Test {
    pub name: String,
    pub desc: String,
    pub data: serde_yaml::Value,
    pub partials: Option<serde_yaml::Value>,
    pub template: String,
    pub expected: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;
    let destination = std::path::Path::new(&out_dir).join("from_specs.rs");
    let mut output_file = std::fs::File::create(destination)?;

    let spec_dir = std::fs::read_dir("./vendor/github.com/mustache/spec/specs")?;
    for entry_res in spec_dir {
        let entry = entry_res?;
        let filename_str = &entry
            .file_name()
            .into_string()
            .expect("Unable to get filename");
        let filename = Path::new(&filename_str);
        if !filename.extension().map_or(false, |ext| ext == "yml") || filename_str.starts_with('~')
        {
            continue;
        }
        let path = entry.path();

        let mod_name = filename_str.replace(".yml", "").replace('~', "");
        writeln!(output_file, "mod {mod_name} {{")?;

        let spec_file: SpecFile = serde_yaml::from_reader(std::fs::File::open(path)?)?;
        for test in spec_file.tests {
            let mut name = test
                .name
                .to_lowercase()
                .replace([' ', '(', ')', '-'], "_")
                .replace("___", "_")
                .replace("__", "_");
            if name.ends_with('_') {
                name.pop();
            }

            let desc = test.desc.lines().fold(String::new(), |mut output, b| {
                let _ = write!(output, "\n            /// {b}");
                output
            });
            let template = test.template;
            let data = serde_yaml::to_string(&test.data)?;
            let partials = match test.partials {
                None => "None".to_string(),
                Some(value) => format!(r##"Some(r#"{}"#)"##, serde_yaml::to_string(&value)?),
            };
            let expected = test.expected;

            write!(
                output_file,
                r##"

            #[test]{desc}
            fn {name}() -> Result<(), Box<dyn std::error::Error>> {{
                let template = r#"{template}"#;
                let data = r#"{data}"#;
                let partials = {partials};
                let expected = r#"{expected}"#;
                let output = ribboncurls::render(template, data, partials)?;
                assert_eq!(output, expected);
                Ok(())
            }}"##
            )?;
        }

        writeln!(output_file, "}}")?;
    }

    Ok(())
}
