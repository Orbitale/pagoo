use crate::config::config::Matcher;

pub(crate) fn match_json(body_as_string: &String, matcher: &Matcher) -> Result<bool, anyhow::Error> {
    if matcher.match_json_body.is_none() {
        return Ok(false);
    }

    let match_json_body = matcher.match_json_body.as_ref().ok_or(anyhow::anyhow!("No match_json_body"))?;
    let match_json_body = serde_json::json!(match_json_body);

    let deserialized_result = serde_json::from_str::<serde_json::Value>(body_as_string.as_str());
    if deserialized_result.is_err() {
        debug!("Deserialization failed, skipping JSON matcher.");
        debug!("Deserialization error: {}", deserialized_result.unwrap_err());
        return Ok(false);
    }

    let deserialized_json = deserialized_result?;

    let json_comparator_config = assert_json_diff::Config::new(assert_json_diff::CompareMode::Strict);
    let matching_json_result = assert_json_diff::assert_json_matches_no_panic(&deserialized_json, &match_json_body, json_comparator_config);

    if matching_json_result.is_ok() {
        return Ok(true);
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_json() {
        let body_as_string = "{\"key1\": \"value1\", \"key2\": \"value2\"}".into();

        let matcher = Matcher {
            match_headers: None,
            match_json_body: Some(serde_json::json!({
            "key1": "value1",
            "key2": "value2",
        })),
        };

        assert_eq!(match_json(&body_as_string, &matcher).unwrap(), true);
    }

    #[test]
    fn test_not_match_json() {
        let body_as_string = "{\"key1\": \"value1\", \"key2\": \"value2\"}".into();

        let matcher = Matcher {
            match_headers: None,
            match_json_body: Some(serde_json::json!({
            "wrongKey1": "value1",
            "wrongKey2": "value2",
        })),
        };

        assert_eq!(match_json(&body_as_string, &matcher).unwrap(), false);
    }
}
