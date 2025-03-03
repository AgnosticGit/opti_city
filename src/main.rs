use api::urls::NATS_URL;
use constants::time::SECS_IN_HOUR;
use futures_util::future;
use lazy_static::lazy_static;
use log::debug;
use services::{yandex_asr::yandex_asr::YandexASR, yandex_tts::yandex_tts::YandexTTS};
use std::{sync::Arc, time::Duration};
use structs::yandex_iam_token::YandexIAMToken;
use tokio::{self, time::sleep};
use workers::yandex_iam_token_refresher::yandex_iam_token_refresher;
use tokio::sync::RwLock;

mod api;
mod constants;
mod services;
mod structs;
mod workers;

mod yandex {
    tonic::include_proto!("speechkit.stt.v3");
}

lazy_static! {
    pub static ref IAM_TOKEN: Arc<RwLock<Option<YandexIAMToken>>> = Arc::new(RwLock::new(None));
}

#[tokio::main]
async fn main() {
    init_logger();
    yandex_iam_token_refresher(Duration::from_secs(SECS_IN_HOUR)).await;

    YandexTTS::start_service().await;
    // YandexASR::start_service().await;

    future::pending::<()>().await;
}

fn init_logger() {
    let env = env_logger::Env::default().default_filter_or("debug");
    env_logger::Builder::from_env(env).init();
}
