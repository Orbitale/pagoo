use std::sync::Mutex;

#[derive(Debug)]
pub(crate) struct Config {
}

impl Config {
    pub fn new() -> Self {
        Self { }
    }
}

pub(crate) fn get_config() -> Config {
    Config::new()
}
