//! Configuration management for Pi Rust Client
//!
//! Supports:
//! - Multiple provider profiles (e.g., Anthropic, DashScope, local)
//! - Persistent config file storage
//! - API key and base URL configuration
//! - Model profiles for quick switching
//! - OpenClaw-style SecretRef for secure credential reference
//! - Model definitions with full capability metadata

#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ============================================================================
// OpenClaw-style Secret Reference Types
// ============================================================================

/// Secret reference for secure credential storage
/// Supports environment variable references like OpenClaw's SecretInput
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum SecretInput {
    /// Literal value stored directly
    Literal(String),
    /// Reference to environment variable
    Ref(SecretRef),
}

/// Reference to an environment variable for secrets
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretRef {
    /// Source type: "env" for environment variable
    pub source: String,
    /// Environment variable name or identifier
    pub id: String,
}

impl SecretInput {
    /// Resolve the secret value from environment or literal
    pub fn resolve(&self) -> Result<String> {
        match self {
            SecretInput::Literal(value) => Ok(value.clone()),
            SecretInput::Ref(ref_val) => {
                if ref_val.source == "env" {
                    std::env::var(&ref_val.id)
                        .context(format!("Environment variable '{}' not set", ref_val.id))
                } else {
                    anyhow::bail!("Unknown secret source: {}", ref_val.source)
                }
            }
        }
    }

    /// Create from environment variable reference
    pub fn from_env(var_name: &str) -> Self {
        SecretInput::Ref(SecretRef {
            source: "env".to_string(),
            id: var_name.to_string(),
        })
    }

    /// Create from literal value
    pub fn from_literal(value: &str) -> Self {
        SecretInput::Literal(value.to_string())
    }
}

// ============================================================================
// Model Definition Types (OpenClaw-style)
// ============================================================================

/// Model input capabilities
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelInputType {
    Text,
    Image,
    Video,
    Audio,
    Document,
}

/// Model cost configuration (per million tokens in USD)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelCost {
    /// Input token cost (USD per million)
    pub input: f64,
    /// Output token cost (USD per million)
    pub output: f64,
    /// Cache read cost (USD per million)
    #[serde(default)]
    pub cache_read: f64,
    /// Cache write cost (USD per million)
    #[serde(default)]
    pub cache_write: f64,
}

impl Default for ModelCost {
    fn default() -> Self {
        Self {
            input: 0.0,
            output: 0.0,
            cache_read: 0.0,
            cache_write: 0.0,
        }
    }
}

/// Model compatibility configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelCompat {
    /// Thinking format: "anthropic", "deepseek", "openrouter", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_format: Option<String>,
    /// Whether model supports tool use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_tools: Option<bool>,
    /// Supported reasoning effort levels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_reasoning_efforts: Option<Vec<String>>,
}

/// Full model definition (OpenClaw ModelDefinitionConfig style)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelDefinition {
    /// Model ID (e.g., "claude-sonnet-4-6")
    pub id: String,
    /// Display name (e.g., "Claude Sonnet 4.6")
    pub name: String,
    /// API type override for this model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    /// Base URL override for this model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Whether model supports extended thinking/reasoning
    #[serde(default)]
    pub reasoning: bool,
    /// Input capabilities
    #[serde(default = "default_input_types")]
    pub input: Vec<ModelInputType>,
    /// Cost configuration
    #[serde(default)]
    pub cost: ModelCost,
    /// Context window size (tokens)
    pub context_window: u32,
    /// Effective context tokens (may be less than window)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_tokens: Option<u32>,
    /// Maximum output tokens
    pub max_tokens: u32,
    /// Compatibility settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compat: Option<ModelCompat>,
}

fn default_input_types() -> Vec<ModelInputType> {
    vec![ModelInputType::Text]
}

// ============================================================================
// Provider Configuration Types
// ============================================================================

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
    /// API key for this provider (supports SecretInput)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<SecretInput>,
    /// Default model ID for this profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    /// Context window override for all models in this provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u32>,
    /// Max tokens override for all models in this provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Model definitions for this provider
    #[serde(default)]
    pub models: Vec<ModelDefinition>,
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

// ============================================================================
// Main Configuration Structure
// ============================================================================

/// Configuration file location
pub fn config_path() -> PathBuf {
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
    /// Config mode: "merge" with defaults or "replace" entirely
    #[serde(default = "default_mode")]
    pub mode: String,
}

fn default_profile() -> String {
    "anthropic".to_string()
}

fn default_mode() -> String {
    "merge".to_string()
}

impl Default for PiConfig {
    fn default() -> Self {
        Self {
            default_profile: "anthropic".to_string(),
            mode: "merge".to_string(),
            profiles: vec![
                ProviderProfile {
                    name: "anthropic".to_string(),
                    provider: "anthropic".to_string(),
                    api: None,
                    base_url: None,
                    api_key: None,
                    default_model: None,
                    context_window: None,
                    max_tokens: None,
                    description: Some("Anthropic Claude API".to_string()),
                    models: vec![
                        ModelDefinition {
                            id: "claude-opus-4-7".to_string(),
                            name: "Claude Opus 4.7".to_string(),
                            api: None,
                            base_url: None,
                            reasoning: true,
                            input: vec![ModelInputType::Text, ModelInputType::Image],
                            cost: ModelCost {
                                input: 15.0,
                                output: 75.0,
                                cache_read: 1.5,
                                cache_write: 20.0,
                            },
                            context_window: 200_000,
                            context_tokens: None,
                            max_tokens: 16_000,
                            compat: Some(ModelCompat {
                                thinking_format: Some("anthropic".to_string()),
                                supports_tools: Some(true),
                                supported_reasoning_efforts: Some(vec!["low".to_string(), "medium".to_string(), "high".to_string()]),
                            }),
                        },
                        ModelDefinition {
                            id: "claude-sonnet-4-6".to_string(),
                            name: "Claude Sonnet 4.6".to_string(),
                            api: None,
                            base_url: None,
                            reasoning: true,
                            input: vec![ModelInputType::Text, ModelInputType::Image],
                            cost: ModelCost {
                                input: 3.0,
                                output: 15.0,
                                cache_read: 0.3,
                                cache_write: 4.0,
                            },
                            context_window: 200_000,
                            context_tokens: None,
                            max_tokens: 16_000,
                            compat: Some(ModelCompat {
                                thinking_format: Some("anthropic".to_string()),
                                supports_tools: Some(true),
                                supported_reasoning_efforts: Some(vec!["low".to_string(), "medium".to_string(), "high".to_string()]),
                            }),
                        },
                        ModelDefinition {
                            id: "claude-haiku-4-5-20251001".to_string(),
                            name: "Claude Haiku 4.5".to_string(),
                            api: None,
                            base_url: None,
                            reasoning: false,
                            input: vec![ModelInputType::Text, ModelInputType::Image],
                            cost: ModelCost {
                                input: 0.8,
                                output: 4.0,
                                cache_read: 0.08,
                                cache_write: 1.0,
                            },
                            context_window: 200_000,
                            context_tokens: None,
                            max_tokens: 8_000,
                            compat: Some(ModelCompat {
                                thinking_format: None,
                                supports_tools: Some(true),
                                supported_reasoning_efforts: None,
                            }),
                        },
                    ],
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

    /// Set API key for a profile (literal value)
    pub fn set_api_key(&mut self, profile_name: &str, api_key: &str) -> Result<()> {
        let profile = self.profiles.iter_mut()
            .find(|p| p.name == profile_name)
            .context("Profile not found")?;
        profile.api_key = Some(SecretInput::from_literal(api_key));
        self.save()?;
        Ok(())
    }

    /// Set API key for a profile from environment variable
    pub fn set_api_key_from_env(&mut self, profile_name: &str, env_var: &str) -> Result<()> {
        let profile = self.profiles.iter_mut()
            .find(|p| p.name == profile_name)
            .context("Profile not found")?;
        profile.api_key = Some(SecretInput::from_env(env_var));
        self.save()?;
        Ok(())
    }

    /// Create DashScope profile for Alibaba 百炼
    pub fn create_dashscope_profile(api_key: &str) -> ProviderProfile {
        ProviderProfile {
            name: "dashscope".to_string(),
            provider: "anthropic".to_string(),
            api: Some("anthropic-messages".to_string()),
            base_url: Some("https://coding.dashscope.aliyuncs.com/apps/anthropic".to_string()),
            api_key: Some(SecretInput::from_literal(api_key)),
            default_model: Some("qwen-coder-plus".to_string()),
            context_window: None,
            max_tokens: None,
            description: Some("DashScope API (Anthropic compatible)".to_string()),
            models: vec![],
        }
    }

    /// Create DashScope profile with environment variable reference
    pub fn create_dashscope_profile_from_env(env_var: &str) -> ProviderProfile {
        ProviderProfile {
            name: "dashscope".to_string(),
            provider: "anthropic".to_string(),
            api: Some("anthropic-messages".to_string()),
            base_url: Some("https://coding.dashscope.aliyuncs.com/apps/anthropic".to_string()),
            api_key: Some(SecretInput::from_env(env_var)),
            default_model: Some("qwen-coder-plus".to_string()),
            context_window: None,
            max_tokens: None,
            description: Some("DashScope API (Anthropic compatible)".to_string()),
            models: vec![],
        }
    }
}

/// Print configuration summary
pub fn print_config(config: &PiConfig) {
    println!("Configuration: {}", config_path().display());
    println!("Mode: {}", config.mode);
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
            match key {
                SecretInput::Literal(value) => {
                    let masked = if value.len() > 10 {
                        format!("{}...{}", &value[..8], &value[value.len()-4..])
                    } else {
                        "***".to_string()
                    };
                    println!("    API Key: {}", masked);
                }
                SecretInput::Ref(ref_val) => {
                    println!("    API Key: ${{{}}}", ref_val.id);
                }
            }
        }
        if let Some(ctx) = profile.context_window {
            println!("    Context Window: {} tokens", ctx);
        }
        if let Some(mt) = profile.max_tokens {
            println!("    Max Tokens: {}", mt);
        }
        if let Some(desc) = &profile.description {
            println!("    Description: {}", desc);
        }
        if !profile.models.is_empty() {
            println!("    Models:");
            for model in &profile.models {
                let reasoning_tag = if model.reasoning { " [reasoning]" } else { "" };
                println!("      - {}{}: {}", model.id, reasoning_tag, model.name);
                println!("        Context: {}, Max: {}", model.context_window, model.max_tokens);
            }
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