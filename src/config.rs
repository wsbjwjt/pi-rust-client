//! Configuration management for Pi Rust Client
//!
//! Supports:
//! - Multiple provider profiles (e.g., Anthropic, DashScope, local)
//! - Persistent config file storage
//! - API key and base URL configuration
//! - Model profiles for quick switching

#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration file location
pub fn config_path() -> PathBuf {
    // Use user's home directory
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".pi-client").join("config.json")
}

/// Main configuration structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PiConfig {
    /// Default profile name
    #[serde(default = "default_profile")]
    pub default_profile: String,
    /// Available profiles
    #[serde(default)]
    pub profiles: Vec<ProviderProfile>,
    /// Model presets for quick switching
    #[serde(default)]
    pub model_presets: Vec<ModelPreset>,
}

fn default_profile() -> String {
    "anthropic".to_string()
}

impl Default for PiConfig {
    fn default() -> Self {
        Self {
            default_profile: "anthropic".to_string(),
            profiles: vec![
                ProviderProfile {
                    name: "anthropic".to_string(),
                    provider: "anthropic".to_string(),
                    api: None, // Uses default anthropic-messages
                    base_url: None,
                    api_key: None,
                    default_model: None,
                    description: Some("Anthropic Claude API".to_string()),
                },
            ],
            model_presets: vec![
                ModelPreset {
                    name: "opus".to_string(),
                    model_id: "claude-opus-4-7".to_string(),
                    description: Some("Most capable model".to_string()),
                },
                ModelPreset {
                    name: "sonnet".to_string(),
                    model_id: "claude-sonnet-4-6".to_string(),
                    description: Some("Balanced model".to_string()),
                },
                ModelPreset {
                    name: "haiku".to_string(),
                    model_id: "claude-haiku-4-5-20251001".to_string(),
                    description: Some("Fast and efficient".to_string()),
                },
            ],
        }
    }
}

/// Provider profile for different API endpoints
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderProfile {
    /// Profile name (used to switch profiles)
    pub name: String,
    /// Provider identifier (anthropic, openai, etc.)
    pub provider: String,
    /// API type for this provider (anthropic-messages, openai-completions, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    /// Custom base URL (for compatible APIs like DashScope)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// API key for this provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Default model ID for this profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Model preset for quick switching
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelPreset {
    /// Preset name (short identifier)
    pub name: String,
    /// Model ID to use
    pub model_id: String,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl PiConfig {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = config_path();

        if !path.exists() {
            // Create default config
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read config file")?;

        let config: PiConfig = serde_json::from_str(&content)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = config_path();

        // Create directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&path, content)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Get current/default provider profile
    pub fn current_profile(&self) -> Option<&ProviderProfile> {
        self.profiles.iter()
            .find(|p| p.name == self.default_profile)
    }

    /// Get profile by name
    pub fn get_profile(&self, name: &str) -> Option<&ProviderProfile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    /// Add or update a profile
    pub fn set_profile(&mut self, profile: ProviderProfile) {
        // Remove existing profile with same name
        self.profiles.retain(|p| p.name != profile.name);
        self.profiles.push(profile);
    }

    /// Set default profile
    pub fn set_default_profile(&mut self, name: &str) -> Result<()> {
        if !self.profiles.iter().any(|p| p.name == name) {
            anyhow::bail!("Profile '{}' not found", name);
        }
        self.default_profile = name.to_string();
        self.save()?;
        Ok(())
    }

    /// Get model preset by name
    pub fn get_model_preset(&self, name: &str) -> Option<&ModelPreset> {
        self.model_presets.iter().find(|m| m.name == name)
    }

    /// Add or update a model preset
    pub fn set_model_preset(&mut self, preset: ModelPreset) {
        self.model_presets.retain(|m| m.name != preset.name);
        self.model_presets.push(preset);
    }

    /// Set API key for a profile
    pub fn set_api_key(&mut self, profile_name: &str, api_key: &str) -> Result<()> {
        let profile = self.profiles.iter_mut()
            .find(|p| p.name == profile_name)
            .context("Profile not found")?;
        profile.api_key = Some(api_key.to_string());
        self.save()?;
        Ok(())
    }

    /// Create DashScope profile for Alibaba 百炼
    pub fn create_dashscope_profile(api_key: &str) -> ProviderProfile {
        ProviderProfile {
            name: "dashscope".to_string(),
            provider: "anthropic".to_string(), // Compatible with Anthropic protocol
            api: Some("anthropic-messages".to_string()),
            base_url: Some("https://coding.dashscope.aliyuncs.com/apps/anthropic".to_string()),
            api_key: Some(api_key.to_string()),
            default_model: Some("qwen-coder-plus".to_string()),
            description: Some("DashScope API (Anthropic compatible)".to_string()),
        }
    }
}

/// Print configuration summary
pub fn print_config(config: &PiConfig) {
    println!("Configuration: {}", config_path().display());
    println!();

    println!("Default Profile: {}", config.default_profile);
    println!();

    println!("Provider Profiles:");
    for profile in &config.profiles {
        let current = if profile.name == config.default_profile { " (current)" } else { "" };
        println!("  {}{}", profile.name, current);
        println!("    Provider: {}", profile.provider);
        if let Some(api) = &profile.api {
            println!("    API: {}", api);
        }
        if let Some(url) = &profile.base_url {
            println!("    Base URL: {}", url);
        }
        if let Some(model) = &profile.default_model {
            println!("    Default Model: {}", model);
        }
        if let Some(key) = &profile.api_key {
            let masked = if key.len() > 10 {
                format!("{}...{}", &key[..8], &key[key.len()-4..])
            } else {
                "***".to_string()
            };
            println!("    API Key: {}", masked);
        }
        if let Some(desc) = &profile.description {
            println!("    Description: {}", desc);
        }
    }

    println!();
    println!("Model Presets:");
    for preset in &config.model_presets {
        println!("  {}: {}", preset.name, preset.model_id);
        if let Some(desc) = &preset.description {
            println!("    {}", desc);
        }
    }
}