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
    #[error("Reqwest Error")]
    Reqwest(#[from] reqwest::Error),
    #[error("Ws Connect: '{0}'")]
    TungsteniteConnect(String),
    #[error("Invalid WS Status Code")]
    WsStatus,
}
