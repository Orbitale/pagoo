use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Default)]
pub(crate) struct Config {
    pub(crate) webhooks: Vec<Webhook>,
}

//

#[derive(Debug, Default)]
pub(crate) struct Webhook {
    pub(crate) name: String,
    #[serde(rename(deserialize = "matchers-strategy"))]
    pub(crate) matchers_strategy: Option(MatchersStrategy),
    pub(crate) matchers: Vec<Matcher>,
    pub(crate) actions_to_execute: String,
}

//

pub(crate) enum MatchersStrategy {
    All,
    One,
}

//

pub(crate) struct Matcher {
    #[serde(rename(deserialize = "match-json-body"))]
    pub(crate) match_json_body: HashMap<String, String>,
    #[serde(rename(deserialize = "match-headers"))]
    pub(crate) match_headers: HashMap<String, String>,
}

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

impl Config {
    pub fn new() -> Self {
        Self {
            webhooks: vec![]
        }
    }
}

//

pub(crate) fn get_config() -> Config {
    Config::new()
}
