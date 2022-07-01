use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use crate::APPLICATION_NAME;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct Config {
    pub(crate) webhooks: Vec<Webhook>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct Webhook {
    pub(crate) name: String,
    #[serde(rename(deserialize = "matchers-strategy"))]
    pub(crate) matchers_strategy: Option<MatchersStrategy>,
    pub(crate) matchers: Vec<Matcher>,
    #[serde(rename(deserialize = "actions-to-execute"))]
    pub(crate) actions_to_execute: String,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub(crate) enum MatchersStrategy {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "one")]
    One,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Matcher {
    #[serde(rename(deserialize = "match-json-body"))]
    pub(crate) match_json_body: Option<serde_json::Value>,
    #[serde(rename(deserialize = "match-headers"))]
    pub(crate) match_headers: Option<HashMap<String, String>>,
}

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

pub(crate) fn get_config(config_file: Option<&str>) -> Result<Config, anyhow::Error> {
    let default_file_name = format!(".{}.json", APPLICATION_NAME.to_ascii_lowercase());
    let config_file_name = config_file.unwrap_or(default_file_name.as_str());
    let config_file_path = std::path::Path::new(config_file_name);

    if !config_file_path.is_file() {
        return if config_file.is_some() {
            Err(anyhow::anyhow!("Config file not found: {}", config_file_name))
        } else {
            Err(anyhow::anyhow!("No config file specified, could not find a default one. You can create a \"{}\" file in this directory to configure the application.", default_file_name))
        }
    }

    let config_file_content = std::fs::read_to_string(config_file_path).unwrap();

    let config: Config = serde_json::from_str(&config_file_content).unwrap();

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_sample_file_path() -> PathBuf {
        std::env::current_dir().unwrap().join("samples/json_sample.json")
    }

    #[test]
    fn test_matching_strategy_default() {
        assert_eq!(MatchersStrategy::default(), MatchersStrategy::All);
    }

    #[test]
    fn test_matching_strategy_display_one() {
        assert_eq!(format!("{}", MatchersStrategy::One), "one");
    }

    #[test]
    fn test_matching_strategy_display_all() {
        assert_eq!(format!("{}", MatchersStrategy::All), "all");
    }

    #[test]
    fn test_inexistent_file_with_no_argument() {
        let config = get_config(None);
        assert!(config.is_err());
        assert_eq!(config.unwrap_err().to_string(), "No config file specified, could not find a default one. You can create a \".pagoo.json\" file in this directory to configure the application.".to_string());
    }

    #[test]
    fn test_inexistent_file_with_argument() {
        let config = get_config(Some("some_inexistent_file.json"));
        assert!(config.is_err());
        assert_eq!(config.unwrap_err().to_string(), "Config file not found: some_inexistent_file.json".to_string());
    }

    #[test]
    fn test_config() {
        let sample_file = get_sample_file_path();

        assert_eq!(true, sample_file.is_file());

        let config = get_config(Some(sample_file.to_str().unwrap())).unwrap();

        // Webhook
        assert_eq!(1, config.webhooks.len());
        let webhook = &config.webhooks[0];
        assert_eq!("my_webhook_name", webhook.name);
        assert_eq!("echo \"success!\"", webhook.actions_to_execute);
        assert_eq!(true, webhook.matchers_strategy.is_some());
        assert_eq!(MatchersStrategy::One, webhook.matchers_strategy.unwrap());

        assert_eq!(2, webhook.matchers.len());

        // First matcher
        let matcher = &webhook.matchers[0];
        assert_eq!(true, matcher.match_headers.is_none());
        let json_body = matcher.match_json_body.as_ref();
        assert_eq!(true, json_body.is_some());
        assert_eq!(
            json_body.unwrap(),
            &serde_json::json!({
                "repository": {
                    "url": "https://github.com/my-org/my-repo"
                },
                "action": "published"
            })
        );

        // Second matcher
        let matcher = &webhook.matchers[1];
        assert_eq!(true, matcher.match_json_body.is_none());
        let headers = matcher.match_headers.as_ref();
        assert_eq!(true, headers.is_some());
        let headers_map = headers.unwrap();
        assert_eq!(true, headers_map.contains_key("x-github-event"));
        assert_eq!("release", headers_map.get("x-github-event").unwrap());
        assert_eq!(true, headers_map.contains_key("x-github-delivery"));
        assert_eq!("12345", headers_map.get("x-github-delivery").unwrap());
    }

}
