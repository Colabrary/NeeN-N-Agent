use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub neen_api: NeenApiConfig,
    pub voice: VoiceConfig,
    pub screen: ScreenConfig,
    pub notifications: NotificationConfig,
    pub system: SystemConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeenApiConfig {
    pub base_url: String,
    pub access_token: Option<String>,
    pub ai_access_key: Option<String>,
    pub refresh_token: Option<String>,
    pub tunnel_token: Option<String>,
    pub device_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub enabled: bool,
    pub language: String,
    pub voice_id: String,
    pub speed: f32,
    pub wake_word: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenConfig {
    pub capture_enabled: bool,
    pub capture_interval_ms: u64,
    pub analysis_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub auto_reply_enabled: bool,
    pub monitored_apps: Vec<String>,
    pub reply_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub auto_start: bool,
    pub run_in_background: bool,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            neen_api: NeenApiConfig {
                base_url: "https://crmapi.9ance.com/api".to_string(),
                access_token: None,
                ai_access_key: None,
                refresh_token: None,
                tunnel_token: None,
                device_fingerprint: None,
            },
            voice: VoiceConfig {
                enabled: true,
                language: "en-US".to_string(),
                voice_id: "en_US-lessac-medium".to_string(),
                speed: 1.0,
                wake_word: "Hey NeeN".to_string(),
            },
            screen: ScreenConfig {
                capture_enabled: true,
                capture_interval_ms: 5000,
                analysis_enabled: true,
            },
            notifications: NotificationConfig {
                auto_reply_enabled: true,
                monitored_apps: vec![
                    "WhatsApp".to_string(),
                    "Telegram".to_string(),
                    "Slack".to_string(),
                    "Mail".to_string(),
                ],
                reply_delay_ms: 2000,
            },
            system: SystemConfig {
                auto_start: true,
                run_in_background: true,
                log_level: "info".to_string(),
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let config_str = std::fs::read_to_string(&config_path)?;
            let config: AppConfig = toml::from_str(&config_str)?;
            log::info!("Loaded config from: {:?}", config_path);
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            log::info!("Created default config at: {:?}", config_path);
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let config_str = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, config_str)?;
        log::info!("Saved config to: {:?}", config_path);
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        Ok(config_dir.join("neen-desktop-agent").join("config.toml"))
    }
    
    pub fn update_tokens(&mut self, access_token: String, refresh_token: Option<String>) -> Result<()> {
        self.neen_api.access_token = Some(access_token);
        if let Some(refresh_token) = refresh_token {
            self.neen_api.refresh_token = Some(refresh_token);
        }
        self.save()
    }
}
