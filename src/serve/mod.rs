use crate::actions::executor;
use crate::config;
use crate::config::Webhook;
use crate::db::get_database_connection;
use crate::http;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use rusqlite::Connection;
use std::io::Error;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub(crate) const DEFAULT_WEBHOOK_PORT: &str = "8000";
pub(crate) const DEFAULT_ADMIN_PORT: &str = "8010";
pub(crate) const DEFAULT_HOST: &str = "127.0.0.1";
pub(crate) const WEBHOOK_API_PATH: &str = "/webhook";

#[actix_web::main]
pub(crate) async fn serve_webhook(
    config_file: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
) -> std::io::Result<()> {
    let host = host.unwrap_or(DEFAULT_HOST);
    let port = port.unwrap_or(DEFAULT_WEBHOOK_PORT);

    let port_as_int = port.parse::<u16>().expect("Invalid port value.");

    let config = config::get_config(config_file);

    if config.is_err() {
        return Err(Error::new(ErrorKind::Other, config.unwrap_err()));
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
            .service(web::resource(WEBHOOK_API_PATH).to(crate::http::webhook::webhook))
    })
    .bind((host, port_as_int))?
    .run()
    .await
}

#[actix_web::main]
pub(crate) async fn serve_admin(
    config_file: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
) -> std::io::Result<()> {
    let host = host.unwrap_or(DEFAULT_HOST);
    let port = port.unwrap_or(DEFAULT_ADMIN_PORT);

    let port_as_int = port.parse::<u16>().expect("Invalid port value.");

    let config = config::get_config(config_file).unwrap();
    let database_connection = get_database_connection(config.database_file.clone()).unwrap();

    let config = web::Data::new(config);
    let database_connection = web::Data::new(Arc::new(Mutex::new(database_connection)));

    info!("Starting HTTP server on {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .app_data(database_connection.clone())
            .service(http::admin::index)
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
