fn main() {
    let data = r#"name: Tinted
surname: Theming
url: https://github.com/tinted-theming
"#;
    // let template = r#"
    // Hello,  {{^section}}This is a section!{{/section}}! {{{name}}} {{surname}}. {{#some-section}}This is a section!{{/some-section}}! - {{url}}"#;
    let template = "I ({{cannot}}) be seen!";
    let output = match ribboncurls::rndr(template, data, None) {
        Ok(res) => res,
        Err(err) => {
            println!("error: {:?}", err);
            String::new()
        }
    };

    println!("output {:?}", output);
}

