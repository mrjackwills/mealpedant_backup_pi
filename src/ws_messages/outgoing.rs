use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

/// either sent as is, or nested in StructuredResponse below
/// Should prbably be renamed request
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "name", content = "data")]
pub enum Response {
    Backup,
}

/// These get sent to the websocket server when in structured_data mode,
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct StructuredResponse {
    data: Option<Response>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Response>,
    unique: bool,
}

impl StructuredResponse {
    /// Convert a ResponseMessage into a Tokio message of StructureResponse
    pub fn data(data: Response) -> Message {
        let x = Self {
            data: Some(data),
            error: None,
            unique: true,
        };
        Message::Text(serde_json::to_string(&x).unwrap_or_default().into())
    }

    /// Convert a ErrorResponse into a Tokio message of StructureResponse
    pub fn _error(data: Response) -> Message {
        let x = Self {
            error: Some(data),
            data: None,
            unique: true,
        };
        Message::Text(serde_json::to_string(&x).unwrap_or_default().into())
    }
}
