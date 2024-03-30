# Ribboncurls

Ribboncurls is a Rust library for rendering [mustache] templates.

This was initially written as a weekend project, and was originally not
intended for real use, but it currently passes 132 of the 133 tests in
the [mustache v1.4.1 spec] not including any optional modules.

## Usage

### Basic Usage

```rust
fn main() {
    let template = r#"Hello, {{name}}!"#;
    let data = r#"{"name": "world"}"#;
    let result = ribboncurls::render(template, data, None).unwrap();
    assert_eq!(result, "Hello, world!");
}
```

### With Partials

The following is to make use of [mustache partials]:

```rust
fn main() {
    let partials = r#"
header: <header>Some header partial</header>
footer: <footer>Footer partial content goes here</footer>"#;
    let template = r#"{{> header}}
Hello, {{name}}!
{{> footer}}"#;
    let data = r#"{"name": "world"}"#;
    let result = ribboncurls::render(template, data, Some(partials)).unwrap();
    assert_eq!(result, r#"<header> Some header partial</header>
Hello, world!
<footer>Footer partial content goes here</footer>"#);
}
```

## License

Ribboncurls is dual-licensed under the Apache 2.0 and MIT licenses.

[mustache]: https://mustache.github.io
[mustache v1.4.1 spec]: https://github.com/mustache/spec/tree/v1.4.1
[mustache partials]: https://mustache.github.io/mustache.5.html#Partials
