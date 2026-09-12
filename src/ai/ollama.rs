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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatStreamEvent {
    Chunk(String),
    Done,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatGeneratePayload {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    stream: bool,
    options: GenerateOptions,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OllamaError {
    Disabled,
    HttpError(String),
    StatusError(u16, String),
    ParseError(String),
    StreamError(String),
    BufferExceeded,
}

impl std::fmt::Display for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OllamaError::Disabled => write!(f, "Ollama integration is disabled"),
            OllamaError::HttpError(e) => write!(f, "Ollama HTTP error: {}", e),
            OllamaError::StatusError(code, msg) => write!(f, "Ollama returned status {}: {}", code, msg),
            OllamaError::ParseError(e) => write!(f, "Ollama parse error: {}", e),
            OllamaError::StreamError(e) => write!(f, "Ollama stream error: {}", e),
            OllamaError::BufferExceeded => write!(f, "Stream line buffer exceeded safety threshold"),
        }
    }
}

impl std::error::Error for OllamaError {}

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

    pub async fn fetch_models(&mut self) -> Result<Vec<String>, OllamaError> {
        let url = format!("{}/api/tags", self.endpoint);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| OllamaError::HttpError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(OllamaError::StatusError(
                resp.status().as_u16(),
                resp.status().to_string(),
            ));
        }

        let tags: TagsResponse = resp
            .json()
            .await
            .map_err(|e| OllamaError::ParseError(e.to_string()))?;

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
    ) -> Result<String, OllamaError> {
        if !self.is_enabled {
            return Err(OllamaError::Disabled);
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
            .map_err(|e| OllamaError::HttpError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(OllamaError::StatusError(
                resp.status().as_u16(),
                resp.status().to_string(),
            ));
        }

        let result: GenerateResponse = resp
            .json()
            .await
            .map_err(|e| OllamaError::ParseError(e.to_string()))?;

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

    pub async fn chat_generate(
        &self,
        system_prompt: Option<&str>,
        prompt: &str,
    ) -> Result<String, OllamaError> {
        let url = format!("{}/api/generate", self.endpoint);
        let default_system = "You are an expert AI software engineering assistant integrated directly inside Rooney (CosmicCode) editor. Provide clean, concise code, answers, and refactorings. Wrap code snippets in markdown codeblocks.";
        let sys = system_prompt.unwrap_or(default_system).to_string();

        let payload = ChatGeneratePayload {
            model: self.active_model.clone(),
            prompt: prompt.to_string(),
            system: Some(sys),
            stream: false,
            options: GenerateOptions {
                temperature: 0.3,
                num_predict: 2048,
                stop: Vec::new(),
            },
        };

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| OllamaError::HttpError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(OllamaError::StatusError(
                resp.status().as_u16(),
                resp.status().to_string(),
            ));
        }

        let result: GenerateResponse = resp
            .json()
            .await
            .map_err(|e| OllamaError::ParseError(e.to_string()))?;

        Ok(result.response)
    }

    pub async fn chat_generate_stream(
        &self,
        system_prompt: Option<&str>,
        prompt: &str,
        tx: futures_channel::mpsc::UnboundedSender<ChatStreamEvent>,
        cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) {
        let url = format!("{}/api/generate", self.endpoint);
        let default_system = "You are an expert AI software engineering assistant integrated directly inside Rooney (CosmicCode) editor. Provide clean, concise code, answers, and refactorings. Wrap code snippets in markdown codeblocks.";
        let sys = system_prompt.unwrap_or(default_system).to_string();

        let payload = ChatGeneratePayload {
            model: self.active_model.clone(),
            prompt: prompt.to_string(),
            system: Some(sys),
            stream: true,
            options: GenerateOptions {
                temperature: 0.3,
                num_predict: 2048,
                stop: Vec::new(),
            },
        };

        let resp = match self.client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.unbounded_send(ChatStreamEvent::Error(format!("Ollama request error: {e}")));
                return;
            }
        };

        if !resp.status().is_success() {
            let _ = tx.unbounded_send(ChatStreamEvent::Error(format!("Ollama returned status {}", resp.status())));
            return;
        }

        let mut byte_stream = resp.bytes_stream();
        let mut line_buffer = String::new();

        use futures_util::StreamExt;
        while let Some(item) = byte_stream.next().await {
            if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = tx.unbounded_send(ChatStreamEvent::Done);
                return;
            }

            let bytes = match item {
                Ok(b) => b,
                Err(e) => {
                    let _ = tx.unbounded_send(ChatStreamEvent::Error(format!("Stream read error: {e}")));
                    return;
                }
            };

            line_buffer.push_str(&String::from_utf8_lossy(&bytes));

            // Memory guard: prevent unbounded memory consumption if stream contains no newlines
            if line_buffer.len() > 1024 * 1024 {
                let _ = tx.unbounded_send(ChatStreamEvent::Error(
                    "Stream line buffer exceeded 1MB safety threshold".into(),
                ));
                return;
            }

            while let Some(pos) = line_buffer.find('\n') {
                let line = line_buffer[..pos].trim().to_string();
                line_buffer = line_buffer[pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                #[derive(Deserialize)]
                struct StreamChunk {
                    response: Option<String>,
                    done: Option<bool>,
                    error: Option<String>,
                }

                if let Ok(chunk) = serde_json::from_str::<StreamChunk>(&line) {
                    if let Some(err) = chunk.error {
                        let _ = tx.unbounded_send(ChatStreamEvent::Error(err));
                        return;
                    }
                    if let Some(token) = chunk.response {
                        if !token.is_empty() {
                            let _ = tx.unbounded_send(ChatStreamEvent::Chunk(token));
                        }
                    }
                    if chunk.done.unwrap_or(false) {
                        let _ = tx.unbounded_send(ChatStreamEvent::Done);
                        return;
                    }
                }
            }
        }

        let _ = tx.unbounded_send(ChatStreamEvent::Done);
    }
}
