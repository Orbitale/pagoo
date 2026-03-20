use crate::serve::DEFAULT_HOST;
use crate::serve::DEFAULT_WEBHOOK_PORT;
use crate::serve::DEFAULT_ADMIN_PORT;
use crate::CommandHandler;
use clap::arg;
use clap::ArgMatches;
use clap::Command as ClapCommand;
use std::process::ExitCode;

pub(crate) fn get_command() -> CommandHandler {
    CommandHandler::new(
        ClapCommand::new("serve")
            .alias("serve:webhook")
            .about("(alias: serve:webhook) Starts the Webhook HTTP server")
            .arg(arg!(--port <PORT> "The TCP port to listen to").default_value(DEFAULT_WEBHOOK_PORT))
            .arg(arg!(--host <HOST> "The network host to listen to").default_value(DEFAULT_HOST))
            .arg(arg!(--adminport <ADMINPORT> "The path prefix for the Admin panel").default_value(DEFAULT_ADMIN_PORT))
            .arg(arg!(--admin "Whether to enable the Admin panel"))
        ,
        Box::new(execute),
    )
}

fn execute(config_file_value: Option<&str>, args: &ArgMatches) -> Option<ExitCode> {
    let host: Option<&str> = args.get_one::<String>("host").map(|s| s.as_str());
    let port: Option<&str> = args.get_one::<String>("port").map(|s| s.as_str());
    let admin: bool = args.get_flag("admin");
    let admin_port: Option<&str> = args.get_one::<String>("adminport").map(|s| s.as_str());

    match crate::serve::spawn_servers(config_file_value, host, port, admin, admin_port) {
        Ok(_) => Some(ExitCode::SUCCESS),
        Err(e) => {
            error!("{}", e);
            Some(ExitCode::FAILURE)
        }
    }
}
