use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use crate::APPLICATION_NAME;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct Config {
    pub(crate) webhooks: Vec<Webhook>,
}

//

#[derive(Debug, Default, Deserialize)]
pub(crate) struct Webhook {
    pub(crate) name: String,
    #[serde(rename(deserialize = "matchers-strategy"))]
    pub(crate) matchers_strategy: Option<MatchersStrategy>,
    pub(crate) matchers: Vec<Matcher>,
    #[serde(rename(deserialize = "actions-to-execute"))]
    pub(crate) actions_to_execute: String,
}

//

#[derive(Debug, Deserialize)]
pub(crate) enum MatchersStrategy {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "one")]
    One,
}

//

#[derive(Debug, Deserialize)]
pub(crate) struct Matcher {
    #[serde(rename(deserialize = "match-json-body"))]
    pub(crate) match_json_body: Option<HashMap<String, serde_json::Value>>,
    #[serde(rename(deserialize = "match-headers"))]
    pub(crate) match_headers: Option<HashMap<String, String>>,
}

//
//
//
//
//
//

impl Default for MatchersStrategy {
    fn default() -> Self {
        MatchersStrategy::All
    }
}

impl Display for MatchersStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchersStrategy::All => write!(f, "all"),
            MatchersStrategy::One => write!(f, "one"),
        }
    }
}

//

pub(crate) fn get_config(config_file: Option<&str>) -> Config {
    let default_file_name = format!(".{}.json", APPLICATION_NAME.to_ascii_lowercase());
    let config_file_name = config_file.unwrap_or(default_file_name.as_str());
    let config_file_path = std::path::Path::new(config_file_name);

    if !config_file_path.is_file() {
        if config_file.is_some() {
            error!("Config file not found: {}", config_file_name);
        } else {
            error!("No config file specified, could not find a default one.");
            error!("You can create a {} file in this directory to configure the application.", default_file_name);
        }
        std::process::exit(1);
    }

    let config_file_content = std::fs::read_to_string(config_file_path).unwrap();

    let config: Config = serde_json::from_str(&config_file_content).unwrap();

    dbg!(&config);

    config
}
