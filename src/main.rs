extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use clap::ArgMatches;
use clap::Command as ClapCommand;
use clap::Arg;
use clap::ArgAction;
use std::env;
use std::process::Command;
use std::process::Termination;
use std::process::ExitCode;
use crate::commands::serve_webhook;

mod actions {
    pub(crate) mod executor;
    pub(crate) mod matching_webhooks;
}

mod api {
    pub(crate) mod webhook;
}

mod config;

mod commands {
    pub(crate) mod serve_webhook;
}

mod db;

mod logging;

mod matchers {
    pub(crate) mod headers;
    pub(crate) mod json;
}

mod serve;

#[cfg(test)]
mod test_utils;

const APPLICATION_NAME: &str = "pagoo";
const APP_VERSION_METADATA: &'static str = include_str!("../.version");

fn main() -> ReturnExitCode {
    let application_commands = application_commands();

    let subcommands = application_commands.subcommands().into_iter();

    let app = get_app().subcommands(subcommands);

    let arg_matches = app.get_matches();
    let mut config_file = arg_matches.get_one::<String>("config-file").map(|s|s.as_str());
    if config_file.is_some() && config_file.unwrap() == "" {
        config_file = None;
    }

    let verbosity_level: &u8 = arg_matches.get_one::<u8>("verbose").unwrap_or(&0);
    let is_quiet = arg_matches.get_flag("quiet");

    logging::set_verbosity_value(*verbosity_level, is_quiet);

    let subcommand_name = arg_matches.subcommand_name();
    let args = if subcommand_name.is_some() {
        arg_matches.subcommand_matches(&subcommand_name.unwrap())
    } else {
        None
    };

    if subcommand_name.is_some() {
        let subcommand_name = subcommand_name.unwrap();
        for command in application_commands.commands.iter() {
            if command.command_definition.get_name() == subcommand_name {
                return (command.executor)(config_file, args.unwrap()).into();
            }
        }
    }

    default_command().into()
}

struct ReturnExitCode {
    exit_code: Option<ExitCode>,
}

impl ReturnExitCode {
    fn new(exit_code: Option<ExitCode>) -> Self {
        Self {
            exit_code,
        }
    }
}

impl From<Option<ExitCode>> for ReturnExitCode {
    fn from(input: Option<ExitCode>) -> Self {
        ReturnExitCode::new(input)
    }
}

impl Termination for ReturnExitCode {
    fn report(self) -> ExitCode {
        match self.exit_code {
            Some(code) => code.report(),
            None => ExitCode::FAILURE,
        }
    }
}

struct CommandList {
    commands: Vec<Box<CommandHandler>>,
}

impl CommandList {
    fn subcommands(&self) -> Vec<ClapCommand> {
        self.commands.iter().fold(Vec::new(), |mut commands, command| {
            commands.push(command.command_definition.clone());
            commands
        })
    }
}

pub(crate) struct CommandHandler {
    pub(crate) command_definition: ClapCommand,
    pub(crate) executor: Box<dyn Fn(Option<&str>, &ArgMatches) -> Option<ExitCode>>,
}

impl CommandHandler {
    pub fn new(
        command_definition: ClapCommand,
        executor: Box<dyn Fn(Option<&str>, &ArgMatches) -> Option<ExitCode>>
    ) -> Self {
        Self { command_definition, executor }
    }
}

fn application_commands() -> CommandList {
    CommandList {
        commands: vec![
            Box::new(serve_webhook::get_command())
        ],
    }
}

fn get_app() -> ClapCommand {
    ClapCommand::new(APPLICATION_NAME)
        .version(APP_VERSION_METADATA.trim())
        .author("Alex \"Pierstoval\" Rock <alex@orbitale.io>")
        .about("A tool to manage your local CI/CD/etc setup")
        .arg(
            Arg::new("config-file")
                .short('c')
                .long("config-file")
                .global(true)
                .num_args(1)
                .help("Specify the config file to use for this instance."),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .global(true)
                .action(ArgAction::Count)
                .help("Set the verbosity level. -v for debug, -vv for trace, -vvv to trace executed modules"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .global(true)
                .num_args(0)
                .help("Do not display any output. Has precedence over -v|--verbose"),
        )
}

fn default_command() -> Option<ExitCode> {
    let process_args: Vec<String> = env::args().collect();
    let current_process_name = process_args[0].as_str().to_owned();

    // If no subcommand is specified,
    // re-run the program with "--help"
    let mut subprocess = Command::new(&current_process_name)
        .arg("--help")
        .spawn().ok()?;

    let child = subprocess.wait().ok()?;

    let status = child.code();

    match status {
        Some(code) => Some(ExitCode::from(code as u8)),
        None => Some(ExitCode::FAILURE),
    }
}

#[cfg(test)]
mod main_tests {
    use super::*;

    #[test]
    fn verify_cli() {
        get_app().debug_assert();
    }
}
