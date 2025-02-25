use api::urls::API_TTS_URL;
use async_nats::Message;
use constants::time::SECS_IN_HOUR;
use futures_util::StreamExt;
use lazy_static::lazy_static;
use reqwest::Client;
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

lazy_static! {
    pub static ref IAM_TOKEN: Arc<RwLock<Option<YandexIAMToken>>> = Arc::new(RwLock::new(None));
}

#[tokio::main]
async fn main() {
    let client = async_nats::connect("nats://localhost:4222").await.unwrap();
    let mut subscriber = client.subscribe("tts.yandex").await.unwrap();

    // Воркер который рефрешит токен каждый час
    yandex_iam_token_refresher(Duration::from_secs(SECS_IN_HOUR));

    while let Some(message) = subscriber.next().await {
        tokio::spawn(async move {
            handle_tts_yandex(message).await;
        });
    }

    println!("Exit");
}

async fn handle_tts_yandex(message: Message) {
    let headers = message.headers;
    let payload = message.payload;

    let tts_payload = TTSPayload::from_bytes_json(payload).unwrap();

    let client = Client::new();
    let iam_token = IAM_TOKEN.read().unwrap().clone().unwrap().iam_token;

    println!("{}", iam_token);

    let response = client
        .get(API_TTS_URL)
        .bearer_auth(iam_token)
        .body(tts_payload.to_json_string().unwrap())
        .send()
        .await
        .unwrap();

    if response.status().is_success() {
        let body = response.bytes().await.unwrap();
        println!("Error: {:?}", body.len());
    } else {
        println!("Error: {}", response.status());
    }
    println!("Получено сообщение: {:?}", headers);
    println!("Получено сообщение: {:?}", tts_payload);
}
