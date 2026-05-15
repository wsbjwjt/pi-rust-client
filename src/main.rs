//! Pi Rust RPC Client Example
//!
//! A complete Rust client for communicating with Pi coding agent
//! via the JSONL RPC protocol.
//!
//! Usage:
//!   cargo run -- <command>
//!
//! Commands:
//!   prompt <message>     Send a prompt and get response
//!   state                Get current session state
//!   models               List available models
//!   stats                Get session statistics
//!   bash <command>       Execute a bash command
//!   interactive          Start interactive REPL mode
//!   config               Show/manage configuration
//!   profile              List/set provider profiles
//!   use-profile <name>   Switch to a provider profile
//!   use-model <name>     Switch to a model preset

mod config;
mod rpc_client;
mod types;

use anyhow::Result;
use config::{PiConfig, ProviderProfile, ModelPreset, print_config, config_path};
use rpc_client::{PiClient, PiClientConfig};
use std::io::{self, BufRead};
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<()> {
    // Initialize tracing
    FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .pretty()
        .init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = &args[1];

    // Build client config with environment variables
    let pi_path = std::env::var("PI_PATH")
        .ok()
        .unwrap_or_else(|| "pi".to_string());
    let pi_script = std::env::var("PI_SCRIPT").ok();
    let pi_cwd = std::env::var("PI_CWD").ok();

    let config = PiClientConfig::new()
        .pi_path(pi_path)
        .pi_script(pi_script)
        .pi_cwd(pi_cwd)
        .provider(std::env::var("PI_PROVIDER").ok())
        .model(std::env::var("PI_MODEL").ok())
        .api_key(std::env::var("PI_API_KEY").ok())
        .base_url(std::env::var("PI_BASE_URL").ok())
        .working_dir(std::env::var("PI_WORKING_DIR").ok());

    match command.as_str() {
        "prompt" => {
            if args.len() < 3 {
                eprintln!("Usage: prompt <message>");
                return Ok(());
            }
            let message = args[2..].join(" ");
            run_prompt(config, message)?;
        }
        "state" => {
            run_get_state(config)?;
        }
        "models" => {
            run_get_models(config)?;
        }
        "stats" => {
            run_get_stats(config)?;
        }
        "bash" => {
            if args.len() < 3 {
                eprintln!("Usage: bash <command>");
                return Ok(());
            }
            let command = args[2..].join(" ");
            run_bash(config, command)?;
        }
        "commands" => {
            run_get_commands(config)?;
        }
        "interactive" => {
            run_interactive(config)?;
        }
        "demo" => {
            run_demo(config)?;
        }
        "config" => {
            run_config(&args[2..])?;
        }
        "profile" => {
            run_profile(&args[2..])?;
        }
        "use-profile" => {
            if args.len() < 3 {
                eprintln!("Usage: use-profile <name>");
                return Ok(());
            }
            run_use_profile(&args[2])?;
        }
        "use-model" => {
            if args.len() < 3 {
                eprintln!("Usage: use-model <preset-name>");
                return Ok(());
            }
            run_use_model(&args[2])?;
        }
        "init-dashscope" => {
            if args.len() < 3 {
                eprintln!("Usage: init-dashscope <api-key>");
                return Ok(());
            }
            run_init_dashscope(&args[2])?;
        }
        "model" => {
            run_model(&args[2..])?;
        }
        "chat" => {
            if args.len() < 3 {
                // Interactive chat mode
                run_interactive(config)?;
            } else {
                // Single message chat
                let message = args[2..].join(" ");
                run_prompt(config, message)?;
            }
        }
        _ => {
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Pi Rust RPC Client");
    println!();
    println!("Usage: pi-client <command> [args]");
    println!();
    println!("Commands:");
    println!("  prompt <message>    Send a prompt and wait for response");
    println!("  state               Get current session state");
    println!("  models              List available models");
    println!("  stats               Get session statistics");
    println!("  bash <cmd>          Execute a bash command");
    println!("  commands            List available commands");
    println!("  interactive         Start interactive REPL mode");
    println!("  demo                Run a full demonstration");
    println!();
    println!("Configuration:");
    println!("  config              Show current configuration");
    println!("  config set-api-key <profile> <key>  Set API key for profile");
    println!("  profile             List all provider profiles");
    println!("  profile add <name> <provider> [base-url]  Add new profile");
    println!("  model               List all model presets");
    println!("  model add <name> <model-id>  Add model preset");
    println!("  use-profile <name>  Switch to a provider profile");
    println!("  use-model <preset>  Switch to a model preset");
    println!("  init-dashscope <key>  Setup DashScope profile");
    println!();
    println!("Chat:");
    println!("  chat                Start interactive chat (uses config)");
    println!("  chat <message>      Send single message");
    println!("  interactive         Full REPL mode");
    println!("  prompt <message>    Send prompt and wait for response");
    println!();
    println!("Environment Variables:");
    println!("  PI_PROVIDER         Provider to use (anthropic, openai, etc.)");
    println!("  PI_MODEL            Model pattern or ID");
    println!("  PI_API_KEY          API key for the provider");
    println!("  PI_BASE_URL         Custom base URL (for compatible APIs like DashScope)");
    println!("  PI_CWD              Directory to run pi script from (required with PI_SCRIPT)");
    println!("  PI_WORKING_DIR      Working directory for pi");
    println!("  PI_PATH             Path to pi executable (default: pi)");
    println!("  PI_SCRIPT           Path to pi TypeScript script (use with npx tsx)");
    println!();
    println!("Config file: {}", config_path().display());
}

/// Simple prompt - just send and wait
fn run_prompt(config: PiClientConfig, message: String) -> Result<()> {
    println!("Sending prompt: {}", message);
    println!("---");

    let client = PiClient::new(config)?;
    let messages = client.prompt(message)?;

    println!("---");
    println!("Received {} messages", messages.len());

    // Extract last assistant text
    for msg in messages.iter().rev() {
        if let Some(role) = msg.get("role").and_then(|r| r.as_str()) {
            if role == "assistant" {
                if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
                    for block in content {
                        if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                            println!("{}", text);
                        }
                    }
                }
                break;
            }
        }
    }

    Ok(())
}

/// Get session state
fn run_get_state(config: PiClientConfig) -> Result<()> {
    let client = PiClient::new(config)?;
    let state = client.get_state()?;

    println!("Session State:");
    println!("  Streaming: {}", state.is_streaming);
    println!("  Compacting: {}", state.is_compacting);
    println!("  Thinking Level: {}", state.thinking_level);
    println!("  Steering Mode: {}", state.steering_mode);
    println!("  Follow-up Mode: {}", state.follow_up_mode);
    println!("  Messages: {}", state.message_count);
    println!("  Pending: {}", state.pending_message_count);

    if let Some(model) = state.model {
        println!("  Model: {} ({})", model.name, model.provider);
        println!("  Context Window: {}", model.context_window);
    }

    if let Some(session_id) = state.session_id {
        println!("  Session ID: {}", session_id);
    }

    Ok(())
}

/// Get available models
fn run_get_models(config: PiClientConfig) -> Result<()> {
    let client = PiClient::new(config)?;
    let models = client.get_available_models()?;

    println!("Available Models:");
    println!("{:<30} {:<15} {:<10} {:<10}", "Name", "Provider", "Reasoning", "Context");
    println!("{}", "-".repeat(70));

    for model in models {
        println!(
            "{:<30} {:<15} {:<10} {:<10}",
            model.name,
            model.provider,
            model.reasoning,
            model.context_window
        );
    }

    Ok(())
}

/// Get session statistics
fn run_get_stats(config: PiClientConfig) -> Result<()> {
    let client = PiClient::new(config)?;
    let stats = client.get_session_stats()?;

    println!("Session Statistics:");
    println!("  User Messages: {}", stats.user_messages);
    println!("  Assistant Messages: {}", stats.assistant_messages);
    println!("  Tool Calls: {}", stats.tool_calls);
    println!("  Tool Results: {}", stats.tool_results);
    println!("  Total Messages: {}", stats.total_messages);
    println!();
    println!("Token Usage:");
    println!("  Input: {}", stats.tokens.input);
    println!("  Output: {}", stats.tokens.output);
    println!("  Cache Read: {}", stats.tokens.cache_read);
    println!("  Cache Write: {}", stats.tokens.cache_write);
    println!("  Total: {}", stats.tokens.total);
    println!();
    println!("Cost: ${}", stats.cost);

    if let Some(ctx) = stats.context_usage {
        println!();
        println!("Context Usage:");
        println!("  Tokens: {}", ctx.tokens);
        println!("  Window: {}", ctx.context_window);
        println!("  Percent: {:.1}%", ctx.percent);
    }

    Ok(())
}

/// Execute bash command
fn run_bash(config: PiClientConfig, command: String) -> Result<()> {
    println!("Executing: {}", command);
    println!("---");

    let client = PiClient::new(config)?;
    let result = client.bash(command)?;

    println!("{}", result.output);

    if result.truncated {
        println!("\n[Output truncated]");
        if let Some(path) = result.full_output_path {
            println!("Full output at: {}", path);
        }
    }

    println!("\nExit code: {}", result.exit_code);

    Ok(())
}

/// Get available commands
fn run_get_commands(config: PiClientConfig) -> Result<()> {
    let client = PiClient::new(config)?;
    let commands = client.get_commands()?;

    println!("Available Commands:");
    println!("{:<20} {:<10} {:<40}", "Name", "Source", "Description");
    println!("{}", "-".repeat(70));

    for cmd in commands {
        println!(
            "{:<20} {:<10} {:<40}",
            cmd.name,
            cmd.source,
            cmd.description.unwrap_or_default()
        );
    }

    Ok(())
}

/// Interactive REPL mode
fn run_interactive(config: PiClientConfig) -> Result<()> {
    println!("Pi Interactive Mode");
    println!("Type your prompts, 'exit' to quit, 'state' for session state");
    println!("---");

    let client = PiClient::new(config)?;

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line?;
        let trimmed = input.trim();

        if trimmed == "exit" || trimmed == "quit" {
            break;
        }

        if trimmed == "state" {
            let state = client.get_state()?;
            println!("Messages: {}, Streaming: {}", state.message_count, state.is_streaming);
            continue;
        }

        if trimmed == "stats" {
            let stats = client.get_session_stats()?;
            println!("Tokens: {}, Cost: ${}", stats.tokens.total, stats.cost);
            continue;
        }

        if trimmed == "models" {
            let models = client.get_available_models()?;
            for model in models {
                println!("  {}", model.name);
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        println!("[Processing...]");
        let messages = client.prompt(trimmed.to_string())?;

        // Print last assistant response
        for msg in messages.iter().rev() {
            if msg.get("role").and_then(|r| r.as_str()) == Some("assistant") {
                if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
                    for block in content {
                        if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                            println!("{}", text);
                        }
                    }
                }
                break;
            }
        }
        println!("---");
    }

    println!("Goodbye!");
    Ok(())
}

/// Full demonstration
fn run_demo(config: PiClientConfig) -> Result<()> {
    println!("=== Pi Rust Client Demonstration ===");
    println!();

    let client = PiClient::new(config)?;

    // 1. Get available models
    println!("1. Fetching available models...");
    let models = client.get_available_models()?;
    println!("   Found {} models", models.len());
    if let Some(first) = models.first() {
        println!("   First: {} ({})", first.name, first.provider);
    }
    println!();

    // 2. Get commands
    println!("2. Fetching available commands...");
    let commands = client.get_commands()?;
    println!("   Found {} commands", commands.len());
    println!();

    // 3. Get initial state
    println!("3. Getting initial state...");
    let state = client.get_state()?;
    println!("   Thinking level: {}", state.thinking_level);
    println!("   Streaming: {}", state.is_streaming);
    println!();

    // 4. Send a simple prompt
    println!("4. Sending test prompt...");
    println!("   Prompt: \"What is 2 + 2?\"");
    let messages = client.prompt("What is 2 + 2? Answer briefly.".to_string())?;
    println!("   Received {} messages", messages.len());

    // Extract assistant response
    for msg in messages.iter().rev() {
        if msg.get("role").and_then(|r| r.as_str()) == Some("assistant") {
            if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
                for block in content {
                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                        println!("   Response: {}", text.lines().next().unwrap_or(""));
                    }
                }
            }
            break;
        }
    }
    println!();

    // 5. Get session stats
    println!("5. Getting session statistics...");
    let stats = client.get_session_stats()?;
    println!("   Total tokens: {}", stats.tokens.total);
    println!("   Cost: ${}", stats.cost);
    println!("   Messages: {}", stats.total_messages);
    println!();

    // 6. Execute a bash command
    println!("6. Executing bash command...");
    println!("   Command: \"echo Hello from Rust\"");
    let result = client.bash("echo Hello from Rust".to_string())?;
    println!("   Output: {}", result.output.trim());
    println!("   Exit code: {}", result.exit_code);
    println!();

    // 7. Send another prompt with context
    println!("7. Sending contextual prompt...");
    println!("   Prompt: \"What programming language am I written in?\"");
    let messages = client.prompt("Based on this conversation, what programming language is this client written in?".to_string())?;
    for msg in messages.iter().rev() {
        if msg.get("role").and_then(|r| r.as_str()) == Some("assistant") {
            if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
                for block in content {
                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                        println!("   Response: {}", text);
                    }
                }
            }
            break;
        }
    }
    println!();

    // 8. Final stats
    println!("8. Final session statistics...");
    let stats = client.get_session_stats()?;
    println!("   Total tokens used: {}", stats.tokens.total);
    println!("   Total cost: ${}", stats.cost);
    println!();

    println!("=== Demonstration Complete ===");

    Ok(())
}

// ============================================================================
// Configuration Commands
// ============================================================================

/// Show/manipulate configuration
fn run_config(args: &[String]) -> Result<()> {
    let mut cfg = PiConfig::load()?;

    if args.is_empty() {
        // Show current config
        print_config(&cfg);
        return Ok(());
    }

    match args[0].as_str() {
        "set-api-key" => {
            if args.len() < 3 {
                eprintln!("Usage: config set-api-key <profile-name> <api-key>");
                return Ok(());
            }
            let profile_name = &args[1];
            let api_key = &args[2];
            cfg.set_api_key(profile_name, api_key)?;
            println!("API key set for profile '{}'", profile_name);
        }
        "set-default" => {
            if args.len() < 2 {
                eprintln!("Usage: config set-default <profile-name>");
                return Ok(());
            }
            cfg.set_default_profile(&args[1])?;
            println!("Default profile set to '{}'", args[1]);
        }
        "add-model" => {
            if args.len() < 3 {
                eprintln!("Usage: config add-model <preset-name> <model-id> [description]");
                return Ok(());
            }
            let preset = ModelPreset {
                name: args[1].clone(),
                model_id: args[2].clone(),
                description: if args.len() > 3 { Some(args[3].clone()) } else { None },
            };
            cfg.set_model_preset(preset);
            cfg.save()?;
            println!("Model preset '{}' added", args[1]);
        }
        "path" => {
            println!("Config file: {}", config_path().display());
        }
        _ => {
            eprintln!("Unknown config command: {}", args[0]);
            eprintln!("Commands: set-api-key, set-default, add-model, path");
        }
    }

    Ok(())
}

/// List/manipulate provider profiles
fn run_profile(args: &[String]) -> Result<()> {
    let mut cfg = PiConfig::load()?;

    if args.is_empty() {
        // List all profiles
        println!("Provider Profiles:");
        for profile in &cfg.profiles {
            let current = if profile.name == cfg.default_profile { " (current)" } else { "" };
            println!();
            println!("  {}{}", profile.name, current);
            println!("    Provider: {}", profile.provider);
            if let Some(url) = &profile.base_url {
                println!("    Base URL: {}", url);
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
        return Ok(());
    }

    match args[0].as_str() {
        "add" => {
            if args.len() < 3 {
                eprintln!("Usage: profile add <name> <provider> [base-url] [description]");
                return Ok(());
            }
            let profile = ProviderProfile {
                name: args[1].clone(),
                provider: args[2].clone(),
                api: None,
                base_url: if args.len() > 3 { Some(args[3].clone()) } else { None },
                api_key: None,
                default_model: None,
                description: if args.len() > 4 { Some(args[4].clone()) } else { None },
            };
            cfg.set_profile(profile);
            cfg.save()?;
            println!("Profile '{}' added", args[1]);
        }
        "remove" => {
            if args.len() < 2 {
                eprintln!("Usage: profile remove <name>");
                return Ok(());
            }
            let name = &args[1];
            if cfg.default_profile == name.clone() {
                eprintln!("Cannot remove the default profile. Switch to another first.");
                return Ok(());
            }
            cfg.profiles.retain(|p| &p.name != name);
            cfg.save()?;
            println!("Profile '{}' removed", name);
        }
        _ => {
            eprintln!("Unknown profile command: {}", args[0]);
            eprintln!("Commands: add, remove");
        }
    }

    Ok(())
}

/// Switch to a provider profile
fn run_use_profile(name: &str) -> Result<()> {
    let mut cfg = PiConfig::load()?;
    cfg.set_default_profile(name)?;

    // Print current profile details
    if let Some(profile) = cfg.current_profile() {
        println!("Switched to profile: {}", name);
        println!("  Provider: {}", profile.provider);
        if let Some(url) = &profile.base_url {
            println!("  Base URL: {}", url);
        }
    }

    // Generate environment variable settings
    println!();
    println!("To use this profile, set environment variables:");
    if let Some(profile) = cfg.current_profile() {
        println!("  set PI_PROVIDER={}", profile.provider);
        if let Some(url) = &profile.base_url {
            println!("  set PI_BASE_URL={}", url);
        }
        if let Some(key) = &profile.api_key {
            println!("  set PI_API_KEY={}", key);
        }
    }

    Ok(())
}

/// Switch to a model preset
fn run_use_model(preset_name: &str) -> Result<()> {
    let cfg = PiConfig::load()?;

    if let Some(preset) = cfg.get_model_preset(preset_name) {
        println!("Model preset: {}", preset_name);
        println!("  Model ID: {}", preset.model_id);
        if let Some(desc) = &preset.description {
            println!("  Description: {}", desc);
        }
        println!();
        println!("To use this model, set:");
        println!("  set PI_MODEL={}", preset.model_id);
    } else {
        eprintln!("Model preset '{}' not found", preset_name);
        println!("Available presets:");
        for p in &cfg.model_presets {
            println!("  {}: {}", p.name, p.model_id);
        }
    }

    Ok(())
}

/// Initialize DashScope profile for Alibaba 百炼
fn run_init_dashscope(api_key: &str) -> Result<()> {
    let mut cfg = PiConfig::load()?;

    // Create DashScope profile
    let dashscope = PiConfig::create_dashscope_profile(api_key);
    cfg.set_profile(dashscope.clone());
    cfg.set_default_profile("dashscope")?;
    cfg.save()?;

    println!("DashScope profile initialized!");
    println!();
    println!("Profile: dashscope");
    println!("  Provider: {}", dashscope.provider);
    println!("  Base URL: {}", dashscope.base_url.unwrap_or_default());
    println!("  API Key: {}...{}", &api_key[..8], &api_key[api_key.len()-4..]);
    println!("  Description: {}", dashscope.description.unwrap_or_default());
    println!();
    println!("Config saved to: {}", config_path().display());
    println!();
    println!("To use DashScope, set environment variables:");
    println!("  set PI_PROVIDER=anthropic");
    println!("  set PI_MODEL=qwen-coder-plus  # or other DashScope models");
    println!("  set PI_API_KEY={}", api_key);
    println!();
    println!("Or update run-client.cmd to use these settings.");

    Ok(())
}

/// Manage model presets
fn run_model(args: &[String]) -> Result<()> {
    let mut cfg = PiConfig::load()?;

    if args.is_empty() {
        // List all model presets
        println!("Model Presets:");
        println!("{:<15} {:<30} {:<40}", "Name", "Model ID", "Description");
        println!("{}", "-".repeat(85));

        for preset in &cfg.model_presets {
            println!(
                "{:<15} {:<30} {:<40}",
                preset.name,
                preset.model_id,
                preset.description.as_deref().unwrap_or("")
            );
        }
        println!();
        println!("Usage: use-model <preset-name>  to switch to a preset");
        return Ok(());
    }

    match args[0].as_str() {
        "add" => {
            if args.len() < 3 {
                eprintln!("Usage: model add <preset-name> <model-id> [description]");
                return Ok(());
            }
            let preset = ModelPreset {
                name: args[1].clone(),
                model_id: args[2].clone(),
                description: if args.len() > 3 { Some(args[3].clone()) } else { None },
            };
            cfg.set_model_preset(preset);
            cfg.save()?;
            println!("Model preset '{}' added with model ID '{}'", args[1], args[2]);
        }
        "remove" => {
            if args.len() < 2 {
                eprintln!("Usage: model remove <preset-name>");
                return Ok(());
            }
            let name = &args[1];
            cfg.model_presets.retain(|m| &m.name != name);
            cfg.save()?;
            println!("Model preset '{}' removed", name);
        }
        "use" => {
            if args.len() < 2 {
                eprintln!("Usage: model use <preset-name>");
                return Ok(());
            }
            run_use_model(&args[1])?;
        }
        _ => {
            eprintln!("Unknown model command: {}", args[0]);
            eprintln!("Commands: add, remove, use");
        }
    }

    Ok(())
}