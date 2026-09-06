use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiStatus {
    Disabled,
    Ready(String),
    Generating,
    Error(String),
}

impl AiStatus {
    pub fn display_text(&self) -> String {
        match self {
            AiStatus::Disabled => "󰚩 AI: Off".to_string(),
            AiStatus::Ready(model) => format!("󰚩 AI: Ready ({})", model),
            AiStatus::Generating => "󰚩 AI: Generating...".to_string(),
            AiStatus::Error(err) => format!("󰚩 AI: Error ({})", err),
        }
    }
}

#[derive(Debug, Serialize)]
struct GeneratePayload {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    suffix: Option<String>,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Debug, Serialize)]
struct GenerateOptions {
    temperature: f32,
    num_predict: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    name: String,
}

#[derive(Debug, Clone)]
pub struct OllamaClient {
    pub endpoint: String,
    pub active_model: String,
    pub available_models: Vec<String>,
    pub is_enabled: bool,
    client: reqwest::Client,
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new("http://localhost:11434")
    }
}

impl OllamaClient {
    pub fn new(endpoint: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            active_model: "deepseek-coder-v2:16b".to_string(),
            available_models: vec!["deepseek-coder-v2:16b".to_string()],
            is_enabled: true,
            client,
        }
    }

    pub async fn fetch_models(&mut self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/tags", self.endpoint);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to reach Ollama: {e}"))?;

        let tags: TagsResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse tags JSON: {e}"))?;

        let mut models: Vec<String> = tags.models.into_iter().map(|m| m.name).collect();

        // Prioritize coder models
        models.sort_by(|a, b| {
            let a_coder = a.contains("coder") || a.contains("code");
            let b_coder = b.contains("coder") || b.contains("code");
            match (a_coder, b_coder) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.cmp(b),
            }
        });

        if let Some(first) = models.first() {
            if !models.contains(&self.active_model) {
                self.active_model = first.clone();
            }
        }

        self.available_models = models.clone();
        Ok(models)
    }

    pub async fn generate_fim(
        &self,
        prefix: &str,
        suffix: &str,
    ) -> Result<String, String> {
        if !self.is_enabled {
            return Ok(String::new());
        }

        let url = format!("{}/api/generate", self.endpoint);
        let model = &self.active_model;

        let (prompt, opt_suffix, stop_tokens) = if model.contains("deepseek") {
            let p = format!("<｜fim begin｜>{}<｜fim hole｜>{}<｜fim end｜>", prefix, suffix);
            (
                p,
                None,
                vec![
                    "<｜fim begin｜>".into(),
                    "<｜fim hole｜>".into(),
                    "<｜fim end｜>".into(),
                    "\n\n".into(),
                ],
            )
        } else {
            (
                prefix.to_string(),
                Some(suffix.to_string()),
                vec!["<EOT>".into(), "<file_sep>".into()],
            )
        };

        let payload = GeneratePayload {
            model: model.clone(),
            prompt,
            suffix: opt_suffix,
            stream: false,
            options: GenerateOptions {
                temperature: 0.15,
                num_predict: 48,
                stop: stop_tokens,
            },
        };

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Ollama request error: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Ollama returned status {}", resp.status()));
        }

        let result: GenerateResponse = resp
            .json()
            .await
            .map_err(|e| format!("Invalid JSON response: {e}"))?;

        let mut clean = result.response;
        if let Some(pos) = clean.find("<｜fim") {
            clean.truncate(pos);
        }

        let lines: Vec<&str> = clean.lines().take(2).collect();
        let final_text = if lines.len() == 1 {
            lines[0].to_string()
        } else if lines.len() == 2 {
            format!("{}\n{}", lines[0], lines[1])
        } else {
            clean
        };

        Ok(final_text)
    }
}
