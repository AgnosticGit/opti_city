use crate::api::auth_tokens::YANDEX_OAUTH_TOKEN;
use crate::api::urls::API_AUTH_TTS_URL;
use crate::{structs::yandex_iam_token::YandexIAMToken, IAM_TOKEN};
use reqwest::Client;
use serde_json::json;
use std::thread::sleep;
use std::time::Duration;

// Рефрешит токен каждый "every"
pub fn yandex_iam_token_refresher(every: Duration) {
    tokio::spawn(async move {
        loop {
            let client = Client::new();

            let body = json!({"yandexPassportOauthToken": YANDEX_OAUTH_TOKEN});
            let response = client
                .post(API_AUTH_TTS_URL)
                .body(body.to_string())
                .send()
                .await
                .unwrap();

            if response.status().is_success() {
                let body = response.text().await.unwrap();
                let yandex_iam_token = YandexIAMToken::from_json_string(body).unwrap();
                let mut iam_token_access = IAM_TOKEN.write().unwrap();
                *iam_token_access = Some(yandex_iam_token);
            } else {
                println!("Error: {}", response.status());
            }

            sleep(every);
        }
    });
}
