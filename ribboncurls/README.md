# Ribboncurls

<p align="center">
  <img
    src="https://github.com/tinted-theming/ribboncurls/blob/main/logo.png?raw=true"
    alt="Ribboncurls logo" height="481 width="800" />
</p>

[![Matrix Chat](https://img.shields.io/matrix/tinted-theming:matrix.org)](https://matrix.to/#/#tinted-theming:matrix.org)
[![Crates.io](https://img.shields.io/crates/v/ribboncurls.svg)](https://crates.io/crates/ribboncurls)
[![Tests](https://github.com/tinted-theming/ribboncurls/actions/workflows/ci.yml/badge.svg)](https://github.com/tinted-theming/ribboncurls/actions/workflows/ci.yml)

Ribboncurls is a Rust library for rendering [Mustache] templates.
Ribboncurls is passing all 136 of the [Mustache v1.4.2 spec] tests.

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

The following is to make use of [Mustache partials]:

```rust
let partials = r#"header: <header>Some header partial</header>
footer: <footer>Footer partial content goes here</footer>"#;
let template = r#"{{> header}}
Hello, {{name}}!
{{> footer}}"#;
let data = r#"{"name": "world"}"#;
let result = ribboncurls::render(template, data, Some(partials)).unwrap();
assert_eq!(result, r#"<header>Some header partial</header>Hello, world!
<footer>Footer partial content goes here</footer>"#);
```

### Advanced usage

Have a look at the [Mustache specification] for more detailed Mustache
information. Ribboncurls support all required features mentioned there.

## Mustache spec tests

Ribboncurls runs the [Mustache spec tests] against the Ribboncurls lib
and Ribboncurls passes all required 136 of 136 tests.

<details><summary>Tests</summary>
✅ comments::indented_inline<br>
✅ comments::indented_multiline_standalone<br>
✅ comments::indented_standalone<br>
✅ comments::inline<br>
✅ comments::multiline<br>
✅ comments::multiline_standalone<br>
✅ comments::standalone<br>
✅ comments::standalone_line_endings<br>
✅ comments::standalone_without_newline<br>
✅ comments::standalone_without_previous_line<br>
✅ comments::surrounding_whitespace<br>
✅ comments::variable_name_collision<br>
✅ delimiters::indented_standalone_tag<br>
✅ delimiters::inverted_sections<br>
✅ delimiters::outlying_whitespace_inline<br>
✅ delimiters::pair_behavior<br>
✅ delimiters::pair_with_padding<br>
✅ delimiters::partial_inheritence<br>
✅ delimiters::post_partial_behavior<br>
✅ delimiters::sections<br>
✅ delimiters::special_characters<br>
✅ delimiters::standalone_line_endings<br>
✅ delimiters::standalone_tag<br>
✅ delimiters::standalone_without_newline<br>
✅ delimiters::standalone_without_previous_line<br>
✅ delimiters::surrounding_whitespace<br>
✅ interpolation::ampersand<br>
✅ interpolation::ampersand_context_miss_interpolation<br>
✅ interpolation::ampersand_decimal_interpolation<br>
✅ interpolation::ampersand_integer_interpolation<br>
✅ interpolation::ampersand_null_interpolation<br>
✅ interpolation::ampersand_standalone<br>
✅ interpolation::ampersand_surrounding_whitespace<br>
✅ interpolation::ampersand_with_padding<br>
✅ interpolation::basic_context_miss_interpolation<br>
✅ interpolation::basic_decimal_interpolation<br>
✅ interpolation::basic_integer_interpolation<br>
✅ interpolation::basic_interpolation<br>
✅ interpolation::basic_null_interpolation<br>
✅ interpolation::dotted_names_ampersand_interpolation<br>
✅ interpolation::dotted_names_arbitrary_depth<br>
✅ interpolation::dotted_names_are_never_single_keys<br>
✅ interpolation::dotted_names_basic_interpolation<br>
✅ interpolation::dotted_names_broken_chain_resolution<br>
✅ interpolation::dotted_names_broken_chains<br>
✅ interpolation::dotted_names_context_precedence<br>
✅ interpolation::dotted_names_initial_resolution<br>
✅ interpolation::dotted_names_no_masking<br>
✅ interpolation::dotted_names_triple_mustache_interpolation<br>
✅ interpolation::html_escaping<br>
✅ interpolation::implicit_iterators_ampersand<br>
✅ interpolation::implicit_iterators_basic_integer_interpolation<br>
✅ interpolation::implicit_iterators_basic_interpolation<br>
✅ interpolation::implicit_iterators_html_escaping<br>
✅ interpolation::implicit_iterators_triple_mustache<br>
✅ interpolation::interpolation_standalone<br>
✅ interpolation::interpolation_surrounding_whitespace<br>
✅ interpolation::interpolation_with_padding<br>
✅ interpolation::no_interpolation<br>
✅ interpolation::no_re_interpolation<br>
✅ interpolation::triple_mustache<br>
✅ interpolation::triple_mustache_context_miss_interpolation<br>
✅ interpolation::triple_mustache_decimal_interpolation<br>
✅ interpolation::triple_mustache_integer_interpolation<br>
✅ interpolation::triple_mustache_null_interpolation<br>
✅ interpolation::triple_mustache_standalone<br>
✅ interpolation::triple_mustache_surrounding_whitespace<br>
✅ interpolation::triple_mustache_with_padding<br>
✅ inverted::context<br>
✅ inverted::context_misses<br>
✅ inverted::dotted_names_broken_chains<br>
✅ inverted::dotted_names_falsey<br>
✅ inverted::dotted_names_truthy<br>
✅ inverted::doubled<br>
✅ inverted::empty_list<br>
✅ inverted::falsey<br>
✅ inverted::indented_inline_sections<br>
✅ inverted::internal_whitespace<br>
✅ inverted::list<br>
✅ inverted::nested_falsey<br>
✅ inverted::nested_truthy<br>
✅ inverted::null_is_falsey<br>
✅ inverted::padding<br>
✅ inverted::standalone_indented_lines<br>
✅ inverted::standalone_line_endings<br>
✅ inverted::standalone_lines<br>
✅ inverted::standalone_without_newline<br>
✅ inverted::standalone_without_previous_line<br>
✅ inverted::surrounding_whitespace<br>
✅ inverted::truthy<br>
✅ partials::basic_behavior<br>
✅ partials::context<br>
✅ partials::failed_lookup<br>
✅ partials::inline_indentation<br>
✅ partials::nested<br>
✅ partials::padding_whitespace<br>
✅ partials::recursion<br>
✅ partials::standalone_indentation<br>
✅ partials::standalone_line_endings<br>
✅ partials::standalone_without_newline<br>
✅ partials::standalone_without_previous_line<br>
✅ partials::surrounding_whitespace<br>
✅ sections::context<br>
✅ sections::context_misses<br>
✅ sections::deeply_nested_contexts<br>
✅ sections::dotted_names_broken_chains<br>
✅ sections::dotted_names_falsey<br>
✅ sections::dotted_names_truthy<br>
✅ sections::doubled<br>
✅ sections::empty_list<br>
✅ sections::falsey<br>
✅ sections::implicit_iterator_ampersand<br>
✅ sections::implicit_iterator_array<br>
✅ sections::implicit_iterator_decimal<br>
✅ sections::implicit_iterator_html_escaping<br>
✅ sections::implicit_iterator_integer<br>
✅ sections::implicit_iterator_root_level<br>
✅ sections::implicit_iterator_string<br>
✅ sections::implicit_iterator_triple_mustache<br>
✅ sections::indented_inline_sections<br>
✅ sections::indented_standalone_lines<br>
✅ sections::internal_whitespace<br>
✅ sections::list<br>
✅ sections::list_contexts<br>
✅ sections::nested_falsey<br>
✅ sections::nested_truthy<br>
✅ sections::null_is_falsey<br>
✅ sections::padding<br>
✅ sections::parent_contexts<br>
✅ sections::standalone_line_endings<br>
✅ sections::standalone_lines<br>
✅ sections::standalone_without_newline<br>
✅ sections::standalone_without_previous_line<br>
✅ sections::surrounding_whitespace<br>
✅ sections::truthy<br>
✅ sections::variable_test<br>
</details>

## License

Ribboncurls is licensed under the [MPL-2.0] license.

### Third-Party Licenses

This project includes third-party code licensed under the MPL-2.0
license. See the [THIRD_PARTY_LICENSES] file for details.

[Mustache]: https://mustache.github.io
[Mustache v1.4.2 spec]: https://github.com/mustache/spec/tree/v1.4.2
[Mustache partials]: https://mustache.github.io/mustache.5.html#Partials
[Mustache spec tests]: https://github.com/mustache/spec
[Mustache specification]: https://github.com/mustache/spec
[MPL-2.0]: ../LICENSE
[THIRD_PARTY_LICENSES]: ../THIRD_PARTY_LICENSES.md
