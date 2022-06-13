extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use clap::Command as ClapCommand;
use clap::Arg;
use std::env;
use std::process::Command;

pub(crate) mod config {
    pub(crate) mod config;
}
pub(crate) mod matchers {
    pub(crate) mod json;
    pub(crate) mod headers;
}
pub(crate) mod webhook {
    pub(crate) mod serve;
}

mod logging;

const APPLICATION_NAME: &str = "pagoo";

fn application_commands<'a>() -> Vec<clap::Command<'a>> {
    vec![
        webhook::serve::command_config(),
    ]
}

fn main() {
    let build_metadata = include_str!("../build_metadata.txt").trim().replace("\n", "");
    let build_metadata_str = build_metadata.as_str();
    let version = if build_metadata == "" { "dev" } else { build_metadata_str };

    let application_commands = application_commands();

    let app = get_app(version).subcommands(application_commands);

    let matches = app.get_matches();
    let mut config_file_value = matches.value_of("config-file");
    if config_file_value.is_some() && config_file_value.unwrap() == "" {
        config_file_value = None;
    }

    let verbose_value = matches.indices_of("verbose").unwrap_or_default();
    let is_quiet = matches.index_of("quiet").unwrap_or_default() > 0;

    logging::set_verbosity_value(verbose_value.len(), is_quiet);

    let subcommand_name = matches.subcommand_name();

    match subcommand_name {
        Some(webhook::serve::COMMAND_NAME) => {
            webhook::serve::serve(config_file_value, matches.subcommand_matches(&subcommand_name.unwrap()).unwrap()).unwrap();
        },
        _ => {
            default_command();
        }
    };
}

fn get_app(version: &str) -> ClapCommand {
    ClapCommand::new(APPLICATION_NAME)
        .version(version)
        .author("Alex \"Pierstoval\" Rock <alex@orbitale.io>")
        .about("A tool to manage your local CI/CD/etc setup")
        .arg(
            Arg::new("config-file")
                .short('c')
                .long("config-file")
                .multiple_occurrences(false)
                .multiple_values(false)
                .takes_value(true)
                .help("Specify the config file to use for this instance."),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .multiple_occurrences(true)
                .multiple_values(false)
                .takes_value(false)
                .help("Set the verbosity level. -v for debug, -vv for trace, -vvv to trace executed modules"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .takes_value(false)
                .help("Do not display any output. Has precedence over -v|--verbose"),
        )
}

fn default_command() {
    let process_args: Vec<String> = env::args().collect();
    let current_process_name = process_args[0].as_str().to_owned();

    // If no subcommand is specified,
    // re-run the program with "--help"
    let mut subprocess = Command::new(&current_process_name)
        .arg("--help")
        .spawn()
        .expect("Failed to create the \"help\" command.");

    subprocess
        .wait()
        .expect("Failed to run the \"help\" command.");
}
