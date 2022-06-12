use actix_web::web;
use actix_web::App;
use actix_web::http::header::HeaderMap;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use clap::Command as ClapCommand;
use clap::Arg;
use clap::ArgMatches;
use crate::config::config;
use crate::config::config::Matcher;
use crate::config::config::Config;
use crate::config::config::MatchersStrategy;

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

    let config = config.get_ref();

    let actions_to_add = get_actions_to_execute(config, &body_as_string, headers);

    if actions_to_add.len() > 0 {
        info!("Actions to add: {:?}", &actions_to_add);

        return HttpResponse::Created().body(format!("Matched! Actions to add: {:?}\n", &actions_to_add));
    }

    HttpResponse::BadRequest().body(format!("Request matched no webhook.\nBody:\n{}\n", body_as_string))
}

fn get_actions_to_execute(config: &Config, body_as_string: &String, headers: &HeaderMap) -> Vec<String> {
    let mut actions_to_add: Vec<String> = Vec::new();

    for webhook in &config.webhooks {
        let strategy = webhook.matchers_strategy.as_ref().unwrap_or_default();
        let number_of_matchers = webhook.matchers.len();
        let mut number_matching = 0;

        for matcher in &webhook.matchers {
            if match_headers(headers, matcher) || match_json(body_as_string, matcher) {
                number_matching += 1;
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

    actions_to_add
}

fn match_headers(headers: &HeaderMap, matcher: &Matcher) -> bool {
    if matcher.match_headers.is_none() {
        return false;
    }

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

    return headers_matching == number_of_headers;
}

fn match_json(body_as_string: &String, matcher: &Matcher) -> bool {
    if matcher.match_json_body.is_none() {
        return false;
    }

    let match_json_body = matcher.match_json_body.as_ref().unwrap();
    let match_json_body = serde_json::json!(match_json_body);

    let deserialized_result = serde_json::from_str::<serde_json::Value>(body_as_string.as_str());
    if deserialized_result.is_err() {
        debug!("Deserialization failed, skipping JSON matcher.");
        debug!("Deserialization error: {}", deserialized_result.unwrap_err());
        return false;
    }

    let deserialized_json = deserialized_result.unwrap();

    let json_comparator_config = assert_json_diff::Config::new(assert_json_diff::CompareMode::Strict);
    let matching_json_result = assert_json_diff::assert_json_matches_no_panic(&deserialized_json, &match_json_body, json_comparator_config);

    if matching_json_result.is_ok() {
        return true;
    }

    false
}
