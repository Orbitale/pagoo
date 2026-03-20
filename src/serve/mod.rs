use crate::actions::executor;
use crate::config;
use crate::config::Webhook;
use crate::db::get_database_connection;
use crate::http;
use actix_web::dev::{Server, Service};
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use anyhow::Error;
use futures_util::future::Either;
use futures_util::FutureExt;
use rusqlite::Connection;
use std::future;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;

pub(crate) const DEFAULT_WEBHOOK_PORT: &str = "8000";
pub(crate) const DEFAULT_ADMIN_PORT: &str = "8010";
pub(crate) const DEFAULT_HOST: &str = "127.0.0.1";
pub(crate) const WEBHOOK_API_PATH: &str = "/webhook";

#[actix_web::main]
pub(crate) async fn spawn_servers(
    config_file: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
    admin: bool,
    admin_port: Option<&str>,
) -> Result<(), anyhow::Error> {
    let webhook_server = serve_webhook(config_file.clone(), host.clone(), port)?;

    if admin {
        let admin_server = serve_admin(config_file.clone(), host.clone(), admin_port)?;

        futures::future::try_join(webhook_server, admin_server).await?;
    } else {
        webhook_server.await?;
    }

    Ok(())
}

pub(crate) fn serve_webhook(
    config_file: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
) -> Result<Server, Error> {
    let host = host.unwrap_or(DEFAULT_HOST);
    let port = port.unwrap_or(DEFAULT_WEBHOOK_PORT);

    let port_as_int = port.parse::<u16>().expect("Invalid port value.");

    let config = config::get_config(config_file)?;

    let database_file = config.database_file.clone();

    let database_connection = get_database_connection(database_file)?;

    let (sender, receiver) = mpsc::channel(8);

    info!("Starting queue workers...");

    start_workers(receiver, database_connection);

    let config = web::Data::new(config);
    let transmitter_data = web::Data::new(sender);

    info!("Starting HTTP server on {}:{}", host, port);

    Ok(
        HttpServer::new(move || {
            App::new()
                .app_data(config.clone())
                .app_data(transmitter_data.clone())
                .wrap(Logger::default())
                .service(web::resource(WEBHOOK_API_PATH).to(crate::http::webhook::webhook))
        })
            .bind((host, port_as_int))?
            .run()
    )
}

pub(crate) fn serve_admin(
    config_file: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
) -> Result<Server, anyhow::Error> {
    let host = host.unwrap_or(DEFAULT_HOST);
    let port = port.unwrap_or(DEFAULT_ADMIN_PORT);

    let port_as_int = port.parse::<u16>().expect("Invalid port value.");

    let config = config::get_config(config_file)?;
    let database_connection = get_database_connection(config.database_file.clone())?;

    let config = web::Data::new(config);
    let database_connection = web::Data::new(Arc::new(Mutex::new(database_connection)));

    info!("Starting HTTP server on {}:{}", host, port);

    Ok(
        HttpServer::new(move || {
            App::new()
                .app_data(config.clone())
                .app_data(database_connection.clone())
                // .wrap(http::admin::AdminAsset::new())
                .wrap_fn(|sreq, srv| {
                    let path = sreq.path().to_string();
                    let asset_exists: Option<HttpResponse> = http::admin::frontend_assets(path.clone());

                    match asset_exists {
                        None => Either::Left(srv.call(sreq).map(|res| res)),
                        Some(asset) => return Either::Right(future::ready(Ok(sreq.into_response(asset)))),
                    }
                })
                .service(http::admin::api_root)
                .service(http::admin::get_tasks)
        })
            .bind((host, port_as_int))?
            .run()
    )
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
