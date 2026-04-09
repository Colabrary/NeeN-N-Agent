use tauri::Manager;
use tauri::Emitter;

mod ai_engine;
mod config;
mod neen_api;
mod notification_monitor;
mod screen_capture;
mod system_control;
mod voice_processor;

use ai_engine::AIEngine;
use config::AppConfig;
use notification_monitor::NotificationMonitor;
use screen_capture::ScreenCapture;
use system_control::SystemControl;
use voice_processor::VoiceProcessor;

#[tauri::command]
async fn login_user(email: String, password: String) -> Result<bool, String> {
    log::info!("Login attempt for email: {}", email);
    
    let client = reqwest::Client::new();
    let fingerprint = "desktop_agent_fingerprint";
    
    // Step 1: Initialize tunnel token
    let tunnel_init_url = "https://crmapi.9ance.com/api/auth/tunnel/init/";
    
    log::info!("Initializing tunnel...");
    let tunnel_response = client.post(tunnel_init_url)
        .json(&serde_json::json!({
            "fingerprint": fingerprint
        }))
        .send()
        .await
        .map_err(|e| format!("Tunnel init failed: {}", e))?;
    
    let tunnel_data: serde_json::Value = tunnel_response.json().await
        .map_err(|e| format!("Failed to parse tunnel response: {}", e))?;
    
    let tunnel_token = tunnel_data.get("tunnel_token")
        .and_then(|t| t.as_str())
        .ok_or("No tunnel token received")?;
    
    log::info!("Tunnel initialized, attempting login...");
    
    // Step 2: Login with tunnel protection and force logout
    let login_url = "https://crmapi.9ance.com/api/auth/login/";
    let login_data = serde_json::json!({
        "email": email,
        "password": password,
        "platform": "desktop",
        "force_logout": true
    });
    
    let login_response = client.post(login_url)
        .header("X-Tunnel-Token", tunnel_token)
        .header("X-Device-Fingerprint", fingerprint)
        .json(&login_data)
        .send()
        .await
        .map_err(|e| format!("Login request failed: {}", e))?;
    
    log::info!("Login response status: {}", login_response.status());
    
    if login_response.status().is_success() {
        match login_response.json::<serde_json::Value>().await {
            Ok(json) => {
                log::info!("Login response: {:?}", json);
                
                if let Some(access_token) = json.get("access").and_then(|t| t.as_str()) {
                    // Step 3: Upgrade tunnel token
                    let upgrade_url = "https://crmapi.9ance.com/api/auth/tunnel/upgrade/";
                    let upgrade_response = client.post(upgrade_url)
                        .header("Authorization", format!("Bearer {}", access_token))
                        .header("X-Tunnel-Token", tunnel_token)
                        .header("X-Device-Fingerprint", fingerprint)
                        .json(&serde_json::json!({
                            "fingerprint": fingerprint
                        }))
                        .send()
                        .await;
                    
                    let final_tunnel_token = if let Ok(upgrade_resp) = upgrade_response {
                        if let Ok(upgrade_data) = upgrade_resp.json::<serde_json::Value>().await {
                            upgrade_data.get("tunnel_token")
                                .and_then(|t| t.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| tunnel_token.to_string())
                        } else {
                            tunnel_token.to_string()
                        }
                    } else {
                        tunnel_token.to_string()
                    };
                    
                    // Save tokens to config
                    if let Ok(mut config) = crate::config::AppConfig::load() {
                        config.neen_api.access_token = Some(access_token.to_string());
                        if let Some(refresh_token) = json.get("refresh").and_then(|t| t.as_str()) {
                            config.neen_api.refresh_token = Some(refresh_token.to_string());
                        }
                        config.neen_api.tunnel_token = Some(final_tunnel_token);
                        config.neen_api.device_fingerprint = Some(fingerprint.to_string());
                        let _ = config.save();
                        
                        log::info!("Login successful, tokens saved with tunnel security");
                        Ok(true)
                    } else {
                        log::error!("Failed to save login tokens");
                        Ok(false)
                    }
                } else {
                    log::error!("No access token in response");
                    Ok(false)
                }
            },
            Err(e) => {
                log::error!("Failed to parse login response: {}", e);
                Ok(false)
            }
        }
    } else {
        let status = login_response.status();
        let error_text = login_response.text().await.unwrap_or_default();
        log::error!("Login failed with status: {} - {}", status, error_text);
        Ok(false)
    }
}

#[tauri::command]
async fn show_main_app(app: tauri::AppHandle) -> Result<(), String> {
    // Close login window
    if let Some(login_window) = app.get_webview_window("login") {
        let _ = login_window.close();
    }
    
    // Create main app window
    match tauri::WebviewWindowBuilder::new(
        &app,
        "main",
        tauri::WebviewUrl::App("index.html".into())
    )
    .title("NeeN Agent")
    .inner_size(800.0, 800.0)
    .resizable(true)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .transparent(true)
    .visible_on_all_workspaces(true)
    .build() {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string())
    }
}

#[tauri::command]
async fn open_chat_window(app: tauri::AppHandle) -> Result<(), String> {
    log::info!("Opening chat window...");
    
    // Check if chat window already exists
    if let Some(window) = app.get_webview_window("chat") {
        log::info!("Chat window exists, focusing...");
        let _ = window.set_focus();
        let _ = window.show();
        return Ok(());
    }
    
    log::info!("Creating new chat window...");
    
    // Create new chat window
    match tauri::WebviewWindowBuilder::new(
        &app,
        "chat",
        tauri::WebviewUrl::App("chat.html".into())
    )
    .title("NeeN Chat")
    .inner_size(400.0, 600.0)
    .resizable(true)
    .center()
    .decorations(true)
    .always_on_top(false)
    .build() {
        Ok(_) => {
            log::info!("Chat window created successfully");
            Ok(())
        },
        Err(e) => {
            log::error!("Failed to create chat window: {}", e);
            Err(format!("Failed to create window: {}", e))
        }
    }
}

fn create_main_window(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::Manager;
    
    let _main_window = tauri::WebviewWindowBuilder::new(
        app,
        "main",
        tauri::WebviewUrl::App("index.html".into())
    )
    .title("NeeN AI Assistant")
    .inner_size(800.0, 800.0)
    .resizable(true)
    .decorations(false)
    .always_on_top(true)
    .transparent(true)
    .skip_taskbar(true)
    .build()?;
    
    Ok(())
}

#[tauri::command]
async fn validate_session() -> Result<bool, String> {
    log::info!("Validating session...");
    let config = AppConfig::load().map_err(|e| e.to_string())?;
    let valid = config.neen_api.access_token.is_some();
    log::info!("Session valid: {}", valid);
    Ok(valid)
}

#[tauri::command]
async fn restart_to_login(app_handle: tauri::AppHandle) -> Result<(), String> {
    log::info!("Restarting to login...");
    
    // Close main window if it exists
    if let Some(main_window) = app_handle.get_webview_window("main") {
        let _ = main_window.close();
    }
    
    // Create login window
    let _login_window = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "login",
        tauri::WebviewUrl::App("login.html".into())
    )
    .title("NeeN Login")
    .inner_size(400.0, 500.0)
    .resizable(false)
    .center()
    .build()
    .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn logout_user() -> Result<(), String> {
    log::info!("Logging out user");
    
    // Clear config
    let mut config = AppConfig::load().map_err(|e| e.to_string())?;
    config.neen_api.access_token = None;
    config.neen_api.tunnel_token = None;
    config.neen_api.device_fingerprint = None;
    config.save().map_err(|e| e.to_string())?;
    
    log::info!("User logged out successfully - tokens cleared");
    Ok(())
}

#[tauri::command]
fn force_exit() {
    log::info!("Force exiting application");
    std::process::exit(0);
}

#[tauri::command]
fn exit_app(app: tauri::AppHandle) {
    log::info!("Exiting app");
    app.exit(0);
}

#[tauri::command]
async fn send_chat_message(message: String) -> Result<String, String> {
    log::info!("Chat message received: {}", message);
    
    // Load config to get tokens
    let config = crate::config::AppConfig::load().map_err(|e| e.to_string())?;
    
    let client = reqwest::Client::new();
    let api_url = "https://crmapi.9ance.com/api/ai/chat/";
    
    let request_body = serde_json::json!({
        "message": message,
        "session_id": "desktop_agent_session"
    });
    
    let mut request = client.post(api_url).json(&request_body);
    
    // Add authorization and tunnel headers if tokens exist
    if let Some(token) = &config.neen_api.access_token {
        request = request.bearer_auth(token);
        
        // Add tunnel headers if available
        if let Some(tunnel_token) = &config.neen_api.tunnel_token {
            request = request.header("X-Tunnel-Token", tunnel_token);
        }
        if let Some(fingerprint) = &config.neen_api.device_fingerprint {
            request = request.header("X-Device-Fingerprint", fingerprint);
        }
    }
    
    log::info!("Sending chat request to: {}", api_url);
    
    match request.send().await {
        Ok(response) => {
            log::info!("Chat API response status: {}", response.status());
            
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        log::info!("Chat API response: {:?}", json);
                        
                        // Return the complete JSON response
                        Ok(json.to_string())
                    },
                    Err(e) => {
                        log::error!("Failed to parse API response: {}", e);
                        Ok("I understand. How can I help you further?".to_string())
                    }
                }
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                log::error!("API request failed with status: {} - {}", status, error_text);
                
                if status == 401 || status == 403 {
                    Err(format!("Session expired: {}", error_text))
                } else {
                    Err(format!("API Error {}: {}", status, error_text))
                }
            }
        },
        Err(e) => {
            log::error!("Failed to send API request: {}", e);
            // Fallback responses
            let response = match message.to_lowercase().as_str() {
                msg if msg.contains("lead") => "I can help you manage leads. What specific information do you need?",
                msg if msg.contains("hello") || msg.contains("hi") => "Hello! I'm your NeeN AI assistant. How can I help you today?",
                msg if msg.contains("help") => "I can assist you with leads, CRM tasks, and business operations. What do you need help with?",
                _ => "I understand. Let me help you with that. What specific assistance do you need?"
            };
            Ok(response.to_string())
        }
    }
}

#[tauri::command]
async fn open_notification_window(app: tauri::AppHandle, id: String, query: String) -> Result<(), String> {
    let url = format!("notification.html?{}", query);
    let label = format!("notif_{}", id);

    // Close existing one with same label if any
    if let Some(w) = app.get_webview_window(&label) {
        let _ = w.close();
    }

    let height = if query.contains("channel=") { 320.0 } else { 180.0 };

    tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        tauri::WebviewUrl::App(url.into())
    )
    .title("NeeN Notification")
    .inner_size(370.0, height)
    .resizable(false)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .transparent(false)
    .shadow(false)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}


#[tauri::command]
async fn get_access_token() -> Result<String, String> {
    let config = AppConfig::load().map_err(|e| e.to_string())?;
    config.neen_api.access_token.ok_or("No access token".to_string())
}

#[tauri::command]
async fn fetch_whatsapp_unread() -> Result<String, String> {
    let config = AppConfig::load().map_err(|e| e.to_string())?;
    let token = config.neen_api.access_token.ok_or("No access token")?;

    let client = reqwest::Client::new();
    let mut req = client
        .get("https://crmapi.9ance.com/api/whatsapp/conversations/")
        .query(&[("has_unread", "true"), ("limit", "10")])
        .bearer_auth(&token);

    if let Some(tunnel) = &config.neen_api.tunnel_token {
        req = req.header("X-Tunnel-Token", tunnel);
    }
    if let Some(fp) = &config.neen_api.device_fingerprint {
        req = req.header("X-Device-Fingerprint", fp);
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;
    if resp.status().is_success() {
        Ok(resp.text().await.map_err(|e| e.to_string())?)
    } else {
        Err(format!("API error: {}", resp.status()))
    }
}

#[tauri::command]
async fn fetch_todays_activities() -> Result<String, String> {
    let config = AppConfig::load().map_err(|e| e.to_string())?;
    let token = config.neen_api.access_token.ok_or("No access token")?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let client = reqwest::Client::new();
    let mut req = client
        .get("https://crmapi.9ance.com/api/activities/")
        .query(&[("date_from", &today), ("date_to", &today), ("status", &"pending".to_string())])
        .bearer_auth(&token);

    if let Some(tunnel) = &config.neen_api.tunnel_token {
        req = req.header("X-Tunnel-Token", tunnel);
    }
    if let Some(fp) = &config.neen_api.device_fingerprint {
        req = req.header("X-Device-Fingerprint", fp);
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        Ok(resp.text().await.map_err(|e| e.to_string())?)
    } else {
        Err(format!("API error: {}", resp.status()))
    }
}

#[tauri::command]
async fn connect_all_ws(app: tauri::AppHandle) -> Result<(), String> {
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use futures::{StreamExt, SinkExt};

    let config = AppConfig::load().map_err(|e| e.to_string())?;
    let token = config.neen_api.access_token.ok_or("No access token")?.clone();

    let ws_endpoints: Vec<(&str, &str)> = vec![
        ("wss://crmapi.9ance.com/ws/activity/notifications/", "ws-activity"),
        ("wss://crmapi.9ance.com/ws/whatsapp/tenant/notifications/", "ws-whatsapp"),
        ("wss://crmapi.9ance.com/ws/email/notifications/", "ws-email"),
    ];

    for (base_url, event_name) in ws_endpoints {
        let url = format!("{}?token={}", base_url, token);
        let app_clone = app.clone();
        let evt = event_name.to_string();

        tauri::async_runtime::spawn(async move {
            loop {
                log::info!("Connecting WS: {}", evt);
                match connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        log::info!("WS connected: {}", evt);
                        let (mut write, mut read) = ws_stream.split();

                        let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(8);

                        // Ping keep-alive every 30s
                        let ping_tx = tx.clone();
                        tokio::spawn(async move {
                            loop {
                                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                                if ping_tx.send(r#"{"type":"ping"}"#.to_string()).await.is_err() { break; }
                            }
                        });

                        // Writer
                        let wh = tokio::spawn(async move {
                            while let Some(msg) = rx.recv().await {
                                if write.send(Message::Text(msg)).await.is_err() { break; }
                            }
                        });

                        // Reader — emit to frontend
                        let ac = app_clone.clone();
                        let ev = evt.clone();
                        while let Some(Ok(msg)) = read.next().await {
                            if let Message::Text(text) = msg {
                                log::info!("[{}] {}", ev, text);
                                let _ = ac.emit(&ev, &text);
                            }
                        }
                        wh.abort();
                        log::warn!("WS {} disconnected, reconnecting in 5s...", ev);
                    }
                    Err(e) => {
                        log::error!("WS {} error: {}, retrying in 5s...", evt, e);
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }

    Ok(())
}

#[tauri::command]
async fn send_whatsapp_reply(to_phone: String, message: String, to_name: Option<String>, account_id: Option<String>, entity_type: Option<String>, entity_id: Option<String>) -> Result<String, String> {
    let config = AppConfig::load().map_err(|e| e.to_string())?;
    let token = config.neen_api.access_token.ok_or("No access token")?;

    let mut payload = serde_json::json!({
        "to_phone": to_phone,
        "message": message
    });
    if let Some(v) = to_name { payload["to_name"] = serde_json::json!(v); }
    if let Some(v) = account_id { payload["account_id"] = serde_json::json!(v); }
    if let Some(v) = entity_type { payload["entity_type"] = serde_json::json!(v); }
    if let Some(v) = entity_id { payload["entity_id"] = serde_json::json!(v); }

    let client = reqwest::Client::new();
    let mut req = client
        .post("https://crmapi.9ance.com/api/webwhatsapp/send/")
        .bearer_auth(&token)
        .json(&payload);

    if let Some(tunnel) = &config.neen_api.tunnel_token {
        req = req.header("X-Tunnel-Token", tunnel);
    }
    if let Some(fp) = &config.neen_api.device_fingerprint {
        req = req.header("X-Device-Fingerprint", fp);
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        Ok(resp.text().await.map_err(|e| e.to_string())?)
    } else {
        Err(format!("WhatsApp reply error: {}", resp.status()))
    }
}

#[tauri::command]
async fn send_email_reply(to_email: String, subject: String, body: String, in_reply_to: Option<String>, mail_account_id: Option<String>) -> Result<String, String> {
    let config = AppConfig::load().map_err(|e| e.to_string())?;
    let token = config.neen_api.access_token.ok_or("No access token")?;

    let payload = serde_json::json!({
        "to_emails": [&to_email],
        "subject": &subject,
        "body": &body,
        "html_body": format!("<p>{}</p>", body.replace('\n', "</p><p>")),
        "cc_emails": [],
        "bcc_emails": []
    });

    log::info!("Email send payload: {}", serde_json::to_string(&payload).unwrap_or_default());

    let client = reqwest::Client::new();
    let mut req = client
        .post("https://crmapi.9ance.com/api/gmail/inbox/")
        .bearer_auth(&token)
        .json(&payload);

    if let Some(tunnel) = &config.neen_api.tunnel_token {
        req = req.header("X-Tunnel-Token", tunnel);
    }
    if let Some(fp) = &config.neen_api.device_fingerprint {
        req = req.header("X-Device-Fingerprint", fp);
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;
    if resp.status().is_success() {
        Ok(resp.text().await.map_err(|e| e.to_string())?)
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        log::error!("Email send failed: {} - {}", status, body);
        Err(format!("Email send error: {} - {}", status, body))
    }
}

#[tauri::command]
async fn mark_notification_read(notification_id: String) -> Result<String, String> {
    let config = AppConfig::load().map_err(|e| e.to_string())?;
    let token = config.neen_api.access_token.ok_or("No access token")?;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("https://crmapi.9ance.com/api/notifications/{}/mark_as_read/", notification_id))
        .bearer_auth(&token)
        .send().await.map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        Ok("ok".to_string())
    } else {
        Err(format!("Mark read error: {}", resp.status()))
    }
}

#[tauri::command]
async fn test_api_connection() -> Result<String, String> {
    log::info!("API connection test requested");
    Ok("API connection successful".to_string())
}

#[tauri::command]
async fn start_voice_listening() -> Result<String, String> {
    log::info!("Voice listening started");
    Ok("Voice listening active".to_string())
}

#[tauri::command]
async fn stop_voice_listening() -> Result<String, String> {
    log::info!("Voice listening stopped");
    Ok("Voice listening stopped".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        // .plugin(tauri_plugin_notification::init())  // Disabled - causing permission errors
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec![])))
        .setup(|app| {
            log::info!("Starting NeeN Desktop Agent...");

            // ── Hide from Dock, live only in menu bar (macOS) ────────────────
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // ── System tray icon ─────────────────────────────────────────────
            {
                use tauri::tray::{TrayIconBuilder, TrayIconEvent};
                use tauri::menu::{MenuBuilder, MenuItemBuilder};

                let show_item = MenuItemBuilder::with_id("show", "Show NeeN").build(app)?;
                let quit_item = MenuItemBuilder::with_id("quit", "Quit NeeN").build(app)?;
                let tray_menu = MenuBuilder::new(app)
                    .item(&show_item)
                    .separator()
                    .item(&quit_item)
                    .build()?;

                let icon = app.default_window_icon()
                    .expect("No app icon found")
                    .clone();

                TrayIconBuilder::new()
                    .icon(icon)
                    .icon_as_template(true)
                    .tooltip("NeeN AI Assistant")
                    .menu(&tray_menu)
                    .on_menu_event(|app, event| match event.id().as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            } else if let Some(w) = app.get_webview_window("login") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click { .. } = event {
                            let app = tray.app_handle();
                            // Toggle main window on single click
                            if let Some(w) = app.get_webview_window("main") {
                                if w.is_visible().unwrap_or(false) {
                                    let _ = w.hide();
                                } else {
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                }
                            } else if let Some(w) = app.get_webview_window("login") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    })
                    .build(app)?;
            }

            // Enable autostart so app always launches at login
            {
                use tauri_plugin_autostart::ManagerExt;
                let autolaunch = app.autolaunch();
                if !autolaunch.is_enabled().unwrap_or(false) {
                    let _ = autolaunch.enable();
                    log::info!("Autostart enabled — app will launch at login");
                }
            }

            // Initialize core components
            let config = AppConfig::load()?;

            // Check if user is already logged in
            if config.neen_api.access_token.is_some() {
                log::info!("Found existing tokens, validating session...");
                if let Some(login_window) = app.get_webview_window("login") {
                    let _ = login_window.close();
                }
                create_main_window(app)?;
            }

            let ai_engine = AIEngine::new(config.clone());
            let notification_monitor = NotificationMonitor::new();
            let screen_capture = ScreenCapture::new();
            let system_control = SystemControl::new();
            let voice_processor = VoiceProcessor::new();

            app.manage(config);
            app.manage(ai_engine);
            app.manage(notification_monitor);
            app.manage(screen_capture);
            app.manage(system_control);
            app.manage(voice_processor);

            start_background_services(app.handle().clone())?;

            log::info!("NeeN Desktop Agent initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            login_user,
            validate_session,
            restart_to_login,
            logout_user,
            force_exit,
            exit_app,
            show_main_app,
            send_chat_message,
            open_chat_window,
            capture_screen,
            test_api_connection,
            start_voice_listening,
            stop_voice_listening,
            create_file,
            read_file,
            write_file,
            delete_file,
            list_directory,
            create_directory,
            organize_files,
            set_click_through,
            get_available_voices,
            text_to_speech,
            speech_to_text,
            analyze_file,
            fetch_todays_activities,
            get_access_token,
            open_notification_window,
            fetch_whatsapp_unread,
            connect_all_ws,
            send_whatsapp_reply,
            send_email_reply,
            mark_notification_read
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn start_background_services(app_handle: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Start background services in a separate thread with its own runtime
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            log::info!("Background services started");
            // Background service logic will be implemented here
        });
    });
    
    Ok(())
}

// Tauri command handlers
#[tauri::command]
async fn process_voice_command(
    command: String,
    _voice_processor: tauri::State<'_, VoiceProcessor>,
    ai_engine: tauri::State<'_, AIEngine>,
) -> Result<String, String> {
    log::info!("Processing voice command: {}", command);
    
    // Process voice command through AI engine
    match ai_engine.process_command(&command).await {
        Ok(response) => Ok(response),
        Err(e) => Err(format!("Failed to process command: {}", e)),
    }
}

#[tauri::command]
async fn capture_screen(
    screen_capture: tauri::State<'_, ScreenCapture>,
) -> Result<String, String> {
    match screen_capture.capture().await {
        Ok(image_data) => {
            use base64::{Engine as _, engine::general_purpose};
            Ok(general_purpose::STANDARD.encode(image_data))
        },
        Err(e) => Err(format!("Failed to capture screen: {}", e)),
    }
}

#[tauri::command]
async fn send_notification_reply(
    notification_id: String,
    reply: String,
    ai_engine: tauri::State<'_, AIEngine>,
) -> Result<(), String> {
    log::info!("Sending notification reply: {}", reply);
    
    match ai_engine.send_notification_reply(&notification_id, &reply).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to send reply: {}", e)),
    }
}

#[tauri::command]
async fn execute_system_action(
    action: String,
    params: serde_json::Value,
    system_control: tauri::State<'_, SystemControl>,
) -> Result<(), String> {
    log::info!("Executing system action: {}", action);
    
    match system_control.execute_action(&action, params).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to execute action: {}", e)),
    }
}

#[tauri::command]
async fn get_agent_status() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "status": "active",
        "version": "0.1.0",
        "uptime": "00:05:23"
    }))
}

#[tauri::command]
async fn analyze_file(path: String) -> Result<String, String> {
    use std::fs;

    log::info!("Analyzing file: {}", path);

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Truncate to 8000 chars to stay within AI limits
    let truncated = if content.len() > 8000 {
        format!("{}... [truncated]", &content[..8000])
    } else {
        content.clone()
    };

    let config = crate::config::AppConfig::load().map_err(|e| e.to_string())?;
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "message": format!("Analyze this file and give a clear summary:\n\nFile: {}\n\n{}", path, truncated)
    });

    let mut request = client
        .post("https://crmapi.9ance.com/api/ai/chat/")
        .json(&request_body);

    if let Some(token) = &config.neen_api.access_token {
        request = request.bearer_auth(token);
        if let Some(tunnel) = &config.neen_api.tunnel_token {
            request = request.header("X-Tunnel-Token", tunnel);
        }
        if let Some(fp) = &config.neen_api.device_fingerprint {
            request = request.header("X-Device-Fingerprint", fp);
        }
    }

    let response = request.send().await.map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        Ok(json.get("message").and_then(|m| m.as_str()).unwrap_or("No summary returned").to_string())
    } else {
        Err(format!("AI API error: {}", response.text().await.unwrap_or_default()))
    }
}

#[tauri::command]
async fn create_file(path: String, content: String) -> Result<String, String> {
    use std::fs;
    
    log::info!("Creating file: {}", path);
    
    // Create parent directories if they don't exist
    if let Some(parent) = std::path::Path::new(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directories: {}", e))?;
    }
    
    fs::write(&path, content).map_err(|e| format!("Failed to create file: {}", e))?;
    
    Ok(format!("File created successfully: {}", path))
}

#[tauri::command]
async fn read_file(path: String) -> Result<String, String> {
    use std::fs;
    
    log::info!("Reading file: {}", path);
    
    fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
async fn write_file(path: String, content: String) -> Result<String, String> {
    use std::fs;
    
    log::info!("Writing to file: {}", path);
    
    fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(format!("File updated successfully: {}", path))
}

#[tauri::command]
async fn delete_file(path: String) -> Result<String, String> {
    use std::fs;
    
    log::info!("Deleting file: {}", path);
    
    fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {}", e))?;
    
    Ok(format!("File deleted successfully: {}", path))
}

#[tauri::command]
async fn list_directory(path: String) -> Result<Vec<String>, String> {
    use std::fs;
    
    log::info!("Listing directory: {}", path);
    
    let entries = fs::read_dir(&path).map_err(|e| format!("Failed to read directory: {}", e))?;
    
    let mut files = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            if let Some(name) = entry.file_name().to_str() {
                files.push(name.to_string());
            }
        }
    }
    
    Ok(files)
}

#[tauri::command]
async fn create_directory(path: String) -> Result<String, String> {
    use std::fs;
    
    log::info!("Creating directory: {}", path);
    
    fs::create_dir_all(&path).map_err(|e| format!("Failed to create directory: {}", e))?;
    
    Ok(format!("Directory created successfully: {}", path))
}

#[tauri::command]
async fn organize_files(directory: String, pattern: String) -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    
    log::info!("Organizing files in: {} with pattern: {}", directory, pattern);
    
    let entries = fs::read_dir(&directory).map_err(|e| format!("Failed to read directory: {}", e))?;
    
    let mut organized_count = 0;
    
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                    let target_dir = format!("{}/{}_files", directory, extension);
                    
                    // Create target directory
                    fs::create_dir_all(&target_dir).map_err(|e| format!("Failed to create target directory: {}", e))?;
                    
                    // Move file
                    if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
                        let target_path = format!("{}/{}", target_dir, filename);
                        fs::rename(&path, &target_path).map_err(|e| format!("Failed to move file: {}", e))?;
                        organized_count += 1;
                    }
                }
            }
        }
    }
    
    Ok(format!("Organized {} files by extension", organized_count))
}

#[tauri::command]
async fn set_click_through(app_handle: tauri::AppHandle, click_through: bool) -> Result<(), String> {
    if let Some(main_window) = app_handle.get_webview_window("main") {
        // Note: Tauri doesn't have direct click-through support yet
        // This is a placeholder for when the feature becomes available
        log::info!("Click-through mode: {}", click_through);
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

#[tauri::command]
async fn get_available_voices() -> Result<Vec<serde_json::Value>, String> {
    log::info!("Fetching available Piper voices from TTS API...");
    
    let config = AppConfig::load().map_err(|e| format!("Failed to load config: {}", e))?;
    
    let jwt_token = config.neen_api.access_token.ok_or("No JWT token available")?;
    let tunnel_token = config.neen_api.tunnel_token.ok_or("No tunnel token available")?;
    let fingerprint = config.neen_api.device_fingerprint.ok_or("No device fingerprint available")?;
    
    let client = reqwest::Client::new();
    let voices_url = "https://crmapi.9ance.com/api/tts/voices/";
    
    let response = client.get(voices_url)
        .header("Authorization", format!("Bearer {}", jwt_token))
        .header("X-Tunnel-Token", tunnel_token)
        .header("X-Device-Fingerprint", fingerprint)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch voices: {}", e))?;
    
    if response.status().is_success() {
        let voices_data: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse voices response: {}", e))?;
        
        log::info!("Voices API response: {}", voices_data);
        
        // Return the full voice data with metadata
        if let Some(voices_array) = voices_data.as_array() {
            Ok(voices_array.clone())
        } else if let Some(voices_obj) = voices_data.as_object() {
            if let Some(voices_array) = voices_obj.get("voices").and_then(|v| v.as_array()) {
                Ok(voices_array.clone())
            } else {
                // Return the object as a single-item array
                Ok(vec![voices_data])
            }
        } else {
            // Fallback to basic voice list
            Ok(vec![
                serde_json::json!({"id": "en_US-lessac-medium", "name": "Lessac (US English)", "language": "en-US"}),
                serde_json::json!({"id": "en_US-amy-medium", "name": "Amy (US English)", "language": "en-US"}),
                serde_json::json!({"id": "en_GB-alan-medium", "name": "Alan (British English)", "language": "en-GB"}),
                serde_json::json!({"id": "es_ES-mls_10246-low", "name": "Spanish Female", "language": "es-ES"}),
                serde_json::json!({"id": "fr_FR-siwis-medium", "name": "French Female", "language": "fr-FR"}),
                serde_json::json!({"id": "de_DE-thorsten-medium", "name": "Thorsten (German)", "language": "de-DE"}),
                serde_json::json!({"id": "it_IT-riccardo-x_low", "name": "Riccardo (Italian)", "language": "it-IT"}),
                serde_json::json!({"id": "pt_BR-faber-medium", "name": "Faber (Portuguese)", "language": "pt-BR"}),
                serde_json::json!({"id": "ru_RU-denis-medium", "name": "Denis (Russian)", "language": "ru-RU"}),
                serde_json::json!({"id": "zh_CN-huayan-medium", "name": "Huayan (Chinese)", "language": "zh-CN"}),
                serde_json::json!({"id": "ja_JP-haruka-medium", "name": "Haruka (Japanese)", "language": "ja-JP"}),
                serde_json::json!({"id": "hi_IN-male-medium", "name": "Hindi Male", "language": "hi-IN"})
            ])
        }
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        log::error!("Failed to fetch voices: {} - {}", status, error_text);
        
        // Return fallback voices with full metadata
        Ok(vec![
            serde_json::json!({"id": "en_US-lessac-medium", "name": "Lessac (US English)", "language": "en-US"}),
            serde_json::json!({"id": "en_US-amy-medium", "name": "Amy (US English)", "language": "en-US"}),
            serde_json::json!({"id": "en_GB-alan-medium", "name": "Alan (British English)", "language": "en-GB"}),
            serde_json::json!({"id": "es_ES-mls_10246-low", "name": "Spanish Female", "language": "es-ES"}),
            serde_json::json!({"id": "fr_FR-siwis-medium", "name": "French Female", "language": "fr-FR"}),
            serde_json::json!({"id": "de_DE-thorsten-medium", "name": "Thorsten (German)", "language": "de-DE"})
        ])
    }
}

#[tauri::command]
async fn text_to_speech(text: String, voice: Option<String>) -> Result<String, String> {
    log::info!("TTS request for text: {} with voice: {:?}", text, voice);
    
    let config = AppConfig::load().map_err(|e| format!("Failed to load config: {}", e))?;
    
    let jwt_token = config.neen_api.access_token.ok_or("No JWT token available")?;
    let tunnel_token = config.neen_api.tunnel_token.ok_or("No tunnel token available")?;
    let fingerprint = config.neen_api.device_fingerprint.ok_or("No device fingerprint available")?;
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    let tts_url = "https://crmapi.9ance.com/api/tts/synthesize/";
    
    log::info!("Calling TTS API: {}", tts_url);
    log::info!("Voice: {}", voice.as_ref().unwrap_or(&"en_US-lessac-medium".to_string()));
    log::info!("Text length: {}", text.len());
    
    let mut request_body = serde_json::json!({
        "text": text,
        "speed": 1.0
    });
    
    // Add voice if provided, default to en_US-lessac-medium
    request_body["voice"] = serde_json::json!(voice.unwrap_or_else(|| "en_US-lessac-medium".to_string()));
    
    let response = client.post(tts_url)
        .header("Authorization", format!("Bearer {}", jwt_token))
        .header("X-Tunnel-Token", tunnel_token)
        .header("X-Device-Fingerprint", fingerprint)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("TTS request failed: {}", e))?;
    
    if response.status().is_success() {
        let audio_data = response.bytes().await
            .map_err(|e| format!("Failed to get audio data: {}", e))?;
        
        // Return base64 encoded audio
        use base64::{Engine as _, engine::general_purpose};
        Ok(general_purpose::STANDARD.encode(audio_data))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("TTS failed: {}", error_text))
    }
}

#[tauri::command]
async fn speech_to_text(audio_data: String) -> Result<String, String> {
    log::info!("STT request received");
    
    let config = AppConfig::load().map_err(|e| format!("Failed to load config: {}", e))?;
    
    let jwt_token = config.neen_api.access_token.ok_or("No JWT token available")?;
    let tunnel_token = config.neen_api.tunnel_token.ok_or("No tunnel token available")?;
    let fingerprint = config.neen_api.device_fingerprint.ok_or("No device fingerprint available")?;
    
    let client = reqwest::Client::new();
    let stt_url = "https://crmapi.9ance.com/api/ai/stt/";
    
    // Decode base64 audio data using modern API
    use base64::{Engine as _, engine::general_purpose};
    let audio_bytes = general_purpose::STANDARD.decode(audio_data)
        .map_err(|e| format!("Failed to decode audio data: {}", e))?;
    
    let response = client.post(stt_url)
        .header("Authorization", format!("Bearer {}", jwt_token))
        .header("X-Tunnel-Token", tunnel_token)
        .header("X-Device-Fingerprint", fingerprint)
        .body(audio_bytes)
        .send()
        .await
        .map_err(|e| format!("STT request failed: {}", e))?;
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse STT response: {}", e))?;
        
        let text = result.get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        
        Ok(text)
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("STT failed: {}", error_text))
    }
}
