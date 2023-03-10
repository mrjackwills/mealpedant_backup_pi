use time::OffsetDateTime;
use tokio::sync::broadcast::Sender;
use tracing::info;

use crate::{env::EnvTimeZone, ws::InternalMessage};

/// A basic cron like structure, in order to request a new backup at a specific
pub struct Croner;

impl Croner {
    /// create a looper and spawn into it's own async thread
    pub async fn init(sx: Sender<InternalMessage>, timezone: EnvTimeZone) {
        let looper = Self {};
        tokio::spawn(async move { looper.init_loop(sx, timezone).await });
    }

    /// loop every 60 second,check if its 3am local time, and send internal file request message, which, if connected to ws, will send a ws message
    async fn init_loop(&self, sx: Sender<InternalMessage>, timezone: EnvTimeZone) {
        loop {
            let now = OffsetDateTime::now_utc().to_offset(timezone.get_offset());
            if now.hour() == 3 && now.minute() == 0 {
                info!("sending backup request to via internal ThreadChannel");
                sx.send(InternalMessage::RequestBackup).ok();
            }
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }
}
