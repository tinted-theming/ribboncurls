# Ribboncurls

Ribboncurls is a Rust library for rendering [mustache](https://mustache.github.io) templates.

This was initially written as a weekend project, and was originally not intended for real use,
but it currently passes 132 of the 133 tests in the [mustache spec](https://github.com/mustache/spec) v1.4.1,
not including any optional modules.

## Usage

```rust
fn main() {
    let template = r#"Hello, {{name}}!"#;
    let data = r#"{"name": "world"}"#;
    let result = ribboncurls::render(template, data, None).unwrap();
    assert_eq!(result, "Hello, world!");
}
```

## License

Ribboncurls is dual-licensed under the Apache 2.0 and MIT licenses.
