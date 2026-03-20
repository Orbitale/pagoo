use crate::config::Webhook;
use rusqlite::named_params;
use rusqlite::Connection;
use std::process::Command;
use std::process::Output;
use std::sync::Arc;
use std::sync::Mutex;
use actix_web::ResponseError;

pub(crate) fn execute_webhook_actions(
    webhooks: Vec<Webhook>,
    conn: Arc<Mutex<Connection>>,
) -> anyhow::Result<()> {
    let conn = conn
        .lock()
        .expect("Could not retrieve database connection.");

    for webhook in webhooks {
        let mut actions = webhook.actions_to_execute.clone();
        let all_actions = actions.clone();

        let command = actions.remove(0);
        let mut cmd = Command::new(command);
        cmd.args(actions.clone());
        let output: std::io::Result<Output> = cmd.output();

        let status: i32;
        let mut stdout_str = String::from("");
        let stderr_str: String;

        if output.is_ok() {
            let output_result = output.unwrap();
            status = output_result.status.code().unwrap_or(161);
            stdout_str = String::from_utf8_lossy(&output_result.stdout).trim().to_string();
            stderr_str = String::from_utf8_lossy(&output_result.stderr).trim().to_string();
        } else {
            let err = output.unwrap_err();
            status = err.status_code().as_u16().into();
            stderr_str = format!("kind: {} ; message: {}", err.kind(), err.to_string());
        }

        conn.execute(
            "
            INSERT INTO logs_webhooks (
                execution_date,
                webhook_name,
                executed_command,
                command_exit_code,
                command_stdout,
                command_stderr
            ) VALUES (
                datetime(),
                :webhook_name,
                :executed_command,
                :command_exit_code,
                :command_stdout,
                :command_stderr
            )
            ",
            named_params! {
                ":webhook_name": webhook.name,
                ":executed_command": all_actions.join(" "),
                ":command_exit_code": status,
                ":command_stdout": stdout_str,
                ":command_stderr": stderr_str,
            },
        )?;
    }

    Ok(())
}
