use screenshots::Screen;
use image::{ImageBuffer, RgbaImage, ExtendedColorType};
use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct ScreenCapture {
    last_capture: Option<Instant>,
    capture_interval: Duration,
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self {
            last_capture: None,
            capture_interval: Duration::from_millis(5000), // 5 seconds default
        }
    }
    
    pub async fn capture(&self) -> Result<Vec<u8>> {
        log::info!("Capturing screen");
        
        let screens = Screen::all();
        let primary_screen = screens.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No screens found"))?;
        
        let image = primary_screen.capture()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture screen"))?;
        
        // Convert to PNG bytes
        let mut png_data = Vec::new();
        {
            use image::codecs::png::PngEncoder;
            use image::ImageEncoder;
            
            let encoder = PngEncoder::new(&mut png_data);
            encoder.write_image(
                &image.buffer(),
                image.width(),
                image.height(),
                ExtendedColorType::Rgba8,
            )?;
        }
        
        log::info!("Screen captured: {}x{} pixels, {} bytes", 
                  image.width(), image.height(), png_data.len());
        
        Ok(png_data)
    }
    
    pub async fn capture_region(&self, x: i32, y: i32, width: u32, height: u32) -> Result<Vec<u8>> {
        log::info!("Capturing screen region: {}x{} at ({}, {})", width, height, x, y);
        
        let screens = Screen::all();
        let primary_screen = screens.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No screens found"))?;
        
        let full_image = primary_screen.capture()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture screen"))?;
        
        // Crop the image to the specified region
        let cropped = self.crop_image(&full_image, x, y, width, height)?;
        
        // Convert to PNG bytes
        let mut png_data = Vec::new();
        {
            use image::codecs::png::PngEncoder;
            use image::ImageEncoder;
            
            let encoder = PngEncoder::new(&mut png_data);
            encoder.write_image(
                &cropped.as_raw(),
                cropped.width(),
                cropped.height(),
                ExtendedColorType::Rgba8,
            )?;
        }
        
        Ok(png_data)
    }
    
    pub async fn start_continuous_capture<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(Vec<u8>) -> Result<()> + Send + 'static,
    {
        log::info!("Starting continuous screen capture");
        
        loop {
            let now = Instant::now();
            
            // Check if enough time has passed since last capture
            if let Some(last) = self.last_capture {
                if now.duration_since(last) < self.capture_interval {
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }
            }
            
            match self.capture().await {
                Ok(image_data) => {
                    self.last_capture = Some(now);
                    
                    if let Err(e) = callback(image_data) {
                        log::error!("Screen capture callback error: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Screen capture error: {}", e);
                    sleep(Duration::from_secs(1)).await;
                }
            }
            
            sleep(Duration::from_millis(100)).await;
        }
    }
    
    pub fn set_capture_interval(&mut self, interval_ms: u64) {
        self.capture_interval = Duration::from_millis(interval_ms);
        log::info!("Screen capture interval set to {}ms", interval_ms);
    }
    
    pub async fn detect_changes(&self, previous_image: &[u8], current_image: &[u8]) -> Result<bool> {
        // Simple change detection by comparing image sizes and a sample of pixels
        if previous_image.len() != current_image.len() {
            return Ok(true);
        }
        
        // Sample every 1000th byte for quick comparison
        let sample_size = previous_image.len().min(10000);
        let step = if previous_image.len() > sample_size {
            previous_image.len() / sample_size
        } else {
            1
        };
        
        let mut differences = 0;
        for i in (0..previous_image.len()).step_by(step) {
            if previous_image[i] != current_image[i] {
                differences += 1;
                if differences > 10 { // Threshold for significant change
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    pub async fn find_element_by_text(&self, text: &str) -> Result<Option<(i32, i32, u32, u32)>> {
        log::info!("Searching for text on screen: {}", text);
        
        // This would require OCR integration
        // For now, return None as placeholder
        log::warn!("Text search not implemented yet");
        Ok(None)
    }
    
    pub async fn find_element_by_image(&self, template: &[u8]) -> Result<Option<(i32, i32, u32, u32)>> {
        log::info!("Searching for image template on screen");
        
        // This would require image matching algorithms
        // For now, return None as placeholder
        log::warn!("Image template matching not implemented yet");
        Ok(None)
    }
    
    fn crop_image(&self, image: &screenshots::Image, x: i32, y: i32, width: u32, height: u32) -> Result<RgbaImage> {
        let img_width = image.width();
        let img_height = image.height();
        
        // Ensure coordinates are within bounds
        let x = x.max(0) as u32;
        let y = y.max(0) as u32;
        let width = width.min(img_width.saturating_sub(x));
        let height = height.min(img_height.saturating_sub(y));
        
        if width == 0 || height == 0 {
            return Err(anyhow::anyhow!("Invalid crop dimensions"));
        }
        
        let mut cropped = ImageBuffer::new(width, height);
        
        for (crop_x, crop_y, pixel) in cropped.enumerate_pixels_mut() {
            let src_x = x + crop_x;
            let src_y = y + crop_y;
            
            if src_x < img_width && src_y < img_height {
                let src_index = ((src_y * img_width + src_x) * 4) as usize;
                let buffer = image.buffer();
                
                if src_index + 3 < buffer.len() {
                    *pixel = image::Rgba([
                        buffer[src_index],
                        buffer[src_index + 1],
                        buffer[src_index + 2],
                        buffer[src_index + 3],
                    ]);
                }
            }
        }
        
        Ok(cropped)
    }
    
    pub fn get_screen_info(&self) -> Result<Vec<ScreenInfo>> {
        let screens = Screen::all();
        let mut screen_info = Vec::new();
        
        for (index, screen) in screens.iter().enumerate() {
            screen_info.push(ScreenInfo {
                index,
                width: screen.width,
                height: screen.height,
                x: screen.x,
                y: screen.y,
                is_primary: index == 0, // Assume first screen is primary
            });
        }
        
        Ok(screen_info)
    }
}

#[derive(Debug, Clone)]
pub struct ScreenInfo {
    pub index: usize,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
}
