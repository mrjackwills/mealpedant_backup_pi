use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Base64 Decode Error")]
    B64Decode(#[from] data_encoding::DecodeError),
    #[error("FS Write error")]
    FSWrite(#[from] std::io::Error),
    #[error("File/Directory not found: '{0}'")]
    FileNotFound(String),
    #[error("missing env: '{0}'")]
    MissingEnv(String),
    // #[error("Time offset error")]
    // Offset(#[from] time::error::ComponentRange),
    #[error("Reqwest Error")]
    Reqwest(#[from] reqwest::Error),
    #[error("WS Connect")]
    TungsteniteConnect(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Invalid WS Status Code")]
    WsStatus,
}
