# Ribboncurls

<p align="center">
  <img
    src="https://github.com/tinted-theming/ribboncurls/blob/main/logo.png?raw=true"
    alt="Ribboncurls logo" height="481 width="800" />
</p>

[![Matrix Chat](https://img.shields.io/matrix/tinted-theming:matrix.org)](https://matrix.to/#/#tinted-theming:matrix.org)
[![Crates.io](https://img.shields.io/crates/v/ribboncurls-cli.svg)](https://crates.io/crates/ribboncurls-cli)
[![Tests](https://github.com/tinted-theming/ribboncurls/actions/workflows/ci.yml/badge.svg)](https://github.com/tinted-theming/ribboncurls/actions/workflows/ci.yml)

Ribboncurls is a [Mustache] template rendering engine written in [Rust].

There is both a Rust [Ribboncurls library] and [Ribboncurls CLI tool], depending
on your needs. You can read more about those specific projects in their
respective README.md files.

**Note**: Ribboncurls library public API is subject to change, so use
with caution.

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


## Code of Conduct

This project and everyone participating in it is governed by the
Tinted Theming [Code of Conduct].

## License

Ribboncurls is licensed under the [MPL-2.0] license.

### Third-Party Licenses

This project includes third-party code licensed under the MPL-2.0
license. See the [THIRD_PARTY_LICENSES] file for details.

[Mustache]: https://mustache.github.io
[Rust]: https://www.rust-lang.org/
[Ribboncurls library]: ribboncurls/README.md
[Ribboncurls CLI tool]: ribboncurls-cli/README.md
[MPL-2.0]: LICENSE
[Code of Conduct]: https://github.com/tinted-theming/home/blob/main/CODE_OF_CONDUCT.md
[THIRD_PARTY_LICENSES]: THIRD_PARTY_LICENSES.md
[mustache spec tests]: https://github.com/mustache/spec
