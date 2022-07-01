use crate::config::config::Config;
use crate::config::config;

pub(crate) fn get_sample_config() -> anyhow::Result<Config> {
    let file = "samples/json_sample.json";
    let project_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/{}", project_dir, file);

    config::get_config(Some(path.as_str()))
}
