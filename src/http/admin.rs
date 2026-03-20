use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use rusqlite::Connection;
use serde::Serialize;
use std::sync::Arc;
use std::sync::Mutex;

pub(crate) fn frontend_assets(path: String) -> Option<HttpResponse> {
    let path = &mut path.clone();

    let last_char = path.chars().last().unwrap().to_string();
    if last_char == "/" {
        path.push_str("index.html");
    }
    path.remove(0);

    let assets = crate::generate();
    let asset = assets.get(path.as_str());
    if asset.is_none() {
        return None;
    }
    let asset = asset.unwrap();

    Some(
        HttpResponse::Ok()
            .insert_header(("Content-Type", asset.mime_type))
            .body(asset.data),
    )
}

#[derive(Serialize)]
pub(crate) struct Task {
    pub id: String,
    pub execution_date: String,
    pub webhook_name: String,
    pub executed_command: String,
    pub command_exit_code: i32,
    pub command_stdout: String,
    pub command_stderr: String,
}

#[get("/api")]
pub(crate) async fn api_root() -> impl Responder {
    "Api endpoint.".to_string()
}

#[get("/api/tasks")]
pub(crate) async fn get_tasks(
    db: web::Data<Arc<Mutex<Connection>>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> actix_web::Result<HttpResponse> {
    let db = db.lock().map_err(|_| actix_web::error::ErrorInternalServerError("Database lock failed"))?;

    let cursor = query.get("cursor").cloned();
    let limit: i32 = query
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    let tasks: Vec<Task> = if let Some(cursor_val) = cursor {
        let mut stmt = db
            .prepare("SELECT id, execution_date, webhook_name, executed_command, command_exit_code, command_stdout, command_stderr FROM logs_webhooks WHERE execution_date > ? ORDER BY execution_date DESC LIMIT ?")
            .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to prepare query"))?;

        let rows = stmt.query_map(rusqlite::params![cursor_val, limit], |row| {
            Ok(Task {
                id: row.get(0)?,
                execution_date: row.get(1)?,
                webhook_name: row.get(2)?,
                executed_command: row.get(3)?,
                command_exit_code: row.get(4)?,
                command_stdout: row.get(5)?,
                command_stderr: row.get(6)?,
            })
        })
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to query tasks"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to collect tasks"))?
    } else {
        let mut stmt = db
            .prepare("SELECT id, execution_date, webhook_name, executed_command, command_exit_code, command_stdout, command_stderr FROM logs_webhooks ORDER BY execution_date DESC LIMIT ?")
            .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to prepare query"))?;

        let rows = stmt.query_map(rusqlite::params![limit], |row| {
            Ok(Task {
                id: row.get(0)?,
                execution_date: row.get(1)?,
                webhook_name: row.get(2)?,
                executed_command: row.get(3)?,
                command_exit_code: row.get(4)?,
                command_stdout: row.get(5)?,
                command_stderr: row.get(6)?,
            })
        })
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to query tasks"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to collect tasks"))?
    };

    Ok(HttpResponse::Ok().json(tasks))
}
