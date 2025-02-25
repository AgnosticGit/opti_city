use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TTSPayload {
    pub voice: String,
    pub lang: Option<String>,
    pub text: String,
}

impl TTSPayload {
    pub fn from_bytes_json(bytes: Bytes) -> Result<Self, Box<dyn std::error::Error>> {
        let payload: TTSPayload = serde_json::from_slice(&bytes)?;
        Ok(payload)
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
