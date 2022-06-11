
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
                let match_json_body = matcher.match_json_body.as_ref().unwrap();
                let match_json_body = serde_json::json!(match_json_body);
                let deserialized_result = serde_json::from_str::<serde_json::Value>(body_as_string.as_str());
                if deserialized_result.is_err() {
                    // Deserialization failed = we won't try to match.
                    debug!("Deserialization failed, skipping JSON matcher.");
                    debug!("Deserialization error: {}", deserialized_result.unwrap_err());
                    continue;
                }
                let deserialized_json = deserialized_result.unwrap();

                let json_comparator_config = assert_json_diff::Config::new(assert_json_diff::CompareMode::Strict);
                let matching_json_result = assert_json_diff::assert_json_matches_no_panic(&deserialized_json, &match_json_body, json_comparator_config);
                if matching_json_result.is_ok() {
                    number_matching += 1;
                } else {
                    debug!("JSON is not matching, skipping JSON matcher.");
                }
            }
        }

        let matched = match strategy {
            MatchersStrategy::All => number_of_matchers == number_matching,
            MatchersStrategy::One => number_matching > 0,
        };

        if matched {
            let actions_to_execute = &webhook.actions_to_execute;

            actions_to_add.push(actions_to_execute.clone());
        }
    }

    if actions_to_add.len() > 0 {
        info!("Actions to add: {:?}", &actions_to_add);

        return HttpResponse::Created().body(format!("Matched! Actions to add: {:?}\n", &actions_to_add));
    }

    dbg!(&config);

    HttpResponse::BadRequest().body(format!("Request matched no webhook.\nBody:\n{}\n", body_as_string))
}
