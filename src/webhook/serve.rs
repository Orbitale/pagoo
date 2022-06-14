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
use crate::config::config::Config;
use crate::config::config::MatchersStrategy;
use crate::matchers::headers::match_headers;
use crate::matchers::json::match_json;

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

            info!("Matched webhook: {:?}", webhook.name);

            actions_to_add.push(actions_to_execute.clone());
        }
    }

    actions_to_add
}

#[cfg(test)]
mod tests {
    use hyper::Body;
    use hyper::Client;
    use hyper::Request;
    use std::io::Read;
    use std::process::Command;
    use std::process::Child;
    use crate::APPLICATION_NAME;

    fn get_command() -> Command {
        let extension = if cfg!(target_os = "windows") { ".exe" } else { "" };
        let command_name = format!("target/release/{}{}", APPLICATION_NAME, extension);
        let mut command = Command::new(command_name);

        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .args(&["--config-file", "samples/json_sample.json", "serve:webhook"])
        ;

        command
    }

    fn get_command_outputs(child_command: Child) -> (String, String) {
        let stdout = child_command.stdout.unwrap();
        let stderr = child_command.stderr.unwrap();

        let stdout_slice = stdout.bytes().map(|b| b.unwrap()).collect();
        let stderr_slice = stderr.bytes().map(|b| b.unwrap()).collect();

        let stdout_string = String::from_utf8(stdout_slice).unwrap();
        let sterr_string = String::from_utf8(stderr_slice).unwrap();

        (stdout_string, sterr_string)
    }

    fn get_http_client() -> Client<hyper::client::HttpConnector> {
        let builder = Client::builder();

        builder.build_http()
    }

    #[test]
    fn test_command_with_json() -> anyhow::Result<()> {
        let tokio_runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

        let mut command = get_command();

        let mut child = command.spawn()?;
        std::thread::sleep(std::time::Duration::from_millis(250));

        let client = get_http_client();

        let req = Request::builder()
            .method("POST")
            .uri("http://127.0.0.1:8000/webhook")
            .body(Body::from(r#"{"repository":{"url":"https://github.com/my-org/my-repo"},"action":"published"}"#))?
        ;

        let (res, body) = tokio_runtime.block_on(client.request(req))?.into_parts();

        child.kill()?;
        let (stdout, stderr) = get_command_outputs(child);
        assert_eq!(stdout, "".to_string());
        assert!(stderr.contains("Starting HTTP server on 127.0.0.1:8000\n"));

        let status = res.status;
        assert_eq!(status, hyper::StatusCode::CREATED);

        let body_bytes = tokio_runtime.block_on(hyper::body::to_bytes(body))?.to_vec();
        let body_as_string = String::from_utf8(body_bytes)?;

        assert_eq!("Matched! Actions to add: [\"curl -i ...\\nmy_binary --verbose ...\"]\n", body_as_string);

        Ok(())
    }

    #[test]
    fn test_command_with_headers() -> anyhow::Result<()> {
        let tokio_runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

        let mut command = get_command();

        let mut child = command.spawn()?;
        std::thread::sleep(std::time::Duration::from_millis(250));

        let client = get_http_client();

        let req = Request::builder()
            .method("POST")
            .uri("http://127.0.0.1:8000/webhook")
            .header("X-GitHub-Event", "release")
            .header("X-GitHub-delivery", "12345")
            .body(Body::from(r#""#))?
        ;

        let (res, body) = tokio_runtime.block_on(client.request(req))?.into_parts();

        child.kill()?;
        let (stdout, stderr) = get_command_outputs(child);
        assert_eq!(stdout, "".to_string());
        assert!(stderr.contains("Starting HTTP server on 127.0.0.1:8000\n"));

        let status = res.status;
        assert_eq!(status, hyper::StatusCode::CREATED);

        let body_bytes = tokio_runtime.block_on(hyper::body::to_bytes(body))?.to_vec();
        let body_as_string = String::from_utf8(body_bytes)?;

        assert_eq!("Matched! Actions to add: [\"curl -i ...\\nmy_binary --verbose ...\"]\n", body_as_string);

        Ok(())
    }
}
