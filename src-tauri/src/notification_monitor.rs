use anyhow::Result;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{Datelike, Timelike};

#[derive(Debug, Clone)]
pub struct NotificationMonitor {
    monitored_apps: Vec<String>,
    notification_handlers: HashMap<String, NotificationHandler>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub app_name: String,
    pub title: String,
    pub content: String,
    pub sender: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub notification_type: NotificationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Message,
    Email,
    Call,
    Reminder,
    System,
    Other,
}

#[derive(Debug, Clone)]
pub struct NotificationHandler {
    pub app_name: String,
    pub auto_reply_enabled: bool,
    pub reply_template: Option<String>,
}

impl NotificationMonitor {
    pub fn new() -> Self {
        let monitored_apps = vec![
            "WhatsApp".to_string(),
            "Telegram".to_string(),
            "Slack".to_string(),
            "Microsoft Teams".to_string(),
            "Mail".to_string(),
            "Messages".to_string(),
        ];
        
        let mut notification_handlers = HashMap::new();
        
        // Set up default handlers
        notification_handlers.insert("WhatsApp".to_string(), NotificationHandler {
            app_name: "WhatsApp".to_string(),
            auto_reply_enabled: true,
            reply_template: None,
        });
        
        notification_handlers.insert("Telegram".to_string(), NotificationHandler {
            app_name: "Telegram".to_string(),
            auto_reply_enabled: true,
            reply_template: None,
        });
        
        notification_handlers.insert("Mail".to_string(), NotificationHandler {
            app_name: "Mail".to_string(),
            auto_reply_enabled: false, // Email auto-reply should be more careful
            reply_template: Some("Thank you for your email. I'll get back to you soon.".to_string()),
        });
        
        Self {
            monitored_apps,
            notification_handlers,
        }
    }
    
    pub async fn start_monitoring<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(Notification) -> Result<()> + Send + 'static,
    {
        log::info!("Starting notification monitoring for apps: {:?}", self.monitored_apps);
        
        #[cfg(target_os = "macos")]
        {
            self.start_macos_monitoring(callback).await
        }
        
        #[cfg(target_os = "windows")]
        {
            self.start_windows_monitoring(callback).await
        }
        
        #[cfg(target_os = "linux")]
        {
            self.start_linux_monitoring(callback).await
        }
    }
    
    #[cfg(target_os = "macos")]
    async fn start_macos_monitoring<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(Notification) -> Result<()> + Send + 'static,
    {
        log::info!("Starting macOS notification monitoring");
        
        // This would integrate with macOS NSUserNotificationCenter
        // For now, simulate notifications for testing
        tokio::spawn(async move {
            let mut counter = 0;
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                
                counter += 1;
                let notification = Notification {
                    id: format!("test_{}", counter),
                    app_name: "WhatsApp".to_string(),
                    title: "New Message".to_string(),
                    content: format!("Test message #{}", counter),
                    sender: Some("John Doe".to_string()),
                    timestamp: chrono::Utc::now(),
                    notification_type: NotificationType::Message,
                };
                
                if let Err(e) = callback(notification) {
                    log::error!("Notification callback error: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    async fn start_windows_monitoring<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(Notification) -> Result<()> + Send + 'static,
    {
        log::info!("Starting Windows notification monitoring");
        
        // This would integrate with Windows WinRT notifications
        // Placeholder implementation
        Ok(())
    }
    
    #[cfg(target_os = "linux")]
    async fn start_linux_monitoring<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(Notification) -> Result<()> + Send + 'static,
    {
        log::info!("Starting Linux notification monitoring");
        
        // This would integrate with D-Bus notifications
        // Placeholder implementation
        Ok(())
    }
    
    pub fn should_auto_reply(&self, notification: &Notification) -> bool {
        if let Some(handler) = self.notification_handlers.get(&notification.app_name) {
            handler.auto_reply_enabled && self.is_business_hours()
        } else {
            false
        }
    }
    
    pub fn get_reply_template(&self, notification: &Notification) -> Option<String> {
        self.notification_handlers
            .get(&notification.app_name)
            .and_then(|handler| handler.reply_template.clone())
    }
    
    pub fn classify_notification(&self, notification: &Notification) -> NotificationType {
        let content_lower = notification.content.to_lowercase();
        let title_lower = notification.title.to_lowercase();
        
        // Simple classification based on keywords
        if content_lower.contains("call") || title_lower.contains("call") {
            NotificationType::Call
        } else if content_lower.contains("email") || notification.app_name.contains("Mail") {
            NotificationType::Email
        } else if content_lower.contains("reminder") || title_lower.contains("reminder") {
            NotificationType::Reminder
        } else if self.is_messaging_app(&notification.app_name) {
            NotificationType::Message
        } else {
            NotificationType::Other
        }
    }
    
    pub fn extract_lead_info(&self, notification: &Notification) -> Option<LeadInfo> {
        let content = &notification.content;
        
        // Simple lead detection patterns
        let phone_regex = regex::Regex::new(r"\b\d{10,}\b").ok()?;
        let email_regex = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").ok()?;
        
        let phone = phone_regex.find(content).map(|m| m.as_str().to_string());
        let email = email_regex.find(content).map(|m| m.as_str().to_string());
        
        // Check for lead indicators
        let lead_keywords = ["interested", "quote", "price", "buy", "purchase", "inquiry"];
        let is_potential_lead = lead_keywords.iter().any(|&keyword| {
            content.to_lowercase().contains(keyword)
        });
        
        if is_potential_lead || phone.is_some() || email.is_some() {
            Some(LeadInfo {
                name: notification.sender.clone(),
                phone,
                email,
                source: notification.app_name.clone(),
                message: content.clone(),
                confidence: if is_potential_lead { 0.8 } else { 0.5 },
            })
        } else {
            None
        }
    }
    
    pub fn add_monitored_app(&mut self, app_name: String) {
        if !self.monitored_apps.contains(&app_name) {
            self.monitored_apps.push(app_name.clone());
            log::info!("Added {} to monitored apps", app_name);
        }
    }
    
    pub fn remove_monitored_app(&mut self, app_name: &str) {
        self.monitored_apps.retain(|app| app != app_name);
        self.notification_handlers.remove(app_name);
        log::info!("Removed {} from monitored apps", app_name);
    }
    
    pub fn set_auto_reply(&mut self, app_name: String, enabled: bool) {
        if let Some(handler) = self.notification_handlers.get_mut(&app_name) {
            handler.auto_reply_enabled = enabled;
        } else {
            self.notification_handlers.insert(app_name.clone(), NotificationHandler {
                app_name,
                auto_reply_enabled: enabled,
                reply_template: None,
            });
        }
    }
    
    fn is_messaging_app(&self, app_name: &str) -> bool {
        let messaging_apps = ["WhatsApp", "Telegram", "Slack", "Teams", "Messages", "Discord"];
        messaging_apps.iter().any(|&app| app_name.contains(app))
    }
    
    fn is_business_hours(&self) -> bool {
        let now = chrono::Local::now();
        let hour = now.hour();
        let weekday = now.weekday();
        
        // Business hours: 9 AM to 6 PM, Monday to Friday
        use chrono::Weekday;
        matches!(weekday, Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri) && (9..18).contains(&hour)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadInfo {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub source: String,
    pub message: String,
    pub confidence: f32,
}
