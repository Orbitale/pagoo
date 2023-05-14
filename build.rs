use static_files::resource_dir;
use std::env;
use std::fs::read_to_string;
use std::fs::remove_file;
use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

fn main() {
    let yarn = which::which("yarn").expect("Could not find Yarn executable.");

    admin_frontend_deps(yarn.clone());
    admin_frontend_build(yarn.clone());

    resource_dir(format!("{}/admin_app/build/", env::var("CARGO_MANIFEST_DIR").unwrap())).build().unwrap();
}

fn admin_frontend_deps(yarn: PathBuf){
    let mut command = Command::new(yarn);
    let (stdout, stderr) = get_std_outputs();
    command
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .arg("--cwd")
        .arg(format!("{}/admin_app/", env::var("CARGO_MANIFEST_DIR").unwrap()))
        .arg("install")
    ;

    handle_error(command.output());
}

fn admin_frontend_build(yarn: PathBuf){
    let mut command = Command::new(yarn);
    let (stdout, stderr) = get_std_outputs();
    command
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .arg("--cwd")
        .arg(format!("{}/admin_app/", env::var("CARGO_MANIFEST_DIR").unwrap()))
        .arg("build")
    ;

    handle_error(command.output());
}

fn handle_error(output: std::io::Result<Output>) {
    match output {
        Ok(output) => {
            let code = output.status.code().unwrap();
            if code != 0 {
                let error = read_to_string("build.err")
                    .expect("Could not retrieve error log after failing to build admin frontend.");
                panic!(" An error occured when building admin frontend.\n Here is the error log:\n{}", error);
            }
        },
        Err(e) => {
            panic!(" Could not build admin frontend: {}", e);
        },
    };
}

fn get_std_outputs() -> (Stdio, Stdio) {
    let stdout_file_path = Path::new("build.log");
    if stdout_file_path.is_file() {
        remove_file(stdout_file_path).unwrap();
    }

    let stderr_file_path = Path::new("build.err");
    if stderr_file_path.is_file() {
        remove_file(stderr_file_path).unwrap();
    }

    (
        Stdio::from(File::create(stdout_file_path).unwrap()),
        Stdio::from(File::create(stderr_file_path).unwrap()),
    )
}
