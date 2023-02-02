mod connect;
mod connection_details;

use connect::ws_upgrade;
use connection_details::ConnectionDetails;
use futures_util::{
    lock::Mutex,
    stream::{SplitSink, SplitStream},
    StreamExt, TryStreamExt,
};
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::broadcast::{Receiver, Sender},
    task::JoinHandle,
};
use tokio_tungstenite::{self, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{error, info};

mod ws_sender;

use crate::{env::AppEnv, ws_messages::Response};

use self::ws_sender::WSSender;

#[derive(Debug, Clone, Copy)]
pub enum InternalMessage {
    RequestBackup,
}

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WSReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type WSWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

#[derive(Debug, Default)]
struct AutoClose(Option<JoinHandle<()>>);

impl AutoClose {
    /// Will close the connection after 40 seconds, unless it is called within that 40 seconds
    /// Get called on every ping recevied (server sends a ping every 30 seconds)
    fn on_ping(&mut self, ws_sender: &WSSender) {
        if let Some(handle) = self.0.as_ref() {
            handle.abort();
        };
        let mut ws_sender = ws_sender.clone();
        self.0 = Some(tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(40)).await;
            ws_sender.close().await;
        }));
    }
}

/// Handle each incoming ws message
async fn incoming_ws_message(mut reader: WSReader, mut ws_sender: WSSender) {
    let mut auto_close = AutoClose::default();
    auto_close.on_ping(&ws_sender);
    while let Ok(Some(message)) = reader.try_next().await {
        match message {
            Message::Text(message) => {
                let mut ws_sender = ws_sender.clone();
                tokio::spawn(async move {
                    ws_sender.on_text(message).await;
                });
            }
            Message::Ping(_) => auto_close.on_ping(&ws_sender),
            Message::Close(_) => {
                ws_sender.close().await;
                break;
            }
            _ => (),
        };
    }
    info!("incoming_ws_message done");
}

/// Send a ws message to request a backup file, is executed by a cron type job on abother thread
async fn incoming_internal_message(mut rx: Receiver<InternalMessage>, mut ws_sender: WSSender) {
    while let Ok(_message) = rx.recv().await {
        ws_sender.send_backup_request(Response::Backup).await;
    }
}

/// try to open WS connection, and spawn a ThreadChannel message handler
pub async fn open_connection(app_envs: AppEnv, rx: Sender<InternalMessage>) {
    let mut connection_details = ConnectionDetails::new();
    loop {
        info!("in connection loop, awaiting delay then try to connect");
        connection_details.reconnect_delay().await;

        match ws_upgrade(&app_envs).await {
            Ok(socket) => {
                info!("connected in ws_upgrade match");
                connection_details.valid_connect();

                let (writer, reader) = socket.split();
                let writer = Arc::new(Mutex::new(writer));

                let ws_sender = WSSender::new(app_envs.clone(), writer);

                let in_ws_sender = ws_sender.clone();
                let rx = rx.subscribe();
                let internal_message_thread = tokio::spawn(async move {
                    incoming_internal_message(rx, in_ws_sender).await;
                });

                incoming_ws_message(reader, ws_sender).await;

                internal_message_thread.abort();
                info!("aborted spawns, incoming_ws_message done, reconnect next");
            }
            Err(e) => {
                error!("connect::{e}");
                connection_details.fail_connect();
            }
        }
    }
}
