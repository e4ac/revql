#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Header(String),

    #[error("{0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Network(#[from] reqwest::Error),

    #[error("{0}")]
    Schema(&'static str),
}
