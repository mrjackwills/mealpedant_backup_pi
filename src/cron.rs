use jiff::tz::TimeZone;
use tokio::sync::broadcast::Sender;

use crate::{app_env::AppEnv, ws::InternalMessage};

/// A basic cron like structure, in order to request a new backup at a specific
pub struct Croner;

impl Croner {
    /// create a looper and spawn into it's own async thread
    pub fn init(sx: Sender<InternalMessage>, app_env: &AppEnv) {
        let looper = Self {};
        let timezone = app_env.timezone.clone();
        let download_time = app_env.download_time;
        tokio::spawn(async move { looper.init_loop(sx, timezone, download_time).await });
    }

    /// loop every 60 second,check if its $DL_TIME, and send internal file request message, which, if connected to ws, will send a ws message
    async fn init_loop(&self, sx: Sender<InternalMessage>, timezone: TimeZone, dl_time: (i8, i8)) {
        loop {
            let now = jiff::Timestamp::now().to_zoned(timezone.clone()).datetime();
            // let now = OffsetDateTime::now_utc().to_offset(timezone.get_offset());
            if now.hour() == dl_time.0 && now.minute() == dl_time.1 {
                tracing::info!("sending backup request to via internal ThreadChannel");
                sx.send(InternalMessage::RequestBackup).ok();
            }
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }
}
