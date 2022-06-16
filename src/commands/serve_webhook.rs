use std::process::ExitCode;
use clap::ArgMatches;
use clap::Command as ClapCommand;
use clap::Arg;
use crate::CommandHandler;

use crate::webhook::serve::DEFAULT_PORT;
use crate::webhook::serve::DEFAULT_HOST;

pub(crate) fn get_command() -> CommandHandler {
    CommandHandler::new(
        ClapCommand::new("serve:webhook")
            .about("Starts the Webhook HTTP server")
            .arg(
                Arg::new("port")
                    .long("port")
                    .help("The TCP port to listen to")
                    .default_value(DEFAULT_PORT.as_ref())
                    .takes_value(true),
            )
            .arg(
                Arg::new("host")
                    .long("host")
                    .help("The network host to listen to")
                    .default_value(DEFAULT_HOST.as_ref())
                    .takes_value(true),
            )
        ,
        Box::new(execute)
    )
}

fn execute(config_file_value: Option<&str>, args: &ArgMatches) -> Option<ExitCode> {
    let host: Option<&str> = args.value_of("host");
    let port: Option<&str> = args.value_of("port");

    match crate::webhook::serve::serve(config_file_value, host, port) {
        Ok(_) => Some(ExitCode::SUCCESS),
        Err(e) => {
            error!("{}", e);
            Some(ExitCode::FAILURE)
        }
    }
}
