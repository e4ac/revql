use revql_core::network::NetworkError;
use revql_core::schema::SchemaError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Network(#[from] NetworkError),

    #[error("{0}")]
    Schema(#[from] SchemaError),
}
