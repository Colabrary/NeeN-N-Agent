use rdev::{simulate, Button, EventType, Key};
use anyhow::Result;
use serde_json::Value;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SystemControl {
    // Configuration and state
}

impl SystemControl {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn execute_action(&self, action: &str, params: Value) -> Result<()> {
        log::info!("Executing system action: {} with params: {:?}", action, params);
        
        match action {
            "click" => self.click(params).await,
            "double_click" => self.double_click(params).await,
            "right_click" => self.right_click(params).await,
            "type_text" => self.type_text(params).await,
            "key_press" => self.key_press(params).await,
            "key_combination" => self.key_combination(params).await,
            "scroll" => self.scroll(params).await,
            "drag" => self.drag(params).await,
            "open_application" => self.open_application(params).await,
            "close_application" => self.close_application(params).await,
            "switch_window" => self.switch_window(params).await,
            _ => Err(anyhow::anyhow!("Unknown action: {}", action)),
        }
    }
    
    async fn click(&self, params: Value) -> Result<()> {
        let x = params["x"].as_f64().ok_or_else(|| anyhow::anyhow!("Missing x coordinate"))?;
        let y = params["y"].as_f64().ok_or_else(|| anyhow::anyhow!("Missing y coordinate"))?;
        
        log::info!("Clicking at ({}, {})", x, y);
        
        // Move mouse to position
        self.move_mouse(x, y).await?;
        
        // Small delay for natural movement
        thread::sleep(Duration::from_millis(100));
        
        // Click
        simulate(&EventType::ButtonPress(Button::Left))?;
        thread::sleep(Duration::from_millis(50));
        simulate(&EventType::ButtonRelease(Button::Left))?;
        
        Ok(())
    }
    
    async fn double_click(&self, params: Value) -> Result<()> {
        let x = params["x"].as_f64().ok_or_else(|| anyhow::anyhow!("Missing x coordinate"))?;
        let y = params["y"].as_f64().ok_or_else(|| anyhow::anyhow!("Missing y coordinate"))?;
        
        log::info!("Double clicking at ({}, {})", x, y);
        
        // Move mouse to position
        self.move_mouse(x, y).await?;
        thread::sleep(Duration::from_millis(100));
        
        // First click
        simulate(&EventType::ButtonPress(Button::Left))?;
        thread::sleep(Duration::from_millis(50));
        simulate(&EventType::ButtonRelease(Button::Left))?;
        
        // Small delay between clicks
        thread::sleep(Duration::from_millis(100));
        
        // Second click
        simulate(&EventType::ButtonPress(Button::Left))?;
        thread::sleep(Duration::from_millis(50));
        simulate(&EventType::ButtonRelease(Button::Left))?;
        
        Ok(())
    }
    
    async fn right_click(&self, params: Value) -> Result<()> {
        let x = params["x"].as_f64().ok_or_else(|| anyhow::anyhow!("Missing x coordinate"))?;
        let y = params["y"].as_f64().ok_or_else(|| anyhow::anyhow!("Missing y coordinate"))?;
        
        log::info!("Right clicking at ({}, {})", x, y);
        
        // Move mouse to position
        self.move_mouse(x, y).await?;
        thread::sleep(Duration::from_millis(100));
        
        // Right click
        simulate(&EventType::ButtonPress(Button::Right))?;
        thread::sleep(Duration::from_millis(50));
        simulate(&EventType::ButtonRelease(Button::Right))?;
        
        Ok(())
    }
    
    async fn type_text(&self, params: Value) -> Result<()> {
        let text = params["text"].as_str().ok_or_else(|| anyhow::anyhow!("Missing text parameter"))?;
        let delay_ms = params["delay_ms"].as_u64().unwrap_or(50);
        
        log::info!("Typing text: {}", text);
        
        for ch in text.chars() {
            if let Some(key) = self.char_to_key(ch) {
                simulate(&EventType::KeyPress(key))?;
                thread::sleep(Duration::from_millis(10));
                simulate(&EventType::KeyRelease(key))?;
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
        
        Ok(())
    }
    
    async fn key_press(&self, params: Value) -> Result<()> {
        let key_name = params["key"].as_str().ok_or_else(|| anyhow::anyhow!("Missing key parameter"))?;
        let key = self.string_to_key(key_name)?;
        
        log::info!("Pressing key: {}", key_name);
        
        simulate(&EventType::KeyPress(key))?;
        thread::sleep(Duration::from_millis(50));
        simulate(&EventType::KeyRelease(key))?;
        
        Ok(())
    }
    
    async fn key_combination(&self, params: Value) -> Result<()> {
        let keys = params["keys"].as_array().ok_or_else(|| anyhow::anyhow!("Missing keys array"))?;
        
        log::info!("Pressing key combination: {:?}", keys);
        
        let mut key_objects = Vec::new();
        
        // Press all keys
        for key_value in keys {
            if let Some(key_name) = key_value.as_str() {
                let key = self.string_to_key(key_name)?;
                simulate(&EventType::KeyPress(key))?;
                key_objects.push(key);
                thread::sleep(Duration::from_millis(10));
            }
        }
        
        thread::sleep(Duration::from_millis(100));
        
        // Release all keys in reverse order
        for key in key_objects.iter().rev() {
            simulate(&EventType::KeyRelease(*key))?;
            thread::sleep(Duration::from_millis(10));
        }
        
        Ok(())
    }
    
    async fn scroll(&self, params: Value) -> Result<()> {
        let direction = params["direction"].as_str().unwrap_or("up");
        let amount = params["amount"].as_i64().unwrap_or(3);
        
        log::info!("Scrolling {} by {}", direction, amount);
        
        let delta_y = match direction {
            "up" => amount,
            "down" => -amount,
            _ => return Err(anyhow::anyhow!("Invalid scroll direction: {}", direction)),
        };
        
        for _ in 0..amount.abs() {
            simulate(&EventType::Wheel {
                delta_x: 0,
                delta_y: if delta_y > 0 { 1 } else { -1 },
            })?;
            thread::sleep(Duration::from_millis(50));
        }
        
        Ok(())
    }
    
    async fn drag(&self, params: Value) -> Result<()> {
        let from_x = params["from_x"].as_f64().ok_or_else(|| anyhow::anyhow!("Missing from_x"))?;
        let from_y = params["from_y"].as_f64().ok_or_else(|| anyhow::anyhow!("Missing from_y"))?;
        let to_x = params["to_x"].as_f64().ok_or_else(|| anyhow::anyhow!("Missing to_x"))?;
        let to_y = params["to_y"].as_f64().ok_or_else(|| anyhow::anyhow!("Missing to_y"))?;
        
        log::info!("Dragging from ({}, {}) to ({}, {})", from_x, from_y, to_x, to_y);
        
        // Move to start position
        self.move_mouse(from_x, from_y).await?;
        thread::sleep(Duration::from_millis(100));
        
        // Press mouse button
        simulate(&EventType::ButtonPress(Button::Left))?;
        thread::sleep(Duration::from_millis(50));
        
        // Move to end position (smooth movement)
        let steps = 20;
        let dx = (to_x - from_x) / steps as f64;
        let dy = (to_y - from_y) / steps as f64;
        
        for i in 1..=steps {
            let x = from_x + dx * i as f64;
            let y = from_y + dy * i as f64;
            self.move_mouse(x, y).await?;
            thread::sleep(Duration::from_millis(20));
        }
        
        // Release mouse button
        simulate(&EventType::ButtonRelease(Button::Left))?;
        
        Ok(())
    }
    
    async fn open_application(&self, params: Value) -> Result<()> {
        let app_name = params["app_name"].as_str().ok_or_else(|| anyhow::anyhow!("Missing app_name"))?;
        
        log::info!("Opening application: {}", app_name);
        
        #[cfg(target_os = "macos")]
        {
            let command = format!("open -a '{}'", app_name);
            std::process::Command::new("sh")
                .arg("-c")
                .arg(&command)
                .spawn()?;
        }
        
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(&["/C", "start", app_name])
                .spawn()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new(app_name)
                .spawn()?;
        }
        
        Ok(())
    }
    
    async fn close_application(&self, params: Value) -> Result<()> {
        let app_name = params["app_name"].as_str().ok_or_else(|| anyhow::anyhow!("Missing app_name"))?;
        
        log::info!("Closing application: {}", app_name);
        
        // Use Alt+F4 (Windows/Linux) or Cmd+Q (macOS) to close active window
        #[cfg(target_os = "macos")]
        {
            simulate(&EventType::KeyPress(Key::MetaLeft))?;
            simulate(&EventType::KeyPress(Key::KeyQ))?;
            thread::sleep(Duration::from_millis(50));
            simulate(&EventType::KeyRelease(Key::KeyQ))?;
            simulate(&EventType::KeyRelease(Key::MetaLeft))?;
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            simulate(&EventType::KeyPress(Key::Alt))?;
            simulate(&EventType::KeyPress(Key::F4))?;
            thread::sleep(Duration::from_millis(50));
            simulate(&EventType::KeyRelease(Key::F4))?;
            simulate(&EventType::KeyRelease(Key::Alt))?;
        }
        
        Ok(())
    }
    
    async fn switch_window(&self, params: Value) -> Result<()> {
        log::info!("Switching window");
        
        // Use Alt+Tab (Windows/Linux) or Cmd+Tab (macOS)
        #[cfg(target_os = "macos")]
        {
            simulate(&EventType::KeyPress(Key::MetaLeft))?;
            simulate(&EventType::KeyPress(Key::Tab))?;
            thread::sleep(Duration::from_millis(100));
            simulate(&EventType::KeyRelease(Key::Tab))?;
            simulate(&EventType::KeyRelease(Key::MetaLeft))?;
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            simulate(&EventType::KeyPress(Key::Alt))?;
            simulate(&EventType::KeyPress(Key::Tab))?;
            thread::sleep(Duration::from_millis(100));
            simulate(&EventType::KeyRelease(Key::Tab))?;
            simulate(&EventType::KeyRelease(Key::Alt))?;
        }
        
        Ok(())
    }
    
    async fn move_mouse(&self, x: f64, y: f64) -> Result<()> {
        simulate(&EventType::MouseMove { x, y })?;
        Ok(())
    }
    
    fn char_to_key(&self, ch: char) -> Option<Key> {
        match ch {
            'a'..='z' => Some(Key::KeyA), // This is simplified - would need full mapping
            'A'..='Z' => Some(Key::KeyA), // Would need shift handling
            '0'..='9' => Some(Key::Num0), // Simplified
            ' ' => Some(Key::Space),
            '\n' => Some(Key::Return),
            '\t' => Some(Key::Tab),
            _ => None,
        }
    }
    
    fn string_to_key(&self, key_name: &str) -> Result<Key> {
        match key_name.to_lowercase().as_str() {
            "enter" | "return" => Ok(Key::Return),
            "space" => Ok(Key::Space),
            "tab" => Ok(Key::Tab),
            "escape" | "esc" => Ok(Key::Escape),
            "backspace" => Ok(Key::Backspace),
            "delete" => Ok(Key::Delete),
            "up" => Ok(Key::UpArrow),
            "down" => Ok(Key::DownArrow),
            "left" => Ok(Key::LeftArrow),
            "right" => Ok(Key::RightArrow),
            "f1" => Ok(Key::F1),
            "f2" => Ok(Key::F2),
            "f3" => Ok(Key::F3),
            "f4" => Ok(Key::F4),
            "f5" => Ok(Key::F5),
            "f6" => Ok(Key::F6),
            "f7" => Ok(Key::F7),
            "f8" => Ok(Key::F8),
            "f9" => Ok(Key::F9),
            "f10" => Ok(Key::F10),
            "f11" => Ok(Key::F11),
            "f12" => Ok(Key::F12),
            "ctrl" | "control" => Ok(Key::ControlLeft),
            "alt" => Ok(Key::Alt),
            "shift" => Ok(Key::ShiftLeft),
            "cmd" | "meta" => Ok(Key::MetaLeft),
            _ => Err(anyhow::anyhow!("Unknown key: {}", key_name)),
        }
    }
}
