
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use clap::Command as ClapCommand;
use clap::Arg;
use clap::ArgMatches;
use crate::config::config;

const DEFAULT_PORT: &str = "8000";
const DEFAULT_HOST: &str = "127.0.0.1";

pub(crate) const COMMAND_NAME: &str = "serve:webhook";

pub(crate) fn command_config<'a>() -> ClapCommand<'a> {
    ClapCommand::new(COMMAND_NAME)
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
}

#[actix_web::main]
pub(crate) async fn serve(config_file: Option<&str>, args: &'_ ArgMatches) -> std::io::Result<()> {
    let host = args.value_of("host").unwrap_or(DEFAULT_HOST.as_ref()).to_string();
    let port = args.value_of("port").unwrap_or(DEFAULT_PORT.as_ref()).to_string();

    let port_as_int = port.parse::<u16>().expect("Invalid port value.");

    info!("Starting HTTP server on {}:{}", host, port);

    let config = web::Data::new(config::get_config(config_file));

    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .service(web::resource("/webhook").to(webhook))
    })
        .bind((host.as_str(), port_as_int))?
        .run()
        .await
}

async fn webhook(body_bytes: web::Bytes) -> impl Responder {
    let body_as_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    HttpResponse::Ok().body(format!("Hello world!\nRequest body:\n========\n{}\n========", body_as_string))
}
