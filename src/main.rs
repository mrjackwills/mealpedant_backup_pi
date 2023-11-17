// Only allow when debugging
// #![allow(unused)]

mod app_env;
mod app_error;
mod cron;
mod word_art;
mod ws;
mod ws_messages;

use app_env::AppEnv;
use cron::Croner;
use tokio::sync::broadcast;
use word_art::Intro;
use ws::open_connection;

fn setup_tracing(app_env: &AppEnv) {
    tracing_subscriber::fmt()
        .with_max_level(app_env.log_level)
        .init();
}

#[tokio::main]
async fn main() {
    let app_envs = AppEnv::get();
    setup_tracing(&app_envs);
    Intro::new(&app_envs).show();
    let (sx, _keep_alive) = broadcast::channel(128);
    Croner::init(sx.clone(), &app_envs);
    open_connection(app_envs, sx).await;
}
