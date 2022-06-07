use std::sync::Mutex;

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) number_of_handled_requests: Mutex<u64>,
}

impl Config {
    pub fn new(counter: Mutex<u64>) -> Self {
        Self { number_of_handled_requests: counter }
    }
}

pub(crate) fn get_config() -> Config {
    Config::new(Mutex::new(0))
}
