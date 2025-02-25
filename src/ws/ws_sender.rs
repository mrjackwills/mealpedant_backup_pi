use data_encoding::BASE64;
use futures_util::SinkExt;
use futures_util::lock::Mutex;
use std::sync::Arc;
use tokio::fs;

use crate::app_error::AppError;
use crate::ws_messages::{BackupData, MessageValues, ParsedMessage, Response, StructuredResponse};
use crate::{app_env::AppEnv, ws_messages::to_struct};

use super::WSWriter;

#[derive(Debug, Clone)]
pub struct WSSender {
    app_envs: AppEnv,
    writer: Arc<Mutex<WSWriter>>,
}

impl WSSender {
    pub const fn new(app_envs: AppEnv, writer: Arc<Mutex<WSWriter>>) -> Self {
        Self { app_envs, writer }
    }

    async fn save_backup(&self, backup_data: BackupData) -> Result<(), AppError> {
        let file_name = format!(
            "{}/{}",
            self.app_envs.location_backup, backup_data.file_name
        );

        Ok(fs::write(
            file_name,
            BASE64.decode(backup_data.file_as_b64.as_bytes())?,
        )
        .await?)
    }

    /// Handle text message, in this program they will all be json text
    pub async fn on_text(&self, message: String) {
        if let Some(data) = to_struct(&message) {
            match data {
                MessageValues::Invalid(error) => tracing::error!("{error:?}"),
                MessageValues::Valid(data) => match data {
                    ParsedMessage::BackupData(backup_data) => {
                        match self.save_backup(backup_data).await {
                            Ok(()) => tracing::trace!("backup saved to disk"),
                            Err(e) => {
                                tracing::error!("Unable to save to disk::{e}");
                            }
                        }
                    }
                },
            }
        }
    }

    /// Send a message to the socket
    pub async fn send_backup_request(&self, response: Response) {
        match self
            .writer
            .lock()
            .await
            .send(StructuredResponse::data(response))
            .await
        {
            Ok(()) => tracing::trace!("Message sent"),
            Err(e) => tracing::error!("send_ws_response::SEND-ERROR::{e:?}"),
        }
    }

    /// close connection, uses a 2 second timeout
    pub async fn close(&self) {
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            self.writer.lock().await.close(),
        )
        .await
        .ok();
    }
}
