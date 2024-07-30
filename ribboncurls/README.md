# Ribboncurls

<p align="center">
  <img
    src="https://github.com/tinted-theming/ribboncurls/blob/main/logo.png?raw=true"
    alt="Ribboncurls logo" height="481 width="800" />
</p>

[![Matrix Chat](https://img.shields.io/matrix/tinted-theming:matrix.org)](https://matrix.to/#/#tinted-theming:matrix.org)
[![Crates.io](https://img.shields.io/crates/v/ribboncurls.svg)](https://crates.io/crates/ribboncurls)
[![Tests](https://github.com/tinted-theming/ribboncurls/actions/workflows/ci.yml/badge.svg)](https://github.com/tinted-theming/ribboncurls/actions/workflows/ci.yml)

Ribboncurls is a Rust library for rendering [mustache] templates.
Ribboncurls is passing all 133 of the [mustache v1.4.1 spec] tests.

**Note**: Ribboncurls library public API is subject to change, so use
with caution.

## Usage

### Basic Usage

```rust
let template = r#"Hello, {{name}}!"#;
let data = r#"{"name": "world"}"#;
let result = ribboncurls::render(template, data, None).unwrap();
assert_eq!(result, "Hello, world!");
```

### With Partials

The following is to make use of [mustache partials]:

```rust
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
```

## Mustache spec tests

We run the [mustache spec tests] against the Ribboncurls lib and we are
passing all 133 of 133 tests.

<details><summary>Tests</summary>
✅ comments::inline<br>
✅ comments::standalone<br>
✅ comments::multiline_standalone<br>
✅ comments::variable_name_collision<br>
✅ comments::multiline<br>
✅ comments::indented_inline<br>
✅ comments::indented_standalone<br>
✅ comments::indented_multiline_standalone<br>
✅ comments::surrounding_whitespace<br>
✅ comments::standalone_line_endings<br>
✅ comments::standalone_without_previous_line<br>
✅ comments::standalone_without_newline<br>
✅ delimiters::pair_behavior<br>
✅ delimiters::pair_with_padding<br>
✅ delimiters::indented_standalone_tag<br>
✅ delimiters::outlying_whitespace_inline<br>
✅ delimiters::special_characters<br>
✅ delimiters::standalone_tag<br>
✅ interpolation::ampersand_context_miss_interpolation<br>
✅ delimiters::surrounding_whitespace<br>
✅ delimiters::standalone_without_newline<br>
✅ delimiters::standalone_without_previous_line<br>
✅ interpolation::ampersand<br>
✅ delimiters::inverted_sections<br>
✅ delimiters::standalone_line_endings<br>
✅ delimiters::post_partial_behavior<br>
✅ interpolation::ampersand_integer_interpolation<br>
✅ delimiters::sections<br>
✅ interpolation::ampersand_standalone<br>
✅ interpolation::ampersand_decimal_interpolation<br>
✅ interpolation::ampersand_with_padding<br>
✅ interpolation::ampersand_null_interpolation<br>
✅ interpolation::basic_context_miss_interpolation<br>
✅ interpolation::ampersand_surrounding_whitespace<br>
✅ interpolation::basic_null_interpolation<br>
✅ interpolation::dotted_names_broken_chain_resolution<br>
✅ interpolation::basic_decimal_interpolation<br>
✅ interpolation::basic_integer_interpolation<br>
✅ interpolation::dotted_names_arbitrary_depth<br>
✅ interpolation::dotted_names_broken_chains<br>
✅ interpolation::dotted_names_ampersand_interpolation<br>
✅ interpolation::basic_interpolation<br>
✅ delimiters::partial_inheritence<br>
✅ interpolation::dotted_names_basic_interpolation<br>
✅ interpolation::dotted_names_context_precedence<br>
✅ interpolation::implicit_iterators_basic_integer_interpolation<br>
✅ interpolation::html_escaping<br>
✅ interpolation::dotted_names_initial_resolution<br>
✅ interpolation::implicit_iterators_triple_mustache<br>
✅ interpolation::interpolation_surrounding_whitespace<br>
✅ interpolation::implicit_iterators_ampersand<br>
✅ interpolation::dotted_names_triple_mustache_interpolation<br>
✅ interpolation::interpolation_standalone<br>
✅ interpolation::implicit_iterators_basic_interpolation<br>
✅ interpolation::interpolation_with_padding<br>
✅ interpolation::implicit_iterators_html_escaping<br>
✅ interpolation::triple_mustache<br>
✅ interpolation::no_interpolation<br>
✅ interpolation::triple_mustache_surrounding_whitespace<br>
✅ interpolation::triple_mustache_context_miss_interpolation<br>
✅ interpolation::triple_mustache_standalone<br>
✅ interpolation::triple_mustache_integer_interpolation<br>
✅ interpolation::triple_mustache_null_interpolation<br>
✅ interpolation::triple_mustache_with_padding<br>
✅ interpolation::triple_mustache_decimal_interpolation<br>
✅ inverted::context_misses<br>
✅ inverted::context<br>
✅ inverted::dotted_names_truthy<br>
✅ inverted::falsey<br>
✅ inverted::dotted_names_broken_chains<br>
✅ inverted::empty_list<br>
✅ inverted::dotted_names_falsey<br>
✅ inverted::list<br>
✅ inverted::doubled<br>
✅ inverted::indented_inline_sections<br>
✅ inverted::internal_whitespace<br>
✅ inverted::nested_falsey<br>
✅ inverted::padding<br>
✅ inverted::null_is_falsey<br>
✅ inverted::standalone_line_endings<br>
✅ inverted::standalone_indented_lines<br>
✅ inverted::nested_truthy<br>
✅ inverted::standalone_lines<br>
✅ inverted::truthy<br>
✅ inverted::standalone_without_previous_line<br>
✅ partials::failed_lookup<br>
✅ inverted::standalone_without_newline<br>
✅ inverted::surrounding_whitespace<br>
✅ partials::standalone_without_previous_line<br>
✅ partials::basic_behavior<br>
✅ partials::context<br>
✅ partials::padding_whitespace<br>
✅ partials::inline_indentation<br>
✅ sections::dotted_names_broken_chains<br>
✅ sections::dotted_names_falsey<br>
✅ partials::standalone_line_endings<br>
✅ partials::standalone_indentation<br>
✅ partials::nested<br>
✅ partials::surrounding_whitespace<br>
✅ sections::dotted_names_truthy<br>
✅ partials::standalone_without_newline<br>
✅ partials::recursion<br>
✅ sections::context<br>
✅ sections::empty_list<br>
✅ sections::falsey<br>
✅ sections::context_misses<br>
✅ sections::doubled<br>
✅ sections::implicit_iterator_html_escaping<br>
✅ sections::implicit_iterator_decimal<br>
✅ sections::implicit_iterator_root_level<br>
✅ sections::implicit_iterator_ampersand<br>
✅ sections::implicit_iterator_string<br>
✅ sections::implicit_iterator_integer<br>
✅ sections::implicit_iterator_array<br>
✅ sections::indented_standalone_lines<br>
✅ sections::implicit_iterator_triple_mustache<br>
✅ sections::list<br>
✅ sections::indented_inline_sections<br>
✅ sections::internal_whitespace<br>
✅ sections::padding<br>
✅ sections::deeply_nested_contexts<br>
✅ sections::standalone_lines<br>
✅ sections::nested_falsey<br>
✅ sections::parent_contexts<br>
✅ sections::standalone_line_endings<br>
✅ sections::null_is_falsey<br>
✅ sections::list_contexts<br>
✅ sections::standalone_without_newline<br>
✅ sections::surrounding_whitespace<br>
✅ sections::nested_truthy<br>
✅ sections::variable_test<br>
✅ sections::truthy<br>
✅ sections::standalone_without_previous_line<br>
</details>

## License

Ribboncurls is licensed under the [MPL-2.0] license.

### Third-Party Licenses

This project includes third-party code licensed under the MPL-2.0
license. See the [THIRD_PARTY_LICENSES] file for details.

[mustache]: https://mustache.github.io
[mustache v1.4.1 spec]: https://github.com/mustache/spec/tree/v1.4.1
[mustache partials]: https://mustache.github.io/mustache.5.html#Partials
[mustache spec tests]: https://github.com/mustache/spec
[MPL-2.0]: ../LICENSE
[THIRD_PARTY_LICENSES]: ../THIRD_PARTY_LICENSES.md
