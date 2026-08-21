use crate::error::AppError;
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::header::{HeaderName, HeaderValue};
use serde_json::json;
use std::str::FromStr;

const DEFAULT_QUERY: &str = include_str!("resources/introspection.graphql");

pub fn post_network_content(url: &str, headers: &[String], query: Option<String>) -> Result<String, AppError> {
    let builder = get_request_builder(url, headers, true)?;
    let query = query.unwrap_or_else(|| DEFAULT_QUERY.to_owned());
    Ok(builder.json(&json!({"query": query})).send()?.text()?)
}

pub fn get_network_content(url: &str, headers: &[String], query: Option<String>) -> Result<String, AppError> {
    let builder = get_request_builder(url, headers, false)?;
    let query = query.unwrap_or_else(|| DEFAULT_QUERY.to_owned());
    Ok(builder.query(&[("query", query)]).send()?.text()?)
}

fn get_request_builder(url: &str, headers: &[String], is_post: bool) -> Result<RequestBuilder, AppError> {
    let client = Client::new();
    let mut builder = if is_post { client.post(url) } else { client.get(url) };
    for header in headers {
        let (k, v) = header
            .split_once(':')
            .ok_or_else(|| AppError::Header(format!("Invalid header: '{header}'")))?;

        let name = HeaderName::from_str(k.trim()).map_err(|_| AppError::Header(format!("Invalid name: '{k}'")))?;
        let val = HeaderValue::from_str(v.trim()).map_err(|_| AppError::Header(format!("Invalid value: '{v}'")))?;
        builder = builder.header(name, val);
    }
    Ok(builder)
}

#[cfg(test)]
mod tests {
    use crate::error::AppError;
    use crate::network::{get_network_content, post_network_content};

    #[test]
    fn test_post_network_content() -> Result<(), AppError> {
        let response = post_network_content("https://graphqlzero.almansi.me/api", &[], None)?;
        let expected = r#"{"data":{"__schema":{"queryType":{"name":"Query"},"mutationType":{"name":"Mutation"}"#;
        assert!(response.starts_with(expected));
        Ok(())
    }

    #[test]
    fn test_post_network_content_with_custom_query() -> Result<(), AppError> {
        let query = "query IntrospectionQuery { __schema { queryType { name }, mutationType { name } } }";
        let response = post_network_content("https://graphqlzero.almansi.me/api", &[], Some(query.to_owned()))?;
        let expected = r#"{"data":{"__schema":{"queryType":{"name":"Query"},"mutationType":{"name":"Mutation"}}}}"#;
        assert_eq!(response.trim(), expected);
        Ok(())
    }

    #[test]
    fn test_get_network_content() -> Result<(), AppError> {
        let headers = [
            "Content-Type: application/json".to_owned(),
            "apollo-require-preflight: true".to_owned(),
        ];

        let response = get_network_content("https://graphqlzero.almansi.me/api", &headers, None)?;
        let expected = r#"{"data":{"__schema":{"queryType":{"name":"Query"},"mutationType":{"name":"Mutation"}"#;
        assert!(response.starts_with(expected));
        Ok(())
    }

    #[test]
    fn test_get_network_content_with_custom_query() -> Result<(), AppError> {
        let query = "query IntrospectionQuery { __schema { queryType { name }, mutationType { name } } }";
        let headers = [
            "Content-Type: application/json".to_owned(),
            "apollo-require-preflight: true".to_owned(),
        ];

        let response = get_network_content("https://graphqlzero.almansi.me/api", &headers, Some(query.to_owned()))?;
        let expected = r#"{"data":{"__schema":{"queryType":{"name":"Query"},"mutationType":{"name":"Mutation"}}}}"#;
        assert_eq!(response.trim(), expected);
        Ok(())
    }
}
