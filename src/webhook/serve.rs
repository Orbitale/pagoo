
use actix_web::web;
use actix_web::App;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use clap::Command as ClapCommand;
use clap::Arg;
use clap::ArgMatches;
use crate::config::config;
use crate::config::config::{Config, MatchersStrategy};

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

async fn webhook(request: HttpRequest, body_bytes: web::Bytes, config: web::Data<Config>) -> impl Responder {
    let body_as_string = String::from_utf8(body_bytes.to_vec()).unwrap();
    let headers = request.headers();

    let mut actions_to_add: Vec<String> = Vec::new();

    for webhook in &config.webhooks {
        let strategy = webhook.matchers_strategy.as_ref().unwrap_or_default();
        let number_of_matchers = webhook.matchers.len();
        let mut number_matching = 0;

        for matcher in &webhook.matchers {
            if matcher.match_headers.is_some() {
                let matcher_headers = matcher.match_headers.as_ref().unwrap();
                let number_of_headers = matcher_headers.len();
                let mut headers_matching = 0;

                for (header_name, header_value) in matcher_headers {
                    if headers.contains_key(header_name) {
                        let header_value_as_string = headers.get(header_name).unwrap().to_str().unwrap();
                        if header_value_as_string == header_value {
                            headers_matching += 1
                        }
                    }
                }

                if headers_matching == number_of_headers {
                    number_matching += 1;
                }
            }

            if matcher.match_json_body.is_some() {
                let content_type = headers.get("Content-Type");
                if content_type.is_none() {
                    // No Content-Type = we won't even try to deserialize.
                    continue;
                }
                let content_type = content_type.unwrap();
                if content_type.to_str().unwrap() != "application/json" {
                    // Content-type not JSON = we can't deserialize it.
                    continue;
                }

                let match_json_body = matcher.match_json_body.as_ref().unwrap();
                let deserialized_result = serde_json::from_str::<serde_json::Value>(body_as_string.as_str());
                if deserialized_result.is_err() {
                    // Deserialization failed = we won't try to match.
                    continue;
                }
            }
        }

        let matched = match strategy {
            MatchersStrategy::All => number_of_matchers == number_matching,
            MatchersStrategy::One => number_matching > 0,
            _ => panic!("Unknown matchers strategy"),
        };

        if matched {
            let actions_to_execute = &webhook.actions_to_execute;

            &actions_to_add.push(actions_to_execute.clone());
        }
    }

    if actions_to_add.len() > 0 {
        info!("Actions to add: {:?}", &actions_to_add);

        return HttpResponse::Created().body(format!("Matched! Actions to add: {:?}\n", &actions_to_add));
    }

    HttpResponse::BadRequest().body("Request matched no webhook.\n")
}
