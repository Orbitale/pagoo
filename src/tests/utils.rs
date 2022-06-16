use hyper::Client;
use std::io::BufReader;
use std::io::Read;
use std::process::Command;
use std::process::Stdio;
use crate::APPLICATION_NAME;

pub(crate) fn get_test_http_client() -> Client<hyper::client::HttpConnector> {
    ensure_server_started();

    let builder = Client::builder();

    builder.build_http()
}

fn ensure_server_started() {
    wait_for_http_server_startup(&mut get_serve_webhook_command()).unwrap();
}

fn wait_for_http_server_startup(command: &mut Command) -> Result<(), anyhow::Error> {
    let mut child_command = command.spawn()?;

    let stderr = child_command.stderr.take().ok_or(anyhow::anyhow!("Could not get stderr from child process"))?;

    let mut reader = BufReader::new(stderr);

    let mut buffer = String::new();

    let now = std::time::Instant::now();

    const NUMBER_OF_BYTES_TO_READ: usize = 10;

    loop {
        let mut temp_buffer: [u8; NUMBER_OF_BYTES_TO_READ] = [0; NUMBER_OF_BYTES_TO_READ];
        reader.read_exact(&mut temp_buffer).unwrap_or_default();

        let buf_to_string = String::from_utf8(temp_buffer[..NUMBER_OF_BYTES_TO_READ].to_vec())?;
        let cleaned_buf_str = buf_to_string.trim().replace("\0", "");

        buffer.push_str(&cleaned_buf_str);

        if now.elapsed().as_millis() > 2500 {
            error!("Could not start server: {}", buffer);
            child_command.kill()?;
            return Err(anyhow::anyhow!("Server was too slow to start."));
        }

        if buffer.contains("Starting HTTP server on 127.0.0.1:8000") {
            info!("Server started: {}", buffer);
            break;
        }
    }

    Ok(())
}

fn get_serve_webhook_command() -> Command {
    let extension = if cfg!(target_os = "windows") { ".exe" } else { "" };
    let target = option_env!("OPT_LEVEL").unwrap_or("debug");
    let command_name = format!("target/{}/{}{}", target, APPLICATION_NAME, extension);
    let mut command = Command::new(command_name);

    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(&[
            "--config-file",
            "samples/json_sample.json",
            "serve:webhook",
        ])
    ;

    command
}
