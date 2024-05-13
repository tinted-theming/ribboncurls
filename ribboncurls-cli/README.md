# Ribboncurls CLI

<p align="center">
  <img
    src="https://github.com/tinted-theming/ribboncurls/blob/main/logo.png?raw=true"
    alt="Ribboncurls logo" height="481 width="800" />
</p>

Ribboncurls CLI is a tool for rendering [mustache] templates. 

## Table of Contents

- [CLI](#cli)
  - [Installation](#installation)
  - [Usage](#usage)
  - [Commands](#commands)
  - [Flags](#flags)
- [Contributing](#contributing)
- [License](#license)

## Installation

### Cargo

```sh
cargo install ribboncurls-cli
```

### Homebrew

```sh
brew tap tinted-theming/tinted
brew install ribboncurls
```

### Binaries

Download the relevant binary from the [repository releases] page.

## Usage

### Generate a Mustache file with a YAML data file

The following takes in `./path/to/page.html.mustache` template file and
generates `./page.html` using `./path/to/data.yaml`:

```sh
ribboncurls render ./path/to/page.html.mustache \
  --data-file="./path/to/data.yaml" \
  --out="./page.html"
```

### Use stdin and stdout

```sh
echo "Hello, {{name}}!" | ribboncurls render - --data="name: World" > ./hello-world-example.txt
```

## Commands

The following is a table of the available subcommands for the CLI tool, including the descriptions and any notable arguments.

| Subcommand | Description                          | Arguments            | Example Usage                              |
|------------|--------------------------------------|----------------------|--------------------------------------------|
| `render`  | Renders the Mustache template with provided data. | `mustache_file_path`: Path to mustache file or `-` to accept stdin. | `ribboncurls render ./path/to/file.mustache` or `echo "Hello, {{name}}!" | ribboncurls render --data="name: World" -` |

## Flags

| Flag/Option       | Description                             | Required | Repeat flag | Applicable Subcommands | Example Usage                             |
|-------------------|-----------------------------------------|----------|-------------|------------------------|-------------------------------------------|
| `--data` `-d` | A string of YAML data to be used when rendering. | `--data` and/or `--data-file` | Repeat | `render` | `ribboncurls render /path/to/file.mustache --data="name: some_first_name"` |
| `--data-file` `-f` | Path to your YAML data file. | `--data` and/or `--data-file` | Repeat | `render` | `ribboncurls render /path/to/file.mustache --data-file="/path/to/custom/data-file.yaml"` |
| `--partials` `-p` | A path to a file that contains YAML partial data. | Optional | Repeat | `render` | `ribboncurls path/to/file.mustache --partials="path/to/partials-file.yaml" --partials="path/to/some/other/file.yaml"` |
| `--partial-file` `-f` | YAML data containing a \"partial\" property name and \"partial\" value (path to file to use as partial). | Optional | Repeat | `render` | `ribboncurls render path/to/file.mustache --partial-file="property_name: path/to/file.mustache"` |
| `--out` `-o` | Writes stdout to a file. | Optional | No repeat | `render` | `ribboncurls render /path/to/file.mustache" --out="./output.html"` |
| `--help` `-h`     | Displays help information for the subcommand. | Optional | No repeat | All | `ribboncurls --help`, `ribboncurls render --help`, etc |
| `--version` `-V`  | Displays the current `ribboncurls-cli` version. | Optional | No repeat | All | `ribboncurls --version` |

### Repeat flag

Some flags may be repeated, for example:

```sh
ribboncurls render ./file.mustache \
  --data="name: Gillian" \
  --data="surname: Doe" \
  --data-file="./path/to/datafile1.yaml" \
  --data-file="./path/to/datafile2.yaml"
```

## Contributing

Contributions are welcome! Have a look at [CONTRIBUTING.md] for more
information.

## License

Ribboncurls is dual-licensed under the Apache 2.0 and MIT licenses.

[mustache]: https://mustache.github.io
[mustache v1.4.1 spec]: https://github.com/mustache/spec/tree/v1.4.1
[mustache partials]: https://mustache.github.io/mustache.5.html#Partials
[repository releases]: https://github.com/tinted-theming/ribboncurls/releases/latest
[CONTRIBUTING.md]: CONTRIBUTING.md
