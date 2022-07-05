use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use std::sync::Mutex;
use crate::actions::queue::Queue;
use crate::config::config::Config;
use crate::actions::get_actions;

pub(crate) async fn webhook(
    request: HttpRequest,
    body_bytes: web::Bytes,
    config: web::Data<Config>,
    queue: web::Data<Mutex<Queue<String>>>,
) -> HttpResponse {
    let body_as_string = String::from_utf8(body_bytes.to_vec());
    if body_as_string.is_err() {
        return HttpResponse::BadRequest().body("Invalid body.");
    }
    let body_as_string = body_as_string.unwrap();

    let headers = request.headers();

    let config = config.get_ref();

    let actions_to_add = get_actions::from_request_parts(config, &body_as_string, headers);
    if actions_to_add.is_err() {
        return HttpResponse::BadRequest().body("Could not get actions to execute from this request.");
    }
    let actions_to_add = actions_to_add.unwrap();

    if actions_to_add.len() > 0 {
        let msg = format!("Matched! Actions to add: {:?}\n", &actions_to_add);

        let mut lock = queue.lock().unwrap();
        lock.add_actions(actions_to_add);

        return HttpResponse::Created().body(msg);
    }

    HttpResponse::BadRequest().body(format!("Request matched no webhook.\nBody:\n{}\n", body_as_string))
}

#[cfg(test)]
mod tests {
    use actix_web::dev::ServiceResponse;
    use actix_web::http;
    use actix_web::test::TestRequest;
    use actix_web::test::read_body;
    use actix_web::web;
    use super::*;
    use crate::test_utils;

    #[actix_web::test]
    async fn test_no_matcher() {
        let body_str = r#"{"repository":{"url":"https://github.com/my-org/my-repo"},"action":"published"}"#;
        let body_webhook = web::Bytes::from_static(body_str.as_bytes());

        let req = TestRequest::default()
            .uri("http://127.0.0.1:8000/webhook")
            .set_payload(body_str.as_bytes())
            .to_http_request();

        let config = Config::default();
        let config = web::Data::new(config);
        let queue = web::Data::new(Mutex::new(Queue::new()));

        let res = webhook(req.clone(), body_webhook, config, queue).await;

        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        let response_body = read_body(ServiceResponse::new(req, res)).await;

        assert_eq!(response_body, format!("Request matched no webhook.\nBody:\n{}\n", body_str));
    }

    #[actix_web::test]
    async fn test_webhook_with_json() {
        let body_str = r#"{"repository":{"url":"https://github.com/my-org/my-repo"},"action":"published"}"#.as_bytes();
        let body_webhook = web::Bytes::from_static(body_str.clone());

        let req = TestRequest::default()
            .uri("http://127.0.0.1:8000/webhook")
            .set_payload(body_str.clone())
            .to_http_request();

        let config = test_utils::get_sample_config().unwrap();
        let config = web::Data::new(config);
        let queue = web::Data::new(Mutex::new(Queue::new()));

        let res = webhook(req, body_webhook, config, queue).await;

        assert_eq!(res.status(), http::StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn test_command_with_headers() {
        let req = TestRequest::default()
            .uri("http://127.0.0.1:8000/webhook")
            .insert_header(("X-GitHub-Event", "release"))
            .insert_header(("X-GitHub-delivery", "12345"))
            .to_http_request();

        let config = test_utils::get_sample_config().unwrap();
        let config = web::Data::new(config);
        let queue = web::Data::new(Mutex::new(Queue::new()));

        let res = webhook(req, web::Bytes::new(), config, queue).await;

        assert_eq!(res.status(), http::StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn test_invalid_body() {

        // Remove some unicode elements to create an invalid utf8 string
        let mut bytes: Vec<u8> = "🚀".as_bytes().into();
        bytes.remove(0);
        bytes.pop();
        let request_body = web::Bytes::from(bytes);

        let req = TestRequest::default()
            .uri("http://127.0.0.1:8000/webhook")
            .set_payload(request_body.clone())
            .to_http_request();

        let config = test_utils::get_sample_config().unwrap();
        let config = web::Data::new(config);
        let queue = web::Data::new(Mutex::new(Queue::new()));

        let res = webhook(req.clone(), request_body, config, queue).await;

        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        let body_str = read_body(ServiceResponse::new(req, res)).await;

        assert_eq!(body_str, "Invalid body.");
    }
}
