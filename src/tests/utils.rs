use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::io::BufReader;
use std::io::Read;
use hyper::Client;
use crate::APPLICATION_NAME;

use std::sync::Once;

struct PID {
    pid: u32,
}

impl PID {
    const fn new(pid: u32) -> Self {
        PID { pid }
    }

    fn set(&mut self, pid: u32) {
        self.pid = pid;
    }
}

impl Drop for PID {
    fn drop(&mut self) {
        if self.pid != 0 {
            kill_process(self.pid);
        }
    }
}

static INIT: Once = Once::new();

static mut SERVER_PID: PID = PID::new(0);

fn ensure_server_started() {
    INIT.call_once(|| {
        let child = wait_for_http_server_startup(&mut get_serve_webhook_command()).unwrap();
        unsafe {
            SERVER_PID.set(child.id());
        }
    });
}

pub(crate) fn get_test_http_client() -> Client<hyper::client::HttpConnector> {
    ensure_server_started();

    let builder = Client::builder();

    builder.build_http()
}

fn get_serve_webhook_command() -> Command {
    let extension = if cfg!(target_os = "windows") { ".exe" } else { "" };
    let command_name = format!("target/release/{}{}", APPLICATION_NAME, extension);
    let mut command = Command::new(command_name);

    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .args(&["--config-file", "samples/json_sample.json", "serve:webhook"])
    ;

    command
}

fn wait_for_http_server_startup(command: &mut Command) -> Result<Child, anyhow::Error> {
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

    Ok(child_command)
}


#[cfg(target_family = "windows")]
fn kill_process(pid: u32) {
    let mut child = Command::new("taskkill")
        .arg("/T") // Stops process tree
        .arg("/F") // Force stop
        .arg("/PID")
        .arg(pid.to_string())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("Could not stop server.");

    child
        .wait()
        .expect("An error occured when trying to stop the server");
}

#[cfg(not(target_family = "windows"))]
fn kill_process(pid: u32) {
    let mut child = Command::new("kill")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .arg("-TERM")
        .arg("--")
        .arg(pid.to_string())
        .spawn()
        .expect("Could not stop server.");

    child
        .wait()
        .expect("An error occured when trying to stop the server");
}
