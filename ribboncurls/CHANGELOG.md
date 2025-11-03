# Changelog

## Unreleased

### Changed

- Upgrade dependencies

## 0.4.1 - 2024-09-19

## Added

- Bump support from Mustache spec 1.4.1 to 1.4.2

## Fixed

- Remove unintended stdout prints.
- Fix bug where a nested inverted-section gives an error.

## 0.4.0 - 2024-09-02

## Changed

- The "feature" where properties with dots in data property name is
  supported is a bug according to the mustache spec so it's been
  removed.

## 0.3.1 - 2024-08-17

## Fixed

- Fix bug where data isn't replaced for variables including a `.`
  correctly by going up the context stack.

## 0.3.0 - 2024-06-25

## Changed

- Changed project license to MPL-2.0

## 0.2.1 - 2024-06-22

### Fixed

- Escape single quotes when escaped variable is rendered.

## 0.2.0 - 2024-06-11

### Updated

- Rewrite of library to restructure library into tokenization,
  syntaxTree and rendering steps.

## 0.1.0 - 2024-05-03

- Initial release
