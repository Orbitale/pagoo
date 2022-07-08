use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use std::io::Error;
use std::io::ErrorKind;
use tokio::sync::mpsc;
use crate::actions::executor;
use crate::config::config;
use crate::config::config::Webhook;

pub(crate) const DEFAULT_PORT: &str = "8000";
pub(crate) const DEFAULT_HOST: &str = "127.0.0.1";

#[actix_web::main]
pub(crate) async fn serve(config_file: Option<&str>, host: Option<&str>, port: Option<&str>) -> std::io::Result<()> {
    let host = host.unwrap_or(DEFAULT_HOST);
    let port = port.unwrap_or(DEFAULT_PORT);

    let port_as_int = port.parse::<u16>().expect("Invalid port value.");

    info!("Starting HTTP server on {}:{}", host, port);

    let config = config::get_config(config_file);

    if config.is_err() {
        let config_file_path = config::get_config_file(config_file).unwrap();
        error!("Error loading config file \"{}\"", config_file_path.to_str().unwrap());
        let err = config.unwrap_err();
        return Err(Error::new(ErrorKind::Other, err));
    }

    let (sender, receiver) = mpsc::channel(8);

    start_workers(receiver);

    let config = web::Data::new(config.unwrap());
    let transmitter_data = web::Data::new(sender);

    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .app_data(transmitter_data.clone())
            .service(web::resource("/webhook").to(crate::api::webhook::webhook))
    })
        .bind((host, port_as_int))?
        .run()
        .await
}

fn start_workers(mut receiver: mpsc::Receiver<Vec<Webhook>>) {
    tokio::spawn(async move {
        while let Some(webhooks) = receiver.recv().await {
            let res = executor::execute_webhook_actions(webhooks);

            if res.is_err() {
                error!("Error executing actions");
            }
        }
    });
}
