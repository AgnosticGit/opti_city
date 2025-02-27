use api::urls::NATS_URL;
use constants::time::SECS_IN_HOUR;
use futures_util::future;
use lazy_static::lazy_static;
use services::yandex_tts::yandex_tts::YandexTTS;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use structs::yandex_iam_token::YandexIAMToken;
use tokio::{self};
use workers::yandex_iam_token_refresher::yandex_iam_token_refresher;

mod api;
mod constants;
mod services;
mod structs;
mod workers;

lazy_static! {
    pub static ref IAM_TOKEN: Arc<RwLock<Option<YandexIAMToken>>> = Arc::new(RwLock::new(None));
}

#[tokio::main]
async fn main() {
    yandex_iam_token_refresher(Duration::from_secs(SECS_IN_HOUR));

    YandexTTS::start_service().await;

    future::pending::<()>().await;
}
