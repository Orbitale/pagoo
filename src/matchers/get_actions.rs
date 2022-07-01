use actix_web::http::header::HeaderMap;
use crate::config::config::Config;
use crate::config::config::MatchersStrategy;
use crate::matchers::headers::match_headers;
use crate::matchers::json::match_json;

pub(crate) fn from_request_parts(config: &Config, body_as_string: &String, headers: &HeaderMap) -> Result<Vec<String>, anyhow::Error> {
    let mut actions_to_add: Vec<String> = Vec::new();

    for webhook in &config.webhooks {
        let strategy = webhook.matchers_strategy.as_ref().unwrap_or_default();
        let number_of_matchers = webhook.matchers.len();
        let mut number_matching = 0;

        for matcher in &webhook.matchers {
            if
                match_headers(headers, matcher)?
                || match_json(body_as_string, matcher)?
            {
                number_matching += 1;
            }
        }

        let matched = match strategy {
            MatchersStrategy::All => number_of_matchers == number_matching,
            MatchersStrategy::One => number_matching > 0,
        };

        if matched {
            let actions_to_execute = &webhook.actions_to_execute;

            info!("Matched webhook: {:?}", webhook.name);

            actions_to_add.push(actions_to_execute.clone());
        }
    }

    Ok(actions_to_add)
}
