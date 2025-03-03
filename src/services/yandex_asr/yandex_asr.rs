use crate::api::urls::API_STT_URL;
use crate::yandex::recognizer_client::RecognizerClient;
use crate::{yandex, IAM_TOKEN};
use async_nats::Client;
use futures_util::stream;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::service::Interceptor;
use tonic::transport::Channel;
use tonic::{Request, Status};

pub struct YandexASR {
    nats_client: Arc<RwLock<Client>>,
}

impl YandexASR {
    pub async fn start_service() {
        let channel = Channel::from_static(API_STT_URL).connect().await.unwrap();
        let token = IAM_TOKEN.read().await.clone().unwrap().iam_token;
        let interceptor = AuthInterceptor::new(token);
        let mut connection = RecognizerClient::with_interceptor(channel, interceptor);

        let event = yandex::streaming_request::Event::SilenceChunk(yandex::SilenceChunk {
            duration_ms: 60,
        });

        let request = yandex::StreamingRequest { event: Some(event) };
        let r = vec![request];
        let request_stream = stream::iter(r);

        let response_stream = connection.recognize_streaming(request_stream).await;

        let mut response = response_stream.unwrap();

        println!("{:?}", response);

        while let Ok(resp) = response.get_mut().message().await {
            println!("Received response: {:?}", resp);
        }
    }
}

#[derive(Clone)]
pub struct AuthInterceptor {
    token: String,
}

impl AuthInterceptor {
    /// Создать новый интерсептор с заданным токеном
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        req.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", self.token)
                .parse()
                .map_err(|_| Status::internal("Невалидный токен"))?,
        );
        Ok(req)
    }
}
