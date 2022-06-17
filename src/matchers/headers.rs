use actix_web::http::header::HeaderMap;
use crate::config::config::Matcher;

pub(crate) fn match_headers(headers: &HeaderMap, matcher: &Matcher) -> Result<bool, anyhow::Error> {
    if matcher.match_headers.is_none() {
        return Ok(false);
    }

    let matcher_headers = matcher.match_headers.as_ref().ok_or(anyhow::anyhow!("No match_headers"))?;
    let number_of_headers = matcher_headers.len();
    let mut headers_matching = 0;

    for (header_name, header_value) in matcher_headers {
        if headers.contains_key(header_name) {
            let header_value_as_string = headers.get(header_name).ok_or(anyhow::anyhow!("Could not get header by name \"{}\".", header_name))?.to_str()?;
            if header_value_as_string == header_value {
                headers_matching += 1
            }
        }
    }

    return Ok(headers_matching == number_of_headers);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use actix_web::http::header::HeaderName;
    use actix_web::http::header::HeaderValue;

    #[test]
    fn test_match_headers_with_different_case() {
        let mut headers = HeaderMap::new();
        headers.insert(HeaderName::from_static("x-header-1"), HeaderValue::from_static("value1"));
        headers.insert(HeaderName::from_static("x-header-2"), HeaderValue::from_static("value2"));

        let matcher = Matcher {
            match_headers: Some(HashMap::from([
                ("X-HEADER-1".to_string(), "value1".to_string()),
                ("X-HEADER-2".to_string(), "value2".to_string()),
            ])),
            match_json_body: None,
        };

        assert_eq!(match_headers(&headers, &matcher).unwrap(), true);
    }

    #[test]
    fn test_not_match_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(HeaderName::from_static("x-header-1"), HeaderValue::from_static("value1"));
        headers.insert(HeaderName::from_static("x-header-2"), HeaderValue::from_static("value2"));

        let matcher = Matcher {
            match_headers: Some(HashMap::from([
                ("X-Wrong-HEADER-1".to_string(), "value1".to_string()),
                ("X-Wrong-HEADER-2".to_string(), "value2".to_string()),
            ])),
            match_json_body: None,
        };

        assert_eq!(match_headers(&headers, &matcher).unwrap(), false);
    }
}
