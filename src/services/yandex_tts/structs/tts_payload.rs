use std::collections::HashMap;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::{Error, Value};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TTSPayload {
    pub voice: String,
    pub lang: Option<String>,
    pub text: String,
    #[serde(rename = "folderId")]
    pub folder_id: Option<String>,
}

impl TTSPayload {
    pub fn from_bytes_json(bytes: Bytes) -> Result<Self, Error> {
        let payload: TTSPayload = serde_json::from_slice(&bytes)?;
        Ok(payload)
    }

    pub fn to_hashmap(
        &self,
        lang: String,
        folder_id: String,
    ) -> Result<HashMap<String, Value>, serde_json::Error> {
        let json_value: Value = serde_json::to_value(self)?;
        let mut hashmap: HashMap<String, Value> = serde_json::from_value(json_value)?;

        hashmap.insert("folderId".to_string(), Value::String(folder_id));
        hashmap.insert("lang".to_string(), Value::String(lang));

        Ok(hashmap)
    }
}
