use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::registry::Module;

const API_URL: &str = "https://registrar-agentartificial.ngrok.dev/modules/translation";

#[derive(Debug, Serialize, Deserialize)]
struct TranslationRequest {
    text: String,
    source_language: String,
    target_language: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TranslationResponse {
    translated_text: String,
}

pub struct TranslationModule {
    client: reqwest::Client,
}

impl TranslationModule {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn translate(&self, text: &str, source_lang: &str, target_lang: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = TranslationRequest {
            text: text.to_string(),
            source_language: source_lang.to_string(),
            target_language: target_lang.to_string(),
        };

        let response = self.client.get(API_URL)
            .json(&request)
            .send()
            .await?
            .json::<TranslationResponse>()
            .await?;

        Ok(response.translated_text)
    }
}

#[async_trait]
impl Module for TranslationModule {
    async fn load(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Translation module loaded");
        Ok(())
    }

    async fn unload(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Translation module unloaded");
        Ok(())
    }
}