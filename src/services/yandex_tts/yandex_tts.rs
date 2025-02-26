use crate::services::service::Service;
use async_nats::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct YandexTTS {
    nats_client: Arc<RwLock<Client>>,
}

impl Service for YandexTTS {
    fn start(&self) {
        todo!()
    }

    // async fn init_connection() {}
}
