use crate::config::Webhook;
use rusqlite::named_params;
use rusqlite::Connection;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;

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
        let output = cmd.output()?;

        let status = output.status.code().unwrap();
        let stdout_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr_str = String::from_utf8_lossy(&output.stderr).trim().to_string();

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
