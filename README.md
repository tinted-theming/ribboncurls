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

There is both a Rust [Ribboncurls library] and [Ribboncurls CLI tool],
depending on your needs. You can read more about those specific projects
in their respective README.md files.

**Note**: Ribboncurls library public API is subject to change, so use
with caution.

## Features

Ribboncurls supports all the required Mustache features as per [Mustache
spec] and runs against the [Mustache v1.4.2 spec] tests.

- [x] Comments: `12345{{! Comment Block! }}67890`
- [x] Delimiters: Change delimiters from `{{ }}` to your own custom
  delimiters
  - `{{=<% %>=}}(<%text%>)`
- [x] Interpolation 
  - `{{text}}` - Renders escaped variable `text`
  - `{{&text}}`, `{{{text}}}` - Renders variable `text` without
    escaping
  - `{{a.b.c.d}}` - Renders HashMap properties
  - and more
- [x] Sections 
  - `{{#text}}Renders when text exists{{/text}}` - Renders section
    when `text` is truthy
  - `{{#animals_array}}({{.}}){{/list}}` - Implicit iterator
- [x] Inverted: Functionally serves as inverted sections
  - `{{^text}}Renders when text does not exist{{/text}}`
- [x] Partials: Used to expand an external template into the current
  template
  - `{{>partial_property}}`

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
[Mustache spec]: https://github.com/mustache/spec
[mustache v1.4.2 spec]: https://github.com/mustache/spec/tree/v1.4.2
