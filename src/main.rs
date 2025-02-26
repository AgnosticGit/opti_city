use api::{
    settings::{FOLDER_ID, LANGUAGE},
    urls::{API_TTS_URL, NATS_URL},
};
use async_nats::{HeaderMap, Message};
use bytes::Bytes;
use constants::time::SECS_IN_HOUR;
use futures_util::{future, StreamExt};
use lazy_static::lazy_static;
use reqwest::{Client, Url};
use services::yandex_tts::yandex_tts::YandexTTS;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use structs::{tts_payload::TTSPayload, yandex_iam_token::YandexIAMToken};
use tokio::{self};
use workers::yandex_iam_token_refresher::yandex_iam_token_refresher;

mod api;
mod constants;
mod structs;
mod workers;
mod services;

lazy_static! {
    pub static ref IAM_TOKEN: Arc<RwLock<Option<YandexIAMToken>>> = Arc::new(RwLock::new(None));
}

#[tokio::main]
async fn main() {
    let nats_client = async_nats::connect(NATS_URL).await.unwrap();
    let mut subscriber = nats_client.subscribe("tts.yandex").await.unwrap();

    // Воркер который рефрешит токен каждый час
    yandex_iam_token_refresher(Duration::from_secs(SECS_IN_HOUR));

    YandexTTS::start_service().await;


    future::pending::<()>().await;
}

