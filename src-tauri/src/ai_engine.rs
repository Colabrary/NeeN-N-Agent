use crate::config::AppConfig;
use crate::neen_api::{NeenApiClient, ChatRequest, VoiceRequest, TtsRequest};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AIEngine {
    api_client: NeenApiClient,
    config: AppConfig,
    active_conversations: HashMap<String, String>, // session_id -> conversation_id
}

impl AIEngine {
    pub fn new(config: AppConfig) -> Self {
        let api_client = NeenApiClient::new(config.neen_api.clone());
        
        Self {
            api_client,
            config,
            active_conversations: HashMap::new(),
        }
    }
    
    pub async fn process_command(&self, command: &str) -> Result<String> {
        log::info!("Processing AI command: {}", command);
        
        let request = ChatRequest {
            message: command.to_string(),
            conversation_id: None,
            model: Some("openai/gpt-4o-mini".to_string()),
        };
        
        match self.api_client.chat(request).await {
            Ok(response) => {
                log::info!("AI Response: {}", response.message);
                
                // Handle any actions returned by the AI
                if let Some(action) = &response.action {
                    self.handle_ai_action(action, &response.action_result).await?;
                }
                
                Ok(response.message)
            }
            Err(e) => {
                log::error!("Failed to process command: {}", e);
                
                // Fallback to public API if authenticated API fails
                match self.api_client.public_chat(command.to_string(), None).await {
                    Ok(response) => Ok(response.message),
                    Err(e2) => Err(anyhow::anyhow!("Both APIs failed: {} | {}", e, e2))
                }
            }
        }
    }
    
    pub async fn process_voice_command(&self, audio_data: Vec<u8>) -> Result<(String, Vec<u8>)> {
        log::info!("Processing voice command");
        
        use base64::{Engine as _, engine::general_purpose};
        let audio_base64 = general_purpose::STANDARD.encode(&audio_data);
        
        let request = VoiceRequest {
            text: None,
            audio: Some(audio_base64),
            encoding: Some("LINEAR16".to_string()),
            sample_rate: Some(16000),
            language: Some(self.config.voice.language.clone()),
            voice: Some(self.config.voice.voice_id.clone()),
            session_id: None,
            return_json: Some(true),
        };
        
        let response = self.api_client.process_voice(request).await?;
        
        let response_text = response.response_text.clone();
        let audio_response = if let Some(audio_b64) = response.audio_base64 {
            general_purpose::STANDARD.decode(audio_b64)?
        } else {
            // Generate TTS if no audio returned
            self.text_to_speech(&response_text).await?
        };
        
        Ok((response_text, audio_response))
    }
    
    pub async fn text_to_speech(&self, text: &str) -> Result<Vec<u8>> {
        log::info!("Converting text to speech: {}", text);
        
        let request = TtsRequest {
            text: text.to_string(),
            voice: Some(self.config.voice.voice_id.clone()),
            speed: Some(self.config.voice.speed),
        };
        
        self.api_client.synthesize_speech(request).await
    }
    
    pub async fn analyze_screen_context(&self, screen_data: &[u8]) -> Result<String> {
        log::info!("Analyzing screen context");
        
        // Convert screen data to base64 for API
        use base64::{Engine as _, engine::general_purpose};
        let screen_b64 = general_purpose::STANDARD.encode(screen_data);
        
        let context_prompt = format!(
            "I'm looking at this screen. Please analyze what's visible and suggest relevant actions. Screen data: {}",
            &screen_b64[..100] // Truncate for logging
        );
        
        self.process_command(&context_prompt).await
    }
    
    pub async fn generate_notification_reply(&self, notification_content: &str, sender: &str) -> Result<String> {
        log::info!("Generating notification reply for: {}", sender);
        
        let prompt = format!(
            "Generate an appropriate reply to this notification from {}. Content: '{}'. \
            Keep it professional and helpful. If it's a lead inquiry, capture the lead information.",
            sender, notification_content
        );
        
        self.process_command(&prompt).await
    }
    
    pub async fn send_notification_reply(&self, notification_id: &str, reply: &str) -> Result<()> {
        log::info!("Sending notification reply: {}", reply);
        
        // This would integrate with the actual notification system
        // For now, we'll log the action
        log::info!("Reply sent to notification {}: {}", notification_id, reply);
        
        Ok(())
    }
    
    async fn handle_ai_action(&self, action: &crate::neen_api::ActionResponse, result: &Option<serde_json::Value>) -> Result<()> {
        log::info!("Handling AI action: {}", action.action_type);
        
        match action.action_type.as_str() {
            "create_lead" => {
                log::info!("Lead created: {:?}", result);
                // Could trigger system notification or UI update
            }
            "list_leads" => {
                log::info!("Leads listed: {:?}", result);
                // Could display in overlay UI
            }
            "send_whatsapp" => {
                log::info!("WhatsApp message sent: {:?}", result);
                // Could show confirmation
            }
            "create_activity" => {
                log::info!("Activity created: {:?}", result);
                // Could add to calendar
            }
            _ => {
                log::info!("Unknown action type: {}", action.action_type);
            }
        }
        
        Ok(())
    }
    
    pub async fn execute_system_command(&self, command: &str) -> Result<String> {
        log::info!("Executing system command: {}", command);
        
        // Enhanced command processing with system integration
        let enhanced_prompt = format!(
            "Execute this system command: '{}'. \
            If this involves file operations, CRM actions, or application control, \
            provide specific instructions for the system to execute.",
            command
        );
        
        self.process_command(&enhanced_prompt).await
    }
    
    pub fn get_conversation_id(&self, session_id: &str) -> Option<String> {
        self.active_conversations.get(session_id).cloned()
    }
    
    pub fn set_conversation_id(&mut self, session_id: String, conversation_id: String) {
        self.active_conversations.insert(session_id, conversation_id);
    }
}
