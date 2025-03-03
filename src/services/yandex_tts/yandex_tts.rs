use crate::api::settings::FOLDER_ID;
use crate::api::settings::LANGUAGE;
use crate::api::urls::API_TTS_URL;
use crate::structs::failrue::Failure;
use crate::{IAM_TOKEN, NATS_URL};
use async_nats::Subject;
use async_nats::{Client, Message};
use bytes::Bytes;
use futures_util::StreamExt;
use reqwest::{Client as reqwest_client, Url};
use serde_json::json;
use std::sync::Arc;

use super::structs::tts_payload::TTSPayload;

#[derive(Clone)]
pub struct YandexTTS {
    nats_client: Arc<Client>,
}

impl YandexTTS {
    pub async fn start_service() {
        let yandex_tts_arc = Arc::new(Self::init_client().await.unwrap());

        let mut subscriber = Arc::clone(&yandex_tts_arc)
            .nats_client
            .subscribe("tts.yandex")
            .await
            .unwrap_or_else(|_| {
                log::error!("Не удалось подключиться к tts.yandex");
                std::process::exit(1);
            });

        while let Some(message) = subscriber.next().await {
            let yandex_tts = Arc::clone(&yandex_tts_arc);

            tokio::spawn(async move {
                let result = yandex_tts.handle_tts_yandex(message).await;

                if result.is_ok() {
                    let message = result.unwrap();
                    let reply = message.reply;
                    let payload = message.payload;

                    if reply.is_some() {
                        yandex_tts
                            .publish_tts(&reply.unwrap().to_string(), payload)
                            .await;
                    }
                } else {
                    let err = result.unwrap_err();
                    let reply_to = err.reply_to;

                    if reply_to.is_none() {
                        return;
                    }

                    let failure = json!( {"fail":"Service unavailable."});
                    let json_string = serde_json::to_string(&failure);

                    if json_string.is_err() {
                        log::error!("Не удалось подключиться распарсить строку в JSON");
                        return;
                    }

                    let failure = Bytes::from(json_string.unwrap());

                    yandex_tts
                        .publish_tts(reply_to.unwrap().as_str(), failure)
                        .await;
                }
            });
        }
    }

    async fn init_client() -> Result<Self, Box<dyn std::error::Error>> {
        let nats_client = async_nats::connect(NATS_URL).await?;

        Ok(Self {
            nats_client: Arc::new(nats_client),
        })
    }

    async fn handle_tts_yandex(&self, message: Message) -> Result<Message, Failure> {
        let payload = message.payload;
        let headers = message.headers.ok_or_else(|| Failure {
            reply_to: None,
            message: "Отсутствуют headers",
            error: None,
        })?;
        let reply_to = headers
            .get("reply-to")
            .ok_or_else(|| Failure {
                reply_to: None,
                message: "Отсутствует header reply-to",
                error: None,
            })?
            .to_string();

        let tts_payload = TTSPayload::from_bytes_json(payload).map_err(|e| Failure {
            reply_to: Some(reply_to.clone()),
            message: "Некорректный payload",
            error: Some(Box::new(e)),
        })?;

        let client = reqwest_client::new();
        let mut url = Url::parse(API_TTS_URL).map_err(|e| Failure {
            reply_to: Some(reply_to.clone()),
            message: "Ошибка парсинга URL",
            error: Some(Box::new(e)),
        })?;
        let params = tts_payload
            .to_hashmap(LANGUAGE.to_string(), FOLDER_ID.to_string())
            .map_err(|e| Failure {
                reply_to: Some(reply_to.clone()),
                message: "Ошибка обработки параметров",
                error: Some(Box::new(e)),
            })?;

        for (key, value) in params {
            let val = value.as_str().ok_or_else(|| Failure {
                reply_to: Some(reply_to.clone()),
                message: "Нет значения",
                error: None,
            })?;
            url.query_pairs_mut().append_pair(&key, val);
        }

        let iam_token_guard = IAM_TOKEN.read().await;
        let iam_token_ref = iam_token_guard.as_ref().ok_or_else(|| Failure {
            reply_to: Some(reply_to.clone()),
            message: "Ошибка получения ссылки на iam_token",
            error: None,
        })?;
        let iam_token = &iam_token_ref.iam_token;

        let response = client
            .get(url)
            .bearer_auth(iam_token)
            .send()
            .await
            .map_err(|e| Failure {
                reply_to: Some(reply_to.clone()),
                message: "Ошибка парсинга URL",
                error: Some(Box::new(e)),
            })?;

        if response.status().is_success() {
            let bytes = response.bytes().await.map_err(|e| Failure {
                reply_to: Some(reply_to.clone()),
                message: "Неудалось распарсить ответ yandex сервера",
                error: Some(Box::new(e)),
            })?;

            let subject_parsed = Subject::from_utf8(reply_to.clone()).map_err(|e| Failure {
                reply_to: Some(reply_to.clone()),
                message: "Неудалось создать subject",
                error: Some(Box::new(e)),
            });

            let subject = subject_parsed.unwrap();

            return Ok(Message {
                reply: Some(subject.clone()),
                payload: bytes,
                headers: None,
                description: None,
                length: 0,
                subject,
                status: None,
            });
        }

        Err(Failure {
            reply_to: None,
            message: "Service unavailable",
            error: None,
        })
    }

    async fn publish_tts(&self, reply_to: &str, payload: Bytes) {
        let nats_client = self.nats_client.clone();
        let _ = nats_client.publish(reply_to.to_string(), payload).await;
        nats_client.flush().await;
    }
}
