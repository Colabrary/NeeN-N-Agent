use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::config::NeenApiConfig;

#[derive(Debug, Clone)]
pub struct NeenApiClient {
    client: Client,
    config: NeenApiConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub conversation_id: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub conversation_id: String,
    pub message: String,
    pub action: Option<ActionResponse>,
    pub action_result: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionResponse {
    #[serde(rename = "type")]
    pub action_type: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoiceRequest {
    pub text: Option<String>,
    pub audio: Option<String>,
    pub encoding: Option<String>,
    pub sample_rate: Option<u32>,
    pub language: Option<String>,
    pub voice: Option<String>,
    pub session_id: Option<String>,
    pub return_json: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoiceResponse {
    pub success: bool,
    pub transcript: Option<String>,
    pub stt_confidence: Option<f32>,
    pub response_text: String,
    pub audio_base64: Option<String>,
    pub session_id: String,
    pub voice: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    pub voice: Option<String>,
    pub speed: Option<f32>,
}

impl NeenApiClient {
    pub fn new(config: NeenApiConfig) -> Self {
        let client = Client::new();
        Self { client, config }
    }
    
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/ai/chat/", self.config.base_url);
        
        let mut req_builder = self.client.post(&url)
            .json(&request);
            
        if let Some(token) = &self.config.access_token {
            req_builder = req_builder.bearer_auth(token);
        }
        
        let response = req_builder.send().await?;
        
        if response.status().is_success() {
            let chat_response: ChatResponse = response.json().await?;
            Ok(chat_response)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Chat API error: {}", error_text))
        }
    }
    
    pub async fn public_chat(&self, message: String, session_id: Option<String>) -> Result<ChatResponse> {
        let url = format!("{}/ai/public/chat/", self.config.base_url);
        
        let request = serde_json::json!({
            "message": message,
            "session_id": session_id
        });
        
        let mut req_builder = self.client.post(&url)
            .json(&request);
            
        if let Some(key) = &self.config.ai_access_key {
            req_builder = req_builder.header("X-AI-Access-Key", key);
        }
        
        let response = req_builder.send().await?;
        
        if response.status().is_success() {
            let chat_response: ChatResponse = response.json().await?;
            Ok(chat_response)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Public chat API error: {}", error_text))
        }
    }
    
    pub async fn process_voice(&self, request: VoiceRequest) -> Result<VoiceResponse> {
        let url = format!("{}/voice-agent/process/", self.config.base_url);
        
        let mut req_builder = self.client.post(&url)
            .json(&request);
            
        if let Some(key) = &self.config.ai_access_key {
            req_builder = req_builder.header("X-AI-Access-Key", key);
        }
        
        let response = req_builder.send().await?;
        
        if response.status().is_success() {
            let voice_response: VoiceResponse = response.json().await?;
            Ok(voice_response)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Voice API error: {}", error_text))
        }
    }
    
    pub async fn synthesize_speech(&self, request: TtsRequest) -> Result<Vec<u8>> {
        let url = if self.config.access_token.is_some() {
            format!("{}/tts/synthesize/", self.config.base_url)
        } else {
            format!("{}/tts/public/synthesize/", self.config.base_url)
        };
        
        let mut req_builder = self.client.post(&url)
            .json(&request);
            
        if let Some(token) = &self.config.access_token {
            req_builder = req_builder.bearer_auth(token);
        } else if let Some(key) = &self.config.ai_access_key {
            req_builder = req_builder.header("X-AI-Access-Key", key);
        }
        
        let response = req_builder.send().await?;
        
        if response.status().is_success() {
            let audio_data = response.bytes().await?;
            Ok(audio_data.to_vec())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("TTS API error: {}", error_text))
        }
    }
    
    pub async fn refresh_token(&mut self) -> Result<()> {
        if let Some(refresh_token) = &self.config.refresh_token {
            let url = format!("{}/auth/token/refresh/", self.config.base_url);
            
            let request = serde_json::json!({
                "refresh": refresh_token
            });
            
            let response = self.client.post(&url)
                .json(&request)
                .send()
                .await?;
                
            if response.status().is_success() {
                let token_response: serde_json::Value = response.json().await?;
                if let Some(access_token) = token_response.get("access").and_then(|v| v.as_str()) {
                    self.config.access_token = Some(access_token.to_string());
                    log::info!("Successfully refreshed access token");
                }
                Ok(())
            } else {
                let error_text = response.text().await?;
                Err(anyhow::anyhow!("Token refresh error: {}", error_text))
            }
        } else {
            Err(anyhow::anyhow!("No refresh token available"))
        }
    }
}
