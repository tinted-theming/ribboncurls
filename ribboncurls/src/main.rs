fn main() {
    let data = r#"name: Tinted
surname: Theming
url: https://github.com/tinted-theming
section: true
data: "I got interpolated."
"#;
    // let template = r#"
    // Hello,  {{^section}}This is a section!{{/section}}! {{{name}}} {{surname}}. {{#some-section}}This is a section!{{/some-section}}! - {{url}}"#;
    let template = r#"=\n  {{=@ @=}}"#;
    let output = match ribboncurls::rndr(template, data, None) {
        Ok(res) => res,
        Err(err) => {
            println!("error: {:?}", err);
            String::new()
        }
    };

    println!("output {:?}", output);
}
