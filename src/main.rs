extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use actix_web::HttpRequest;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use clap::Command as ClapCommand;
use clap::Arg;
use clap::ArgMatches;
use log::Level;
use pretty_env_logger::env_logger::fmt::Color;
use pretty_env_logger::env_logger::fmt::Style;
use pretty_env_logger::env_logger::fmt::StyledValue;
use std::env;
use std::io::Write;
use std::process::Command;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

fn main() {
    let build_metadata = include_str!("../build_metadata.txt").trim().replace("\n", "");
    let build_metadata_str = build_metadata.as_str();

    let version = if build_metadata == "" { "dev" } else { build_metadata_str };

    let application_commands = vec![
        spawn_http_command_config(),
    ];

    let app = ClapCommand::new("pagoo")
        .version(version)
        .author("Alex \"Pierstoval\" Rock <alex@orbitale.io>")
        .about("
")
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
        .subcommands(application_commands)
    ;

    let matches = app.get_matches();
    let verbose_value = matches.indices_of("verbose").unwrap_or_default();
    let is_quiet = matches.index_of("quiet").unwrap_or_default() > 0;

    set_verbosity_value(verbose_value.len(), is_quiet);

    let subcommand_name = matches.subcommand_name();

    match subcommand_name {
        Some("serve") => {
            spawn_http_server(matches.subcommand_matches(&subcommand_name.unwrap()).unwrap()).unwrap();
        },
        _ => {
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
    };
}

async fn webhook(request: HttpRequest, body: web::Bytes) -> impl Responder {
    dbg!(&request);
    let bytes = body.to_vec();
    let string = String::from_utf8(bytes).unwrap();

    HttpResponse::Ok().body(format!("Hello world!\n\nRequest body:\n========\n{}\n========", string))
}

#[actix_web::main]
async fn spawn_http_server(args: &'_ ArgMatches) -> std::io::Result<()> {
    let host = args.value_of("host").unwrap_or(DEFAULT_HOST.as_ref()).to_string();
    let port = args.value_of("port").unwrap_or(DEFAULT_PORT.as_ref()).to_string();

    let port_as_int = port.parse::<u16>().expect("Invalid port value.");

    info!("Starting HTTP server on {}:{}", host, port);

    HttpServer::new(|| {
        App::new()
            .service(web::resource("/webhook").to(webhook))
    })
        .bind((host.as_str(), port_as_int))?
        .run()
        .await
}

const DEFAULT_PORT: &str = "8000";
const DEFAULT_HOST: &str = "127.0.0.1";

pub fn spawn_http_command_config<'a>() -> ClapCommand<'a> {
    ClapCommand::new("serve")
        .about("Starts the HTTP server")
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
}


fn set_verbosity_value(value: usize, is_quiet: bool) {
    let level = std::env::var("PAGOO_LOG").unwrap_or(String::from("INFO"));
    let mut level = level.as_str();

    let mut builder = pretty_env_logger::formatted_timed_builder();

    if is_quiet {
        level = "OFF";
    } else {
        match value {
            1 => level = "DEBUG",           // -v
            v if v >= 2 => level = "TRACE", // -vv
            _ => {}
        }
    }

    builder
        .parse_filters(level)
        .format(move |f, record| {
            // This is the same format as the initial one in the pretty_env_logger crate,
            // but only the part with the module name is changed.

            let mut style = f.style();
            let level = colored_level(&mut style, record.level());

            let mut style = f.style();
            let target = if value > 2 {
                let target = format!(" {}", record.target());
                let max_width = max_target_width(&target);
                style.set_bold(true).value(Padded {
                    value: target,
                    width: max_width,
                })
            } else {
                style.value(Padded {
                    value: String::from(""),
                    width: 0,
                })
            };

            let time = f.timestamp_millis();

            writeln!(f, " {} {}{} > {}", time, level, target, record.args(),)
        })
        .try_init()
        .unwrap();
}

// This struct is a copy/paste of the one in pertty_env_logger.
// It's necessary for left-padding the message type.
struct Padded<T> {
    value: T,
    width: usize,
}

impl<T: std::fmt::Display> std::fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value(" INFO"),
        Level::Warn => style.set_color(Color::Yellow).value(" WARN"),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}
