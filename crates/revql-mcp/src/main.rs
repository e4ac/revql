use revql_core::network::{get_network_content, post_network_content};
use revql_core::schema::{MatchConfig, parse_schema};
use revql_core::search::{SearchMode, find_path_lines};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{ServerCapabilities, ServerConfig};
use rmcp::transport::stdio;
use rmcp::{ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum Mode {
    #[default]
    Type,
    Description,
    Field,
}

impl From<Mode> for SearchMode {
    #[inline]
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Type => Self::Type,
            Mode::Description => Self::Description,
            Mode::Field => Self::Field,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum HttpMethod {
    Get,
    #[default]
    Post,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct FindPathsRequest {
    /// Path to a local GraphQL introspection JSON file, or a URL when `url` is true.
    schema_path: String,
    /// Fetch the introspection result from `schema_path` as a URL instead of reading a local file.
    #[serde(default)]
    url: bool,
    /// Headers to send with the request in `Name: value` format (URL mode only).
    #[serde(default)]
    headers: Vec<String>,
    /// Custom introspection query to send instead of the default one (URL mode only).
    #[serde(default)]
    query: Option<String>,
    /// HTTP method to use: post (default) or get (URL mode only).
    #[serde(default)]
    method: HttpMethod,
    /// Text to search for.
    search: String,
    /// What to match against: type (default), description, or field.
    #[serde(default)]
    mode: Mode,
    /// Match substrings instead of whole names.
    #[serde(default)]
    contains: bool,
    /// Ignore case when matching.
    #[serde(default)]
    ignore_case: bool,
}

async fn fetch_schema(req: &FindPathsRequest) -> Result<String, String> {
    let result = match req.method {
        HttpMethod::Get => get_network_content(&req.schema_path, &req.headers, req.query.clone()).await,
        HttpMethod::Post => post_network_content(&req.schema_path, &req.headers, req.query.clone()).await,
    };
    result.map_err(|e| format!("failed to fetch schema from {}: {e}", req.schema_path))
}

#[derive(Clone)]
struct RevqlServer;

#[tool_router]
impl RevqlServer {
    #[tool(description = "Find GraphQL query and mutation paths that reach a type, description, or field")]
    async fn find_paths(&self, Parameters(req): Parameters<FindPathsRequest>) -> Result<String, String> {
        let json = if req.url {
            fetch_schema(&req).await?
        } else {
            std::fs::read_to_string(&req.schema_path)
                .map_err(|e| format!("failed to read schema file {}: {e}", req.schema_path))?
        };
        let schema = parse_schema(&json).map_err(|e| format!("failed to parse schema: {e}"))?;
        let config = MatchConfig {
            contains: req.contains,
            ignore_case: req.ignore_case,
        };
        let lines = find_path_lines(&schema, &req.search, req.mode.into(), config);
        if lines.is_empty() {
            Ok("No paths found".to_owned())
        } else {
            Ok(lines.join("\n"))
        }
    }
}

#[allow(clippy::unused_async_trait_impl)]
#[tool_handler(router = Self::tool_router())]
impl ServerHandler for RevqlServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = RevqlServer.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
