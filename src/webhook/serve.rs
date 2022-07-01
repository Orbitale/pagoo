use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use std::io::Error;
use std::io::ErrorKind;
use crate::config::config;

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
        let err = config.unwrap_err();
        return Err(Error::new(ErrorKind::Other, err));
    }

    let config = web::Data::new(config.unwrap());

    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .service(web::resource("/webhook").to(crate::api::webhook::webhook))
    })
        .bind((host, port_as_int))?
        .run()
        .await
}
