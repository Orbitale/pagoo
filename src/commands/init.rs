use crate::CommandHandler;
use crate::APPLICATION_NAME;
use clap::ArgMatches;
use clap::Command as ClapCommand;
use std::fs::File;
use std::io::Write;
use std::process::ExitCode;

const FILE_SAMPLE: &str = include_str!("../../samples/init_sample.json");

pub(crate) fn get_command() -> CommandHandler {
    CommandHandler::new(
        ClapCommand::new("init").about("Creates a default configuration file"),
        Box::new(execute),
    )
}

fn execute(_config_file_value: Option<&str>, _args: &ArgMatches) -> Option<ExitCode> {
    let config_file_name = format!(".{}.json", APPLICATION_NAME.to_ascii_lowercase());
    let config_file_path = std::env::current_dir().unwrap().join(&config_file_name);

    if config_file_path.exists() {
        error!("File \"{}\" already exists.", config_file_name);

        return Some(ExitCode::FAILURE);
    }

    let file = File::create(&config_file_path);

    if let Err(err) = write!(&mut file.unwrap(), "{}", FILE_SAMPLE) {
        error!("Could not write to file. Error: {}", err.to_string());

        return Some(ExitCode::FAILURE);
    }

    info!(
        "Done! Created config file at path {}",
        config_file_path.to_str().unwrap()
    );

    Some(ExitCode::SUCCESS)
}
