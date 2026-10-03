#![cfg(test)]

use serde_json::{Value, json};
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

fn get_test_file_path(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/")
        .join(name)
        .to_string_lossy()
        .to_string()
}

fn initialize_request() -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "0.0.1" }
        }
    })
}

fn initialized_notification() -> Value {
    json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })
}

fn run_session(requests: &[Value]) -> io::Result<Vec<Value>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_revql-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let mut stdin = child.stdin.take().ok_or_else(|| io::Error::other("missing server stdin"))?;
    for request in requests {
        writeln!(stdin, "{request}")?;
    }
    drop(stdin);

    let output = child.wait_with_output()?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<Value>(line).map_err(io::Error::other))
        .collect()
}

fn find_result(responses: &[Value], id: u64) -> io::Result<&Value> {
    responses
        .iter()
        .find(|response| response["id"] == id)
        .map(|response| &response["result"])
        .ok_or_else(|| io::Error::other(format!("no response for id {id}")))
}

fn call_find_paths(arguments: &Value) -> io::Result<Value> {
    let responses = run_session(&[
        initialize_request(),
        initialized_notification(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "find_paths", "arguments": arguments.clone() }
        }),
    ])?;
    Ok(find_result(&responses, 2)?.clone())
}

fn result_text(result: &Value) -> &str {
    result.pointer("/content/0/text").and_then(Value::as_str).unwrap_or_default()
}

fn is_error(result: &Value) -> bool {
    result.pointer("/isError").and_then(Value::as_bool).unwrap_or(false)
}

#[test]
fn test_initialize() -> io::Result<()> {
    let responses = run_session(&[initialize_request(), initialized_notification()])?;
    let result = find_result(&responses, 1)?;
    assert!(result.pointer("/capabilities/tools").is_some_and(Value::is_object));
    Ok(())
}

#[test]
fn test_tools_list() -> io::Result<()> {
    let responses = run_session(&[
        initialize_request(),
        initialized_notification(),
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
    ])?;
    let result = find_result(&responses, 2)?;
    let tools = result
        .pointer("/tools")
        .and_then(Value::as_array)
        .ok_or_else(|| io::Error::other("tools is not an array"))?;
    let tool = tools
        .iter()
        .find(|tool| tool.pointer("/name").and_then(Value::as_str) == Some("find_paths"))
        .ok_or_else(|| io::Error::other("find_paths tool not found"))?;
    let properties = tool
        .pointer("/inputSchema/properties")
        .ok_or_else(|| io::Error::other("missing input schema properties"))?;
    for key in ["schema_path", "search", "mode", "contains", "ignore_case"] {
        assert!(properties.get(key).is_some(), "missing input property {key}");
    }
    Ok(())
}

#[test]
fn test_find_paths_type() -> io::Result<()> {
    let path = get_test_file_path("sample.json");
    let result = call_find_paths(&json!({ "schema_path": path, "search": "User" }))?;
    let text = result_text(&result);
    assert!(!is_error(&result));
    assert!(text.contains("User: Query.getUser"));
    assert!(text.contains("User: Query.getBlog -> Blog.author"));
    Ok(())
}

#[test]
fn test_find_paths_ignore_case() -> io::Result<()> {
    let path = get_test_file_path("sample.json");
    let result = call_find_paths(&json!({ "schema_path": path, "search": "blog", "ignore_case": true }))?;
    let text = result_text(&result);
    assert!(!is_error(&result));
    assert!(text.contains("Blog: Query.getUser -> User.blogs"));
    assert!(text.contains("Blog: Query.getBlog"));
    Ok(())
}

#[test]
fn test_find_paths_contains() -> io::Result<()> {
    let path = get_test_file_path("sample.json");
    let result = call_find_paths(&json!({ "schema_path": path, "search": "Us", "contains": true }))?;
    let text = result_text(&result);
    assert!(!is_error(&result));
    assert!(text.contains("User: Query.getUser"));
    assert!(text.contains("User: Query.getBlog -> Blog.author"));
    Ok(())
}

#[test]
fn test_find_paths_contains_ignore_case() -> io::Result<()> {
    let path = get_test_file_path("sample.json");
    let result = call_find_paths(&json!({
        "schema_path": path,
        "search": "us",
        "contains": true,
        "ignore_case": true
    }))?;
    let text = result_text(&result);
    assert!(!is_error(&result));
    assert!(text.contains("User: Query.getUser"));
    assert!(text.contains("User: Query.getBlog -> Blog.author"));
    Ok(())
}

#[test]
fn test_find_paths_field_mode() -> io::Result<()> {
    let path = get_test_file_path("sample.json");
    let result = call_find_paths(&json!({ "schema_path": path, "search": "id", "mode": "field" }))?;
    let text = result_text(&result);
    assert!(!is_error(&result));
    assert!(text.contains("User: Query.getUser"));
    assert!(text.contains("User: Query.getBlog -> Blog.author"));
    assert!(text.contains("Image: Query.getUser -> User.profileImage"));
    assert!(text.contains("Image: Query.getUser -> User.blogs -> Blog.coverImage"));
    assert!(text.contains("Image: Query.getBlog -> Blog.coverImage"));
    assert!(text.contains("Image: Query.getBlog -> Blog.author -> User.profileImage"));
    assert!(text.contains("Blog: Query.getUser -> User.blogs"));
    assert!(text.contains("Blog: Query.getBlog"));
    Ok(())
}

#[test]
fn test_find_paths_description_mode() -> io::Result<()> {
    let path = get_test_file_path("sample.json");
    let result = call_find_paths(&json!({
        "schema_path": path,
        "search": "id of the",
        "mode": "description",
        "contains": true,
        "ignore_case": true
    }))?;
    let text = result_text(&result);
    assert!(!is_error(&result));
    assert!(text.contains("User: Query.getUser"));
    assert!(text.contains("User: Query.getBlog -> Blog.author"));
    Ok(())
}

#[test]
fn test_find_paths_no_match() -> io::Result<()> {
    let path = get_test_file_path("sample.json");
    let result = call_find_paths(&json!({ "schema_path": path, "search": "NoSuchType" }))?;
    assert!(!is_error(&result));
    assert_eq!(result_text(&result), "No paths found");
    Ok(())
}

#[test]
fn test_find_paths_missing_file() -> io::Result<()> {
    let path = get_test_file_path("does-not-exist.json");
    let result = call_find_paths(&json!({ "schema_path": path, "search": "User" }))?;
    assert!(is_error(&result));
    assert!(result_text(&result).contains("failed to read schema file"));
    Ok(())
}

#[test]
fn test_find_paths_url_post() -> io::Result<()> {
    let result = call_find_paths(&json!({
        "schema_path": "https://graphqlzero.almansi.me/api",
        "url": true,
        "headers": ["Content-Type: application/json"],
        "search": "User"
    }))?;
    let text = result_text(&result);
    assert!(!is_error(&result));
    assert!(text.contains("User: Query.user"));
    Ok(())
}

#[test]
fn test_find_paths_url_get() -> io::Result<()> {
    let result = call_find_paths(&json!({
        "schema_path": "https://graphqlzero.almansi.me/api",
        "url": true,
        "method": "get",
        "headers": ["Content-Type: application/json", "apollo-require-preflight: true"],
        "search": "User"
    }))?;
    let text = result_text(&result);
    assert!(!is_error(&result));
    assert!(text.contains("User: Query.user"));
    Ok(())
}

#[test]
fn test_find_paths_url_unreachable() -> io::Result<()> {
    let result = call_find_paths(&json!({
        "schema_path": "http://127.0.0.1:1/graphql",
        "url": true,
        "search": "User"
    }))?;
    assert!(is_error(&result));
    assert!(result_text(&result).contains("failed to fetch schema"));
    Ok(())
}

#[test]
fn test_find_paths_invalid_json() -> io::Result<()> {
    let mut file = NamedTempFile::new()?;
    writeln!(file, "{{ not json")?;
    file.flush()?;

    let path = file.path().to_string_lossy().to_string();
    let result = call_find_paths(&json!({ "schema_path": path, "search": "User" }))?;
    assert!(is_error(&result));
    assert!(result_text(&result).contains("failed to parse schema"));
    Ok(())
}
