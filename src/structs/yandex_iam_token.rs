use chrono::{DateTime, Utc};
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct YandexIAMToken {
    pub iam_token: String,
    pub expire_at: DateTime<Utc>,
}

impl YandexIAMToken {
    pub fn from_json_string(json: String) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed: Value = serde_json::from_str(&json)?;

        let iam_token = parsed["iamToken"].as_str().unwrap().to_string();
        let expire_at_str = parsed["expiresAt"].as_str().unwrap();

        let expire_at = DateTime::from_str(expire_at_str)?;

        Ok(YandexIAMToken {
            iam_token,
            expire_at,
        })
    }
}
