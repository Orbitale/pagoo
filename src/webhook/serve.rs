use actix_web::web;
use actix_web::App;
use actix_web::http::header::HeaderMap;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use crate::actions::queue::add_actions_to_queue;
use crate::config::config;
use crate::config::config::Config;
use crate::config::config::MatchersStrategy;
use crate::matchers::headers::match_headers;
use crate::matchers::json::match_json;

pub(crate) const DEFAULT_PORT: &str = "8000";
pub(crate) const DEFAULT_HOST: &str = "127.0.0.1";

#[actix_web::main]
pub(crate) async fn serve(config_file: Option<&str>, host: Option<&str>, port: Option<&str>) -> std::io::Result<()> {
    let host = host.unwrap_or(DEFAULT_HOST);
    let port = port.unwrap_or(DEFAULT_PORT);

    let port_as_int = port.parse::<u16>().expect("Invalid port value.");

    info!("Starting HTTP server on {}:{}", host, port);

    let config = web::Data::new(config::get_config(config_file));

    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .service(web::resource("/webhook").to(webhook))
    })
        .bind((host, port_as_int))?
        .run()
        .await
}

async fn webhook(request: HttpRequest, body_bytes: web::Bytes, config: web::Data<Config>) -> HttpResponse {
    let body_as_string = String::from_utf8(body_bytes.to_vec());
    if body_as_string.is_err() {
        return HttpResponse::BadRequest().body("Invalid body.");
    }
    let body_as_string = body_as_string.unwrap();

    let headers = request.headers();

    let config = config.get_ref();

    let actions_to_add = get_actions_to_execute(config, &body_as_string, headers);
    if actions_to_add.is_err() {
        return HttpResponse::BadRequest().body("Could not get actions to execute from this request.");
    }
    let actions_to_add = actions_to_add.unwrap();

    if actions_to_add.len() > 0 {
        let msg = format!("Matched! Actions to add: {:?}\n", &actions_to_add);


        let add_actions_result = add_actions_to_queue(actions_to_add);
        if add_actions_result.is_err() {
            return HttpResponse::InternalServerError().body("Could not add actions to queue");
        }


        return HttpResponse::Created().body(msg);
    }

    HttpResponse::BadRequest().body(format!("Request matched no webhook.\nBody:\n{}\n", body_as_string))
}

fn get_actions_to_execute(config: &Config, body_as_string: &String, headers: &HeaderMap) -> Result<Vec<String>, anyhow::Error> {
    let mut actions_to_add: Vec<String> = Vec::new();

    for webhook in &config.webhooks {
        let strategy = webhook.matchers_strategy.as_ref().unwrap_or_default();
        let number_of_matchers = webhook.matchers.len();
        let mut number_matching = 0;

        for matcher in &webhook.matchers {
            if
                match_headers(headers, matcher)?
                || match_json(body_as_string, matcher)?
            {
                number_matching += 1;
            }
        }

        let matched = match strategy {
            MatchersStrategy::All => number_of_matchers == number_matching,
            MatchersStrategy::One => number_matching > 0,
        };

        if matched {
            let actions_to_execute = &webhook.actions_to_execute;

            info!("Matched webhook: {:?}", webhook.name);

            actions_to_add.push(actions_to_execute.clone());
        }
    }

    Ok(actions_to_add)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http;
    use actix_web::test::TestRequest;
    use actix_web::web;
    use crate::tests::utils;

    #[actix_web::test]
    async fn test_webhook_with_json() {
        let body_str = r#"{"repository":{"url":"https://github.com/my-org/my-repo"},"action":"published"}"#.as_bytes();
        let body_webhook = web::Bytes::from_static(body_str.clone());

        let req = TestRequest::default()
            .uri("http://127.0.0.1:8000/webhook")
            .set_payload(body_str.clone())
            .to_http_request();

        let config = utils::get_sample_config();
        let config = web::Data::new(config);

        let res = webhook(req, body_webhook, config).await;

        assert_eq!(res.status(), http::StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn test_command_with_headers() {
        let req = TestRequest::default()
            .uri("http://127.0.0.1:8000/webhook")
            .insert_header(("X-GitHub-Event", "release"))
            .insert_header(("X-GitHub-delivery", "12345"))
            .to_http_request();

        let config = utils::get_sample_config();
        let config = web::Data::new(config);

        let res = webhook(req, web::Bytes::new(), config).await;

        assert_eq!(res.status(), http::StatusCode::CREATED);
    }
}
