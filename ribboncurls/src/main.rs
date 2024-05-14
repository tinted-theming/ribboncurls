use std::collections::HashMap;

fn main() {
    let data = HashMap::from([
        ("name".to_string(), "Tinted".to_string()),
        ("surname".to_string(), "Theming".to_string()),
        ("url".to_string(), ">> https://github.com/tinted-theming".to_string()),
    ]);
    let template = r#"
    Hello,  {{^section}}This is a section!{{/section}}! {{{name}}} {{surname}}. {{#some-section}}This is a section!{{/some-section}}! - {{url}}"#;
    let output = match ribboncurls::new_render(template, data) {
        Ok(res) => res,
        Err(err) => {
            println!("error: {:?}", err);
            String::new()
        }
    };

    println!("output {:?}", output);
}

