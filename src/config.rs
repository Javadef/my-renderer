// =============================================================================
// CONFIGURATION - Load settings from config.toml
// =============================================================================
//
// This module handles loading and parsing configuration from config.toml.
// Provides sensible defaults if config file is missing or has errors.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// Root configuration structure
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub window: WindowConfig,
    pub graphics: GraphicsConfig,
    pub debug: DebugConfig,
    pub controls: ControlsConfig,
}

/// Window settings
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Vulkan Renderer".to_string(),
            width: 1280,
            height: 720,
            fullscreen: false,
        }
    }
}

/// Graphics settings
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct GraphicsConfig {
    pub present_mode: String,
    pub clear_color: [f32; 4],
    pub max_frames_in_flight: usize,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            present_mode: "immediate".to_string(),
            clear_color: [0.1, 0.2, 0.8, 1.0],
            max_frames_in_flight: 2,
        }
    }
}

/// Debug settings
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DebugConfig {
    pub validation_layers: bool,
    pub log_to_file: bool,
    pub log_file: String,
    pub show_fps: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            validation_layers: true,
            log_to_file: true,
            log_file: "vulkan_debug.log".to_string(),
            show_fps: true,
        }
    }
}

/// Control key bindings
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ControlsConfig {
    pub fullscreen_key: String,
    pub screenshot_key: String,
    pub quit_key: String,
}

impl Default for ControlsConfig {
    fn default() -> Self {
        Self {
            fullscreen_key: "F11".to_string(),
            screenshot_key: "F12".to_string(),
            quit_key: "Escape".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from file, falling back to defaults if not found
    pub fn load() -> Self {
        Self::load_from_path("config.toml").unwrap_or_else(|e| {
            log::warn!("Failed to load config.toml: {}. Using defaults.", e);
            Config::default()
        })
    }
    
    /// Load configuration from a specific path
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            log::info!("Config file not found at {:?}, using defaults", path);
            return Ok(Config::default());
        }
        
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;
        
        log::info!("Loaded configuration from {:?}", path);
        log::debug!("Config: {:?}", config);
        
        Ok(config)
    }
    
    /// Get present mode as Vulkan enum
    pub fn get_present_mode(&self) -> ash::vk::PresentModeKHR {
        match self.graphics.present_mode.to_lowercase().as_str() {
            "immediate" => ash::vk::PresentModeKHR::IMMEDIATE,
            "mailbox" => ash::vk::PresentModeKHR::MAILBOX,
            "fifo" => ash::vk::PresentModeKHR::FIFO,
            "fifo_relaxed" => ash::vk::PresentModeKHR::FIFO_RELAXED,
            _ => {
                log::warn!(
                    "Unknown present mode '{}', defaulting to IMMEDIATE",
                    self.graphics.present_mode
                );
                ash::vk::PresentModeKHR::IMMEDIATE
            }
        }
    }
}
