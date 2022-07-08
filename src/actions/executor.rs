use std::process::Command;
use crate::config::config::Webhook;

pub(crate) fn execute_webhook_actions(webhooks: Vec<Webhook>) -> anyhow::Result<()> {
    for webhook in webhooks {
        let mut actions = webhook.actions_to_execute.clone();
        let all_actions = actions.clone();

        let command = actions.remove(0);
        let mut cmd = Command::new(command);
        cmd.args(actions.clone());
        let output = cmd.output()?;

        let stdout_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr_str = String::from_utf8_lossy(&output.stderr).trim().to_string();

        println!("==================");
        println!("Matching webhook: {}", webhook.name);
        println!("Command to execute: \"{}\"", all_actions.join(" "));
        println!("STDOUT: {}", stdout_str);
        println!("STDERR: {}", stderr_str);
    }

    Ok(())
}
