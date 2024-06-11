# Ribboncurls

<p align="center">
  <img
    src="https://github.com/tinted-theming/ribboncurls/blob/main/logo.png?raw=true"
    alt="Ribboncurls logo" height="481 width="800" />
</p>

Ribboncurls is a Rust library for rendering [mustache] templates.

It currently passes 132 of the 133 tests in the [mustache v1.4.1 spec]
not including any optional modules.

**Note**: Ribboncurls library public API is subject to change, so use
with caution.

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
    assert_eq!(result, r#"<header>Some header partial</header>Hello, world!
<footer>Footer partial content goes here</footer>"#);
}
```

## Mustache spec tests

We run the [mustache spec tests] against the Ribboncurls lib and we are
passing all 133 of 133 tests.

<details><summary>Tests</summary>
✅ comments::inline
✅ comments::standalone
✅ comments::multiline_standalone
✅ comments::variable_name_collision
✅ comments::multiline
✅ comments::indented_inline
✅ comments::indented_standalone
✅ comments::indented_multiline_standalone
✅ comments::surrounding_whitespace
✅ comments::standalone_line_endings
✅ comments::standalone_without_previous_line
✅ comments::standalone_without_newline
✅ delimiters::pair_behavior
✅ delimiters::pair_with_padding
✅ delimiters::indented_standalone_tag
✅ delimiters::outlying_whitespace_inline
✅ delimiters::special_characters
✅ delimiters::standalone_tag
✅ interpolation::ampersand_context_miss_interpolation
✅ delimiters::surrounding_whitespace
✅ delimiters::standalone_without_newline
✅ delimiters::standalone_without_previous_line
✅ interpolation::ampersand
✅ delimiters::inverted_sections
✅ delimiters::standalone_line_endings
✅ delimiters::post_partial_behavior
✅ interpolation::ampersand_integer_interpolation
✅ delimiters::sections
✅ interpolation::ampersand_standalone
✅ interpolation::ampersand_decimal_interpolation
✅ interpolation::ampersand_with_padding
✅ interpolation::ampersand_null_interpolation
✅ interpolation::basic_context_miss_interpolation
✅ interpolation::ampersand_surrounding_whitespace
✅ interpolation::basic_null_interpolation
✅ interpolation::dotted_names_broken_chain_resolution
✅ interpolation::basic_decimal_interpolation
✅ interpolation::basic_integer_interpolation
✅ interpolation::dotted_names_arbitrary_depth
✅ interpolation::dotted_names_broken_chains
✅ interpolation::dotted_names_ampersand_interpolation
✅ interpolation::basic_interpolation
✅ delimiters::partial_inheritence
✅ interpolation::dotted_names_basic_interpolation
✅ interpolation::dotted_names_context_precedence
✅ interpolation::implicit_iterators_basic_integer_interpolation
✅ interpolation::html_escaping
✅ interpolation::dotted_names_initial_resolution
✅ interpolation::implicit_iterators_triple_mustache
✅ interpolation::interpolation_surrounding_whitespace
✅ interpolation::implicit_iterators_ampersand
✅ interpolation::dotted_names_triple_mustache_interpolation
✅ interpolation::interpolation_standalone
✅ interpolation::implicit_iterators_basic_interpolation
✅ interpolation::interpolation_with_padding
✅ interpolation::implicit_iterators_html_escaping
✅ interpolation::triple_mustache
✅ interpolation::no_interpolation
✅ interpolation::triple_mustache_surrounding_whitespace
✅ interpolation::triple_mustache_context_miss_interpolation
✅ interpolation::triple_mustache_standalone
✅ interpolation::triple_mustache_integer_interpolation
✅ interpolation::triple_mustache_null_interpolation
✅ interpolation::triple_mustache_with_padding
✅ interpolation::triple_mustache_decimal_interpolation
✅ inverted::context_misses
✅ inverted::context
✅ inverted::dotted_names_truthy
✅ inverted::falsey
✅ inverted::dotted_names_broken_chains
✅ inverted::empty_list
✅ inverted::dotted_names_falsey
✅ inverted::list
✅ inverted::doubled
✅ inverted::indented_inline_sections
✅ inverted::internal_whitespace
✅ inverted::nested_falsey
✅ inverted::padding
✅ inverted::null_is_falsey
✅ inverted::standalone_line_endings
✅ inverted::standalone_indented_lines
✅ inverted::nested_truthy
✅ inverted::standalone_lines
✅ inverted::truthy
✅ inverted::standalone_without_previous_line
✅ partials::failed_lookup
✅ inverted::standalone_without_newline
✅ inverted::surrounding_whitespace
✅ partials::standalone_without_previous_line
✅ partials::basic_behavior
✅ partials::context
✅ partials::padding_whitespace
✅ partials::inline_indentation
✅ sections::dotted_names_broken_chains
✅ sections::dotted_names_falsey
✅ partials::standalone_line_endings
✅ partials::standalone_indentation
✅ partials::nested
✅ partials::surrounding_whitespace
✅ sections::dotted_names_truthy
✅ partials::standalone_without_newline
✅ partials::recursion
✅ sections::context
✅ sections::empty_list
✅ sections::falsey
✅ sections::context_misses
✅ sections::doubled
✅ sections::implicit_iterator_html_escaping
✅ sections::implicit_iterator_decimal
✅ sections::implicit_iterator_root_level
✅ sections::implicit_iterator_ampersand
✅ sections::implicit_iterator_string
✅ sections::implicit_iterator_integer
✅ sections::implicit_iterator_array
✅ sections::indented_standalone_lines
✅ sections::implicit_iterator_triple_mustache
✅ sections::list
✅ sections::indented_inline_sections
✅ sections::internal_whitespace
✅ sections::padding
✅ sections::deeply_nested_contexts
✅ sections::standalone_lines
✅ sections::nested_falsey
✅ sections::parent_contexts
✅ sections::standalone_line_endings
✅ sections::null_is_falsey
✅ sections::list_contexts
✅ sections::standalone_without_newline
✅ sections::surrounding_whitespace
✅ sections::nested_truthy
✅ sections::variable_test
✅ sections::truthy
✅ sections::standalone_without_previous_line
</details>

## License

Ribboncurls is dual-licensed under the Apache 2.0 and MIT licenses.

[mustache]: https://mustache.github.io
[mustache v1.4.1 spec]: https://github.com/mustache/spec/tree/v1.4.1
[mustache partials]: https://mustache.github.io/mustache.5.html#Partials
[mustache spec tests]: https://github.com/mustache/spec
