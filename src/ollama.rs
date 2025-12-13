//! Ollama API integration for AI inference

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Ollama API request payload
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

/// Ollama API response payload
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    #[allow(dead_code)]
    done: bool,
}

/// Client for interacting with the Ollama API
pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Send a prompt to Ollama and get the response
    pub async fn generate(&self, prompt: String, model: String) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);

        let request = OllamaRequest {
            model,
            prompt,
            stream: false,
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Ollama API error ({}): {}", status, error_text);
        }

        let ollama_response: OllamaResponse = response.json().await?;

        Ok(ollama_response.response)
    }
}
