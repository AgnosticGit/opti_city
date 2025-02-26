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

    tokio::spawn(async move {
        while let Some(message) = subscriber.next().await {
            handle_tts_yandex(message).await;
        }
    });

    future::pending::<()>().await;
}

async fn handle_tts_yandex(message: Message) {
    let payload = message.payload;

    let tts_payload = TTSPayload::from_bytes_json(payload).unwrap();
    println!("Получено сообщение: {:?}", tts_payload);

    let client = Client::new();
    let mut url = Url::parse(API_TTS_URL).unwrap();
    let params = tts_payload
        .to_hashmap(LANGUAGE.to_string(), FOLDER_ID.to_string())
        .unwrap();

    for (key, value) in params {
        url.query_pairs_mut()
            .append_pair(&key, &value.as_str().unwrap());
    }

    let iam_token = IAM_TOKEN.read().unwrap().clone().unwrap().iam_token;

    let response = client.get(url).bearer_auth(iam_token).send().await.unwrap();

    if response.status().is_success() {
        let body = response.bytes().await.unwrap();
        let headers = message.headers.unwrap();
        let reply_to = headers.get("reply-to").unwrap().as_str();
        println!("{}", reply_to);
        publish_tts(reply_to, body).await;
    } else {
        println!("Error: {:?}", response);
        println!("Error: {}", response.status());
    }
}

async fn publish_tts(reply_to: &str, payload: Bytes) {
    let nats_client = async_nats::connect(NATS_URL).await.unwrap();

    let result = nats_client
        .publish(reply_to.to_string(), payload.into())
        .await;

    nats_client.flush().await.unwrap();

    println!("{:?}", result);
}
