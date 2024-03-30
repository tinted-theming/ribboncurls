use clap::{builder::styling, Arg, ArgAction, ArgGroup, ArgMatches, Command};

fn build_cli() -> Command {
    Command::new(env!("CARGO_BIN_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            Command::new("render")
                .about("Renders the target theme template")
                .arg(
                    Arg::new("mustache-file-path")
                        .help("The path to your mustache file, or read stdin with -")
                        .index(1)
                        .value_name("FILE")
                        .required(true),
                )
                .arg(
                    Arg::new("data")
                        .short('d')
                        .help("A string of YAML data to be used when rendering")
                        .long("data")
                        .action(ArgAction::Append)
                        .value_name("YAML_STRING")
                        .required(false),
                )
                .arg(
                    Arg::new("data-file")
                        .short('f')
                        .help("Path to your YAML data file")
                        .long("data-file")
                        .action(ArgAction::Append)
                        .value_name("FILE")
                        .required(false),
                )
                .arg(
                    Arg::new("partials")
                        .short('p')
                        .value_name("FILE")
                        .help("A path to a file that contains YAML partial data")
                        .long("partials")
                        .action(ArgAction::Append)
                        .value_name("FILE")
                        .required(false),
                )
                .arg(
                    Arg::new("partial-file")
                        .short('r')
                        .help("YAML data containing a \"partial\" property name and \"partial\" value (path to file to use as partial). Eg: `property_name: path/to/file.mustache`")
                        .long("partial-file")
                        .action(ArgAction::Append)
                        .value_name("YAML_STRING")
                        .required(false),
                )
                .arg(
                    Arg::new("out")
                        .short('o')
                        .help("Path to the output file")
                        .value_name("FILE")
                        .long("out")
                        .action(ArgAction::Set)
                        .value_name("OUTFILE")
                        .required(false),
                )
                .group(ArgGroup::new("required_flags")
                    .args(["data", "data-file"])
                    .required(true)
                    .multiple(true)),
        )
}

pub fn get_matches() -> ArgMatches {
    let styles = styling::Styles::styled()
        .header(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .usage(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .literal(styling::AnsiColor::Blue.on_default() | styling::Effects::BOLD)
        .placeholder(styling::AnsiColor::Cyan.on_default());

    build_cli().styles(styles).get_matches()
}
