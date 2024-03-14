use clap::{builder::styling, Arg, ArgAction, ArgMatches, Command};

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
                        .required(true),
                )
                .arg(
                    Arg::new("data")
                        .short('d')
                        .help("A string of yaml data to be used when rendering")
                        .long("data")
                        .action(ArgAction::Append)
                        .required(false),
                )
                .arg(
                    Arg::new("data-file")
                        .short('f')
                        .help("Path to your yaml data file")
                        .long("data-file")
                        .action(ArgAction::Append)
                        .required(false),
                )
                .arg(
                    Arg::new("out")
                        .short('o')
                        .help("Path to the output file")
                        .value_name("FILE")
                        .long("out")
                        .action(ArgAction::Set)
                        .required(false),
                ),
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
