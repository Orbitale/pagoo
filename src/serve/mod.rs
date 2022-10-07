use crate::actions::executor;
use crate::config;
use crate::config::Webhook;
use crate::db::get_database_connection;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use rusqlite::Connection;
use std::io::Error;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub(crate) const DEFAULT_PORT: &str = "8000";
pub(crate) const DEFAULT_HOST: &str = "127.0.0.1";
pub(crate) const API_PATH: &str = "/webhook";

#[actix_web::main]
pub(crate) async fn serve(
    config_file: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
) -> std::io::Result<()> {
    let host = host.unwrap_or(DEFAULT_HOST);
    let port = port.unwrap_or(DEFAULT_PORT);

    let port_as_int = port.parse::<u16>().expect("Invalid port value.");

    let config = config::get_config(config_file);

    if config.is_err() {
        let config_file_path = config::get_config_file(config_file);
        if config_file_path.is_err() {
            error!("{}", config_file_path.unwrap_err().to_string());
            std::process::exit(1);
        }
        error!(
            "Error loading config file \"{}\"",
            config_file_path.unwrap().to_str().unwrap()
        );
        let err = config.unwrap_err();
        return Err(Error::new(ErrorKind::Other, err));
    }

    let config = config.unwrap();
    let database_file = config.database_file.clone();

    let database_connection = get_database_connection(database_file).unwrap();

    let (sender, receiver) = mpsc::channel(8);

    info!("Starting queue workers...");

    start_workers(receiver, database_connection);

    let config = web::Data::new(config);
    let transmitter_data = web::Data::new(sender);

    info!("Starting HTTP server on {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .app_data(transmitter_data.clone())
            .service(web::resource(API_PATH).to(crate::http::webhook::webhook))
    })
    .bind((host, port_as_int))?
    .run()
    .await
}

fn start_workers(mut receiver: mpsc::Receiver<Vec<Webhook>>, conn: Connection) {
    tokio::spawn(async move {
        let conn = Arc::new(Mutex::new(conn));

        while let Some(webhooks) = receiver.recv().await {
            let res = executor::execute_webhook_actions(webhooks, Arc::clone(&conn));

            if res.is_err() {
                error!("Error executing actions");
            }
        }
    });
}
