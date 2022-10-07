use crate::serve::DEFAULT_HOST;
use crate::serve::DEFAULT_PORT;
use crate::CommandHandler;
use clap::arg;
use clap::ArgMatches;
use clap::Command as ClapCommand;
use std::process::ExitCode;

pub(crate) fn get_command() -> CommandHandler {
    CommandHandler::new(
        ClapCommand::new("serve:webhook")
            .about("Starts the Webhook HTTP server")
            .arg(arg!(--port <PORT> "The TCP port to listen to").default_value(DEFAULT_PORT))
            .arg(arg!(--host <HOST> "The network host to listen to").default_value(DEFAULT_HOST)),
        Box::new(execute),
    )
}

fn execute(config_file_value: Option<&str>, args: &ArgMatches) -> Option<ExitCode> {
    let host: Option<&str> = args.get_one::<String>("host").map(|s| s.as_str());
    let port: Option<&str> = args.get_one::<String>("port").map(|s| s.as_str());

    match crate::serve::serve(config_file_value, host, port) {
        Ok(_) => Some(ExitCode::SUCCESS),
        Err(e) => {
            error!("{}", e);
            Some(ExitCode::FAILURE)
        }
    }
}
