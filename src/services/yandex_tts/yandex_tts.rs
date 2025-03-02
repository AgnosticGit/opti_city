use crate::api::settings::FOLDER_ID;
use crate::api::settings::LANGUAGE;
use crate::api::urls::API_TTS_URL;
use crate::structs::failrue::Failure;
use crate::{IAM_TOKEN, NATS_URL};
use async_nats::{Client, Message};
use bytes::Bytes;
use futures_util::StreamExt;
use reqwest::{Client as reqwest_client, Url};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::structs::tts_payload::TTSPayload;

pub struct YandexTTS {
    nats_client: Arc<RwLock<Client>>,
}

impl YandexTTS {
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
                let result = yandex_tts.handle_tts_yandex(message).await;

                if result.is_ok() {
                    let (reply_to, payload) = result.unwrap();
                    yandex_tts.publish_tts(&reply_to, payload).await;
                } else {
                    let err = result.err().unwrap();
                    let reply_to = err.reply_to;

                    if reply_to.is_none() {
                        return;
                    }

                    let failure = json!( {"fail":"Service unavailable."});
                    let json_string = serde_json::to_string(&failure).unwrap();
                    let failure = Bytes::from(json_string);

                    yandex_tts
                        .publish_tts(reply_to.unwrap().as_str(), failure)
                        .await;
                }
            }
        });
    }

    async fn init_client() -> Result<Self, Box<dyn std::error::Error>> {
        let nats_client = async_nats::connect(NATS_URL).await?;

        Ok(Self {
            nats_client: Arc::new(RwLock::new(nats_client)),
        })
    }

    async fn handle_tts_yandex(&self, message: Message) -> Result<(String, Bytes), Failure> {
        let payload = message.payload;
        let headers = message.headers.ok_or_else(|| Failure {
            reply_to: None,
            message: "Отсутствуют headers",
        })?;
        let reply_to = headers
            .get("reply-to")
            .ok_or_else(|| Failure {
                reply_to: None,
                message: "Отсутствует header reply-to",
            })?
            .to_string();

        let tts_payload = TTSPayload::from_bytes_json(payload).unwrap();

        let client = reqwest_client::new();
        let mut url = Url::parse(API_TTS_URL).map_err(|_| Failure {
            reply_to: Some(reply_to.clone()),
            message: "Ошибка парсинга URL",
        })?;
        let params = tts_payload
            .to_hashmap(LANGUAGE.to_string(), FOLDER_ID.to_string())
            .map_err(|_| Failure {
                reply_to: Some(reply_to.clone()),
                message: "Ошибка обработки параметров",
            })?;

        for (key, value) in params {
            let val = value.as_str().ok_or_else(|| Failure {
                reply_to: Some(reply_to.clone()),
                message: "Нет значения",
            })?;
            url.query_pairs_mut().append_pair(&key, val);
        }

        let iam_token = IAM_TOKEN.read().unwrap().clone().unwrap().iam_token;

        let response = client
            .get(url)
            .bearer_auth(iam_token)
            .send()
            .await
            .map_err(|_| Failure {
                reply_to: Some(reply_to.clone()),
                message: "Ошибка парсинга URL",
            })?;

        if response.status().is_success() {
            return Ok((reply_to, response.bytes().await.unwrap()));
        }

        Err(Failure {
            reply_to: None,
            message: "Service unavailable",
        })
    }

    async fn publish_tts(&self, reply_to: &str, payload: Bytes) {
        let nats_client = self.nats_client.read().await;
        let _ = nats_client.publish(reply_to.to_string(), payload).await;
        nats_client.flush().await.unwrap();
    }
}
