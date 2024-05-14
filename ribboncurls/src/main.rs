fn main() {
    let template = r#"Hello, {{{name}}} {{surname}}. {{#some-section}}This is a section!{{/some-section}}!"#;
    let output = match ribboncurls::new_render(template) {
        Ok(res) => res,
        Err(err) => {
            println!("error: {:?}", err);
            Vec::new()
        }
    };

    println!("output {:?}", output);
}

