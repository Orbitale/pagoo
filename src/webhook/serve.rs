use actix_web::web;
use actix_web::App;
use actix_web::http::header::HeaderMap;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
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

async fn webhook(request: HttpRequest, body_bytes: web::Bytes, config: web::Data<Config>) -> impl Responder {
    let body_as_string = String::from_utf8(body_bytes.to_vec()).unwrap();
    let headers = request.headers();

    let config = config.get_ref();

    let actions_to_add = get_actions_to_execute(config, &body_as_string, headers);

    if actions_to_add.len() > 0 {
        let msg = format!("Matched! Actions to add: {:?}\n", &actions_to_add);

        add_actions_to_queue(actions_to_add).unwrap();

        return HttpResponse::Created().body(msg);
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
            if
                match_headers(headers, matcher)
                || match_json(body_as_string, matcher)
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

    actions_to_add
}

#[cfg(test)]
mod tests {
    use hyper::Body;
    use hyper::Request;
    use crate::tests::utils;

    #[test]
    #[serial_test::serial]
    fn test_command_with_json() -> anyhow::Result<()> {
        let tokio_runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

        let client = utils::get_test_http_client()?;

        let req = Request::builder()
            .method("POST")
            .uri("http://127.0.0.1:8000/webhook")
            .body(Body::from(r#"{"repository":{"url":"https://github.com/my-org/my-repo"},"action":"published"}"#))?
        ;

        let (res, body) = tokio_runtime.block_on(client.request(req))?.into_parts();

        let status = res.status;
        assert_eq!(status, hyper::StatusCode::CREATED);

        let body_bytes = tokio_runtime.block_on(hyper::body::to_bytes(body))?.to_vec();
        let body_as_string = String::from_utf8(body_bytes)?;

        assert_eq!("Matched! Actions to add: [\"echo \\\"success!\\\"\"]\n", body_as_string);

        utils::teardown()?;

        Ok(())
    }

    #[test]
    #[serial_test::serial]
    fn test_command_with_headers() -> anyhow::Result<()> {
        let tokio_runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

        let client = utils::get_test_http_client()?;

        let req = Request::builder()
            .method("POST")
            .uri("http://127.0.0.1:8000/webhook")
            .header("X-GitHub-Event", "release")
            .header("X-GitHub-delivery", "12345")
            .body(Body::from(r#""#))?
        ;

        let (res, body) = tokio_runtime.block_on(client.request(req))?.into_parts();

        let status = res.status;
        assert_eq!(status, hyper::StatusCode::CREATED);

        let body_bytes = tokio_runtime.block_on(hyper::body::to_bytes(body))?.to_vec();
        let body_as_string = String::from_utf8(body_bytes)?;

        assert_eq!("Matched! Actions to add: [\"echo \\\"success!\\\"\"]\n", body_as_string);

        utils::teardown()?;

        Ok(())
    }
}
