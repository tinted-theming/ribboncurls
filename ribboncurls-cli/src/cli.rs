use clap::{builder::styling, Arg, ArgAction, ArgMatches, Command};

fn build_cli() -> Command {
    Command::new(env!("CARGO_BIN_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            Command::new("build")
                .about("Builds the target theme template")
                .arg(
                    Arg::new("mustache-file-path")
                        .help("The path to your mustache file")
                        .required(true),
                )
                .arg(
                    Arg::new("yaml-data-file-path")
                        .help("The path to your yaml file used as variables for your mustache file")
                        .required(true),
                )
                .arg(
                    Arg::new("inline")
                        .short('i')
                        .help("Prints the output to stdout")
                        .long("inline")
                        .action(ArgAction::SetTrue),
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
