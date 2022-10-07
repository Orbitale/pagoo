use crate::config::Config;
use crate::config::MatchersStrategy;
use crate::config::Webhook;
use crate::matchers::headers::match_headers;
use crate::matchers::json::match_json;
use actix_web::http::header::HeaderMap;

pub(crate) fn from_request_parts(
    config: &Config,
    body_as_string: &String,
    headers: &HeaderMap,
) -> Result<Vec<Webhook>, anyhow::Error> {
    let mut matching_webhooks: Vec<Webhook> = Vec::new();

    for webhook in &config.webhooks {
        let strategy = webhook.matchers_strategy.clone().unwrap_or_default();
        let number_of_matchers = webhook.matchers.len();
        let mut number_matching = 0;

        for matcher in &webhook.matchers {
            if match_headers(headers, matcher)? || match_json(body_as_string, matcher)? {
                number_matching += 1;
            }
        }

        let matched = match strategy {
            MatchersStrategy::All => number_of_matchers == number_matching,
            MatchersStrategy::One => number_matching > 0,
        };

        if matched {
            matching_webhooks.push(webhook.clone());
        }
    }

    Ok(matching_webhooks)
}
