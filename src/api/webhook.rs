use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use tokio::sync::mpsc;
use crate::config::config::Config;
use crate::config::config::Webhook;
use crate::actions::matching_webhooks;

pub(crate) async fn webhook(
    request: HttpRequest,
    body_bytes: web::Bytes,
    config: web::Data<Config>,
    queue_sender: web::Data<mpsc::Sender<Vec<Webhook>>>,
) -> HttpResponse {
    let body_as_string = String::from_utf8(body_bytes.to_vec());
    if body_as_string.is_err() {
        return HttpResponse::BadRequest().body("Invalid body.");
    }
    let body_as_string = body_as_string.unwrap();

    let headers = request.headers();

    let config = config.get_ref();

    let matching_webhooks = matching_webhooks::from_request_parts(config, &body_as_string, headers);
    if matching_webhooks.is_err() {
        return HttpResponse::BadRequest().body("Could not get actions to execute from this request.");
    }
    let matching_webhooks = matching_webhooks.unwrap();

    if matching_webhooks.len() > 0 {
        let mut matching_webhooks_names = Vec::new();

        for webhook in &matching_webhooks {
            matching_webhooks_names.push(format!("{}", webhook.name));
        }

        let response_body = serde_json::json!({
            "matching_webhooks": matching_webhooks_names.as_slice(),
        });

        let sender_response = queue_sender.send(matching_webhooks).await;

        if sender_response.is_err() {
            error!("Could not send message to queue: {:?}", sender_response.unwrap_err());
            return HttpResponse::InternalServerError().body("Could not send message to queue.");
        }

        let _ = sender_response.unwrap();

        return HttpResponse::Created()
            .append_header(("Content-Type", "application/json"))
            .body(response_body.to_string())
        ;
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

        let (sender, _) = mpsc::channel(8);

        let config = web::Data::new(Config::default());
        let queue_sender = web::Data::new(sender);

        let res = webhook(req.clone(), body_webhook, config, queue_sender).await;

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

        let (sender, mut receiver) = mpsc::channel(8);

        let config = test_utils::get_sample_config().unwrap();
        let config = web::Data::new(config);
        let queue_sender = web::Data::new(sender);

        let res = webhook(req, body_webhook, config, queue_sender).await;

        assert_eq!(res.status(), http::StatusCode::CREATED);

        let res = receiver.recv().await;
        assert!(res.is_some());
        let res = res.unwrap();

        assert_eq!(2, res.len());
        assert_eq!("my_webhook_name", res[0].name);
        assert_eq!("my_webhook_name_2", res[1].name);
    }

    #[actix_web::test]
    async fn test_command_with_headers() {
        let req = TestRequest::default()
            .uri("http://127.0.0.1:8000/webhook")
            .insert_header(("X-GitHub-Event", "release"))
            .insert_header(("X-GitHub-delivery", "12345"))
            .to_http_request();

        let (sender, mut receiver) = mpsc::channel(8);

        let config = test_utils::get_sample_config().unwrap();
        let config = web::Data::new(config);
        let queue_sender = web::Data::new(sender);

        let res = webhook(req, web::Bytes::new(), config, queue_sender).await;

        assert_eq!(res.status(), http::StatusCode::CREATED);

        let res = receiver.recv().await;
        assert!(res.is_some());
        let res = res.unwrap();

        assert_eq!(1, res.len());
        assert_eq!("my_webhook_name", res[0].name);
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

        let (sender, _) = mpsc::channel(8);

        let config = test_utils::get_sample_config().unwrap();
        let config = web::Data::new(config);
        let queue_sender = web::Data::new(sender);

        let res = webhook(req.clone(), request_body, config, queue_sender).await;

        assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

        let body_str = read_body(ServiceResponse::new(req, res)).await;

        assert_eq!(body_str, "Invalid body.");
    }
}
