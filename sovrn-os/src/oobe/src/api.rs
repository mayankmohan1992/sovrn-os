// OOBE API client - calls sovrnd for identity/domain setup

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const SOVRND_URL: &str = "http://127.0.0.1:54771/api";

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityResponse {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub public_key: String,
    pub seed_phrase: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DomainCheckResponse {
    pub available: bool,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DomainRegisterResponse {
    pub domain: String,
    pub registered: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub services: HashMap<String, ServiceHealth>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: String,
}

pub struct OobeClient {
    client: reqwest::Client,
}

impl OobeClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn check_health(&self) -> anyhow::Result<HealthResponse> {
        let resp = self.client
            .get(format!("{}/health", SOVRND_URL))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn create_identity(&self) -> anyhow::Result<IdentityResponse> {
        let resp = self.client
            .post(format!("{}/auth/create_identity", SOVRND_URL))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn check_domain(&self, name: &str) -> anyhow::Result<DomainCheckResponse> {
        let resp = self.client
            .get(format!("{}/dht/check/{}", SOVRND_URL, name))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn register_domain(&self, name: &str, public_key: &str) -> anyhow::Result<DomainRegisterResponse> {
        let resp = self.client
            .post(format!("{}/dht/register", SOVRND_URL))
            .json(&serde_json::json!({
                "name": name,
                "public_key": public_key,
                "signature": "oobe-setup"
            }))
            .send()
            .await?;
        Ok(resp.json().await?)
    }
}
