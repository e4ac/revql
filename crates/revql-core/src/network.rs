use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Client, RequestBuilder};
use serde_json::json;
use std::str::FromStr;

const DEFAULT_QUERY: &str = include_str!("resources/introspection.graphql");

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("{0}")]
    Header(String),

    #[error("{0}")]
    Request(#[from] reqwest::Error),
}

pub async fn post_network_content(url: &str, headers: &[String], query: Option<String>) -> Result<String, NetworkError> {
    let builder = get_request_builder(url, headers, true)?;
    let query = query.unwrap_or_else(|| DEFAULT_QUERY.to_owned());
    Ok(builder.json(&json!({"query": query})).send().await?.text().await?)
}

pub async fn get_network_content(url: &str, headers: &[String], query: Option<String>) -> Result<String, NetworkError> {
    let builder = get_request_builder(url, headers, false)?;
    let query = query.unwrap_or_else(|| DEFAULT_QUERY.to_owned());
    Ok(builder.query(&[("query", query)]).send().await?.text().await?)
}

fn get_request_builder(url: &str, headers: &[String], is_post: bool) -> Result<RequestBuilder, NetworkError> {
    let client = Client::new();
    let mut builder = if is_post { client.post(url) } else { client.get(url) };
    for header in headers {
        let (k, v) = header
            .split_once(':')
            .ok_or_else(|| NetworkError::Header(format!("Invalid header: '{header}'")))?;

        let name = HeaderName::from_str(k.trim()).map_err(|_| NetworkError::Header(format!("Invalid name: '{k}'")))?;
        let val = HeaderValue::from_str(v.trim()).map_err(|_| NetworkError::Header(format!("Invalid value: '{v}'")))?;
        builder = builder.header(name, val);
    }
    Ok(builder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{get_network_content, post_network_content};

    #[tokio::test]
    async fn test_post_network_content() -> Result<(), NetworkError> {
        let response = post_network_content("https://graphqlzero.almansi.me/api", &[], None).await?;
        let expected = r#"{"data":{"__schema":{"queryType":{"name":"Query"},"mutationType":{"name":"Mutation"}"#;
        assert!(response.starts_with(expected));
        Ok(())
    }

    #[tokio::test]
    async fn test_post_network_content_with_custom_query() -> Result<(), NetworkError> {
        let query = "query IntrospectionQuery { __schema { queryType { name }, mutationType { name } } }";
        let response = post_network_content("https://graphqlzero.almansi.me/api", &[], Some(query.to_owned())).await?;
        let expected = r#"{"data":{"__schema":{"queryType":{"name":"Query"},"mutationType":{"name":"Mutation"}}}}"#;
        assert_eq!(response.trim(), expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_network_content() -> Result<(), NetworkError> {
        let headers = [
            "Content-Type: application/json".to_owned(),
            "apollo-require-preflight: true".to_owned(),
        ];

        let response = get_network_content("https://graphqlzero.almansi.me/api", &headers, None).await?;
        let expected = r#"{"data":{"__schema":{"queryType":{"name":"Query"},"mutationType":{"name":"Mutation"}"#;
        assert!(response.starts_with(expected));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_network_content_with_custom_query() -> Result<(), NetworkError> {
        let query = "query IntrospectionQuery { __schema { queryType { name }, mutationType { name } } }";
        let headers = [
            "Content-Type: application/json".to_owned(),
            "apollo-require-preflight: true".to_owned(),
        ];

        let response = get_network_content("https://graphqlzero.almansi.me/api", &headers, Some(query.to_owned())).await?;
        let expected = r#"{"data":{"__schema":{"queryType":{"name":"Query"},"mutationType":{"name":"Mutation"}}}}"#;
        assert_eq!(response.trim(), expected);
        Ok(())
    }
}
