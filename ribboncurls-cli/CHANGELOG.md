# Changelog

## Unreleased

### Fixed

- In tests, use `CARGO_BIN_EXE_ribboncurls` env var for the binary path
  instead of assuming it exists in `./target/release` directory

## [0.4.1] - 2024-09-19

### Fixed

- Update to latest `ribboncurls` which fixes bug where a nested
  inverted-section gives an error.

## [0.4.0] - 2024-09-02

### Removed

- Update to latest `ribboncurls` library, where a previously existing
  "feature" has been removed because the feature, allowing for mustache
  data properties to include dots in the name, is disallowed in the
  mustache spec.

## [0.3.2] - 2024-08-17

### Changed

- Update to latest `ribboncurls` dependency which includes bug fixes

## [0.3.1] - 2024-07-13

### Changed

- Add musl and darwin-universal binaries to release build
- Update deps

## [0.3.0] - 2024-06-25

### Changed

- Changed project license to MPL-2.0

## [0.2.1] - 2024-06-22

### Fixed

- Escape single quotes when escaped variable is rendered.

## [0.2.0] - 2024-06-11

### Updated

- Update to use the rewritten ribboncurls lib `0.2.0`

## [0.1.0] - 2024-05-13

### Added

- Initial release

[0.4.1]: https://github.com/tinted-theming/ribboncurls/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/tinted-theming/ribboncurls/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/tinted-theming/ribboncurls/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/tinted-theming/ribboncurls/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/tinted-theming/ribboncurls/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/tinted-theming/ribboncurls/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/tinted-theming/ribboncurls/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/tinted-theming/ribboncurls/compare/v0.1.0
