use crate::api::settings::FOLDER_ID;
use crate::api::urls::API_TTS_URL;
use crate::structs::tts_payload::TTSPayload;
use crate::{api::settings::LANGUAGE, services::service::Service};
use crate::{IAM_TOKEN, NATS_URL};
use async_nats::{Client, Message};
use bytes::Bytes;
use futures_util::StreamExt;
use reqwest::{Client as reqwest_client, Url};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct YandexASR {
    nats_client: Arc<RwLock<Client>>,
}

impl YandexASR {
    pub async fn start_service() {
        let yandex_tts = Self::init_client().await.unwrap();

        let mut subscriber = yandex_tts
            .nats_client
            .read()
            .await
            .subscribe("tts.yandex")
            .await
            .unwrap();

        tokio::spawn(async move {
            while let Some(message) = subscriber.next().await {
                yandex_tts.handle_tts_yandex(message).await;
            }
        });
    }

    async fn init_client() -> Result<Self, Box<dyn std::error::Error>> {
        let nats_client = async_nats::connect(NATS_URL).await?;

        Ok(Self {
            nats_client: Arc::new(RwLock::new(nats_client)),
        })
    }

    async fn handle_tts_yandex(&self, message: Message) {
        let payload = message.payload;

        let tts_payload = TTSPayload::from_bytes_json(payload).unwrap();
        println!("Получено сообщение: {:?}", tts_payload);

        let client = reqwest_client::new();
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
            self.publish_tts(reply_to, body).await;
        } else {
            println!("Error: {:?}", response);
            println!("Error: {}", response.status());
        }
    }

    async fn publish_tts(&self, reply_to: &str, payload: Bytes) {
        let nats_client = self.nats_client.read().await;

        let result = nats_client
            .publish(reply_to.to_string(), payload.into())
            .await;

        nats_client.flush().await.unwrap();

        println!("{:?}", result);
    }
}
