use std::io::Write;

#[derive(Default, Debug, Clone, PartialEq, serde::Deserialize)]
pub struct SpecFile {
    pub overview: String,
    pub tests: Vec<Test>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Deserialize)]
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
        let filename = entry.file_name().into_string().unwrap();
        if !filename.ends_with(".yml") || filename.starts_with('~') {
            continue;
        }
        let path = entry.path();

        let mod_name = filename.replace(".yml", "").replace('~', "").to_string();
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

            let desc = test
                .desc
                .lines()
                .map(|s| format!("\n            /// {s}"))
                .collect::<Vec<_>>()
                .join("");
            let template = test.template;
            let data = serde_yaml::to_string(&test.data)?;
            let partials = match test.partials {
                None => "None".to_string(),
                Some(value) => format!(r##"Some(r#"{}"#)"##, serde_yaml::to_string(&value)?),
            };
            let expected = test.expected;
            let ignored = match (mod_name.as_ref(), name.as_ref()) {
                ("partials", "standalone_indentation") => "\n            #[ignore]",
                _ => "",
            };

            write!(
                output_file,
                r##"

            #[test]{ignored}{desc}
            fn {name}() -> Result<(), Box<dyn std::error::Error>> {{
                let template = r#"{template}"#;
                let data = r#"{data}"#;
                let partials = {partials};
                let expected = r#"{expected}"#;
                let output = ribboncurls::rndr(template, data, partials)?;
                assert_eq!(output, expected);
                Ok(())
            }}"##
            )?;
        }

        writeln!(output_file, "}}")?;
    }

    Ok(())
}
