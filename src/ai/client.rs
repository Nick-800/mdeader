use crate::config::AiConfig;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    system: String,
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct OllamaStreamChunk {
    #[serde(default)]
    response: String,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiStreamChunk {
    #[serde(default)]
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiStreamChoice {
    #[serde(default)]
    delta: OpenAiDelta,
    #[allow(dead_code)]
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct OpenAiDelta {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Clone, Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction>,
}

#[derive(Debug, Clone, Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiResponse {
    #[serde(default)]
    candidates: Vec<GeminiCandidate>,
    #[serde(default)]
    error: Option<GeminiError>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiCandidate {
    content: GeminiCandidateContent,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiCandidateContent {
    #[serde(default)]
    parts: Vec<GeminiCandidatePart>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiCandidatePart {
    #[serde(default)]
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiError {
    message: String,
}

pub struct AiClient;

impl AiClient {
    /// Streams tokens from the configured AI provider, invoking on_token for each received token chunk.
    pub fn stream_query<F>(
        config: &AiConfig,
        system_prompt: &str,
        user_prompt: &str,
        cancel_token: Arc<AtomicBool>,
        on_token: F,
    ) -> Result<(), String>
    where
        F: FnMut(&str) -> bool, // returns true to continue, false to stop
    {
        match config.provider.to_lowercase().as_str() {
            "ollama" => Self::stream_ollama(config, system_prompt, user_prompt, cancel_token, on_token),
            "gemini" => Self::query_gemini(config, system_prompt, user_prompt, cancel_token, on_token),
            "custom" | "openai" => {
                Self::stream_openai(config, system_prompt, user_prompt, cancel_token, on_token)
            }
            other => Err(format!("Unsupported AI provider: '{}'. Supported: ollama, gemini, custom", other)),
        }
    }

    fn stream_ollama<F>(
        config: &AiConfig,
        system_prompt: &str,
        user_prompt: &str,
        cancel_token: Arc<AtomicBool>,
        mut on_token: F,
    ) -> Result<(), String>
    where
        F: FnMut(&str) -> bool,
    {
        let endpoint = config.endpoint.trim_end_matches('/');
        let url = format!("{}/api/generate", endpoint);

        let req_body = OllamaRequest {
            model: config.model.clone(),
            prompt: user_prompt.to_string(),
            system: system_prompt.to_string(),
            stream: true,
        };

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(8))
            .timeout_read(Duration::from_secs(120))
            .build();

        let resp = agent
            .post(&url)
            .set("Content-Type", "application/json")
            .send_json(&req_body)
            .map_err(|e| format!("Ollama request failed: {}. Is Ollama running at {}?", e, endpoint))?;

        let reader = BufReader::new(resp.into_reader());
        for line_res in reader.lines() {
            if cancel_token.load(Ordering::Relaxed) {
                break;
            }

            let line = line_res.map_err(|e| format!("Failed reading Ollama response: {}", e))?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<OllamaStreamChunk>(&line) {
                Ok(chunk) => {
                    if let Some(err) = chunk.error {
                        return Err(format!("Ollama error: {}", err));
                    }
                    if !chunk.response.is_empty() {
                        if !on_token(&chunk.response) {
                            break;
                        }
                    }
                    if chunk.done {
                        break;
                    }
                }
                Err(e) => {
                    // Try parsing generic error message
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                        if let Some(err) = val.get("error").and_then(|v| v.as_str()) {
                            return Err(format!("Ollama error: {}", err));
                        }
                    }
                    return Err(format!("Failed to parse Ollama chunk ({}): {}", e, line));
                }
            }
        }

        Ok(())
    }

    fn stream_openai<F>(
        config: &AiConfig,
        system_prompt: &str,
        user_prompt: &str,
        cancel_token: Arc<AtomicBool>,
        mut on_token: F,
    ) -> Result<(), String>
    where
        F: FnMut(&str) -> bool,
    {
        let endpoint = config.endpoint.trim_end_matches('/');
        let url = if endpoint.ends_with("/chat/completions") {
            endpoint.to_string()
        } else {
            format!("{}/v1/chat/completions", endpoint)
        };

        let messages = vec![
            OpenAiMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            OpenAiMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ];

        let req_body = OpenAiRequest {
            model: config.model.clone(),
            messages,
            stream: true,
        };

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(8))
            .timeout_read(Duration::from_secs(120))
            .build();

        let mut req = agent.post(&url).set("Content-Type", "application/json");

        if let Some(ref key) = config.api_key {
            if !key.trim().is_empty() {
                req = req.set("Authorization", &format!("Bearer {}", key.trim()));
            }
        }

        let resp = req
            .send_json(&req_body)
            .map_err(|e| format!("Custom API request failed: {}", e))?;

        let reader = BufReader::new(resp.into_reader());
        for line_res in reader.lines() {
            if cancel_token.load(Ordering::Relaxed) {
                break;
            }

            let line = line_res.map_err(|e| format!("Failed reading API response: {}", e))?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(data) = trimmed.strip_prefix("data: ") {
                if data.trim() == "[DONE]" {
                    break;
                }

                if let Ok(chunk) = serde_json::from_str::<OpenAiStreamChunk>(data) {
                    for choice in chunk.choices {
                        if let Some(content) = choice.delta.content {
                            if !content.is_empty() && !on_token(&content) {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn query_gemini<F>(
        config: &AiConfig,
        system_prompt: &str,
        user_prompt: &str,
        cancel_token: Arc<AtomicBool>,
        mut on_token: F,
    ) -> Result<(), String>
    where
        F: FnMut(&str) -> bool,
    {
        let api_key = match config.api_key.as_ref() {
            Some(k) if !k.trim().is_empty() => k.trim(),
            _ => {
                // Check environment variable
                match std::env::var("GEMINI_API_KEY") {
                    Ok(ref val) if !val.trim().is_empty() => "",
                    _ => return Err("Gemini API key is required. Set it in config or GEMINI_API_KEY environment variable.".to_string()),
                }
            }
        };

        let key_str = if api_key.is_empty() {
            std::env::var("GEMINI_API_KEY").unwrap_or_default()
        } else {
            api_key.to_string()
        };

        let model = if config.model.trim().is_empty() {
            "gemini-1.5-flash"
        } else {
            config.model.trim()
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, key_str
        );

        let req_body = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: user_prompt.to_string(),
                }],
            }],
            system_instruction: if system_prompt.is_empty() {
                None
            } else {
                Some(GeminiSystemInstruction {
                    parts: vec![GeminiPart {
                        text: system_prompt.to_string(),
                    }],
                })
            },
        };

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(60))
            .build();

        let resp = agent
            .post(&url)
            .set("Content-Type", "application/json")
            .send_json(&req_body)
            .map_err(|e| format!("Gemini API request failed: {}", e))?;

        if cancel_token.load(Ordering::Relaxed) {
            return Ok(());
        }

        let resp_body: GeminiResponse = resp
            .into_json()
            .map_err(|e| format!("Failed to parse Gemini response: {}", e))?;

        if let Some(err) = resp_body.error {
            return Err(format!("Gemini API error: {}", err.message));
        }

        for candidate in resp_body.candidates {
            for part in candidate.content.parts {
                if !part.text.is_empty() {
                    let _ = on_token(&part.text);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_request_json() {
        let req = OllamaRequest {
            model: "llama3.2".to_string(),
            prompt: "Hello".to_string(),
            system: "You are helpful".to_string(),
            stream: true,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"model\":\"llama3.2\""));
        assert!(json.contains("\"stream\":true"));
    }

    #[test]
    fn test_openai_chunk_parsing() {
        let raw = r#"{"choices":[{"delta":{"content":"Hello world"},"index":0}]}"#;
        let chunk: OpenAiStreamChunk = serde_json::from_str(raw).unwrap();
        assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("Hello world"));
    }
}
