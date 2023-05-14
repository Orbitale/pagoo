use crate::config::Config;
use actix_web::web::Bytes;
use actix_web::web::Data;
use actix_web::{get, HttpRequest};
use actix_web::Responder;
use rusqlite::Connection;
use std::sync::Arc;
use std::sync::Mutex;

#[get("/")]
pub(crate) async fn index(
    _request: HttpRequest,
    _body_bytes: Bytes,
    _config: Data<Config>,
    _database: Data<Arc<Mutex<Connection>>>,
) -> impl Responder {
    let assets = crate::generate();

    let cnt = assets.get("index.html").unwrap();
    let cnt_str = String::from_utf8(cnt.data.to_vec()).unwrap();

    dbg!("{}", &cnt_str);

    "Hello!"
}
