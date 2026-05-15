//! Pi RPC Client implementation

use crate::types::*;
use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use tracing::{debug, warn};

pub struct PiClientConfig {
    pub pi_path: String,
    pub pi_script: Option<String>,
    pub pi_cwd: Option<String>,  // Directory to run pi script from
    pub provider: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,  // API key for provider
    pub base_url: Option<String>, // Custom base URL (e.g., DashScope)
    pub no_session: bool,
    pub working_dir: Option<String>,
}

impl Default for PiClientConfig {
    fn default() -> Self {
        Self {
            pi_path: "pi".to_string(),
            pi_script: None,
            pi_cwd: None,
            provider: None,
            model: None,
            api_key: None,
            base_url: None,
            no_session: true,
            working_dir: None,
        }
    }
}

impl PiClientConfig {
    pub fn new() -> Self { Self::default() }
    pub fn pi_path(mut self, p: String) -> Self { self.pi_path = p; self }
    pub fn pi_script(mut self, s: Option<String>) -> Self { self.pi_script = s; self }
    pub fn pi_cwd(mut self, c: Option<String>) -> Self { self.pi_cwd = c; self }
    pub fn provider(mut self, p: Option<String>) -> Self { self.provider = p; self }
    pub fn model(mut self, m: Option<String>) -> Self { self.model = m; self }
    pub fn api_key(mut self, k: Option<String>) -> Self { self.api_key = k; self }
    pub fn base_url(mut self, u: Option<String>) -> Self { self.base_url = u; self }
    pub fn working_dir(mut self, d: Option<String>) -> Self { self.working_dir = d; self }
}

pub struct PiClient {
    stdin_tx: Sender<String>,
    events_rx: Receiver<RpcEvent>,
}

impl PiClient {
    pub fn new(config: PiClientConfig) -> Result<Self> {
        // Build the command to spawn
        #[cfg(target_os = "windows")]
        let mut cmd = {
            if let Some(script) = &config.pi_script {
                // On Windows, need to use cmd.exe to find npx in PATH
                // Build command with full args in one string to avoid /c stripping
                let mut cmd_str = format!("npx tsx {} --mode rpc", script);
                if config.no_session { cmd_str.push_str(" --no-session"); }
                if let Some(p) = &config.provider { cmd_str.push_str(&format!(" --provider {}", p)); }
                if let Some(m) = &config.model { cmd_str.push_str(&format!(" --model {}", m)); }

                let mut c = Command::new("cmd");
                c.args(["/S", "/C", &cmd_str]);

                if let Some(cwd) = &config.pi_cwd {
                    c.current_dir(cwd);
                }
                c
            } else {
                let mut c = Command::new("cmd");
                c.args(["/S", "/C"]);
                c.arg(&format!("\"{} --mode rpc\"", config.pi_path));
                if config.no_session { c.arg("--no-session"); }
                if let Some(p) = &config.provider { c.arg("--provider").arg(p); }
                if let Some(m) = &config.model { c.arg("--model").arg(m); }

                if let Some(cwd) = &config.pi_cwd {
                    c.current_dir(cwd);
                }
                c
            }
        };

        #[cfg(not(target_os = "windows"))]
        let mut cmd = {
            let mut c = if let Some(script) = &config.pi_script {
                let mut cmd = Command::new("npx");
                cmd.arg("tsx").arg(script);
                cmd
            } else {
                Command::new(&config.pi_path)
            };

            if let Some(cwd) = &config.pi_cwd {
                c.current_dir(cwd);
            }

            c.arg("--mode").arg("rpc");
            if config.no_session { c.arg("--no-session"); }
            if let Some(p) = &config.provider { c.arg("--provider").arg(p); }
            if let Some(m) = &config.model { c.arg("--model").arg(m); }

            c
        };

        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());

        debug!("Starting: {:?}", cmd);
        let mut process = cmd.spawn().context("Failed to spawn pi")?;

        // stderr reader thread for debugging
        let stderr_reader = process.stderr.take().context("stderr")?;
        thread::spawn(move || {
            let reader = BufReader::new(stderr_reader);
            for line in reader.lines() {
                if let Ok(line) = line {
                    eprintln!("[stderr] {}", line);
                }
            }
        });

        let (stdin_tx, stdin_rx) = channel::<String>();
        let (events_tx, events_rx) = channel::<RpcEvent>();

        // stdin writer thread
        let mut stdin_writer = process.stdin.take().context("stdin")?;
        thread::spawn(move || {
            while let Ok(json) = stdin_rx.recv() {
                if stdin_writer.write_all(json.as_bytes()).is_err() { break; }
                if stdin_writer.write_all(b"\n").is_err() { break; }
                let _ = stdin_writer.flush();
            }
        });

        // stdout reader thread
        let stdout_reader = process.stdout.take().context("stdout")?;
        thread::spawn(move || {
            let reader = BufReader::new(stdout_reader);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let clean = if line.ends_with('\r') { &line[..line.len()-1] } else { &line };
                    if let Ok(event) = serde_json::from_str::<RpcEvent>(clean) {
                        if events_tx.send(event).is_err() { break; }
                    } else {
                        warn!("Parse failed: {}", clean);
                    }
                }
            }
        });

        Ok(Self { stdin_tx, events_rx })
    }

    fn send(&self, cmd: RpcCommand) -> Result<()> {
        let json = serde_json::to_string(&cmd)?;
        debug!("Send: {}", json);
        self.stdin_tx.send(json).context("send failed")?;
        Ok(())
    }

    /// Send prompt and wait for agent_end
    pub fn prompt(&self, message: String) -> Result<Vec<serde_json::Value>> {
        self.send(RpcCommand::Prompt(PromptCommand::new(message)))?;

        while let Ok(ev) = self.events_rx.recv() {
            match ev {
                RpcEvent::AgentEnd { messages } => return Ok(messages),
                RpcEvent::ExtensionUiRequest { id, method, .. } => {
                    self.handle_ui(&id, &method)?;
                }
                _ => {}
            }
        }
        anyhow::bail!("No agent_end")
    }

    fn handle_ui(&self, id: &str, method: &str) -> Result<()> {
        match method {
            "select" => {
                self.send(RpcCommand::ExtensionUiResponse(ExtensionUiResponse::value(id.into(), "".into())))?;
            }
            "confirm" | "input" | "editor" => {
                self.send(RpcCommand::ExtensionUiResponse(ExtensionUiResponse::cancelled(id.into())))?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Get session state
    pub fn get_state(&self) -> Result<SessionState> {
        self.send(RpcCommand::GetState(GetStateCommand::new()))?;
        while let Ok(ev) = self.events_rx.recv() {
            match ev {
                RpcEvent::Response(resp) => {
                    if resp.command == "get_state" && resp.success {
                        return Ok(serde_json::from_value(resp.data.unwrap())?);
                    }
                }
                RpcEvent::ExtensionUiRequest { id, method, .. } => {
                    self.handle_ui(&id, &method)?;
                }
                _ => {}
            }
        }
        anyhow::bail!("No state response")
    }

    /// Get available models
    pub fn get_available_models(&self) -> Result<Vec<Model>> {
        self.send(RpcCommand::GetAvailableModels(GetAvailableModelsCommand::new()))?;
        while let Ok(ev) = self.events_rx.recv() {
            match ev {
                RpcEvent::Response(resp) => {
                    if resp.command == "get_available_models" && resp.success {
                        let data = resp.data.unwrap();
                        let models = data.get("models").unwrap().as_array().unwrap();
                        return Ok(models.iter().filter_map(|m| serde_json::from_value(m.clone()).ok()).collect());
                    }
                }
                RpcEvent::ExtensionUiRequest { id, method, .. } => {
                    self.handle_ui(&id, &method)?;
                }
                _ => {}
            }
        }
        anyhow::bail!("No models response")
    }

    /// Set/switch model via RPC
    pub fn set_model(&self, provider: &str, model_id: &str) -> Result<()> {
        self.send(RpcCommand::SetModel(SetModelCommand::new(provider.to_string(), model_id.to_string())))?;
        while let Ok(ev) = self.events_rx.recv() {
            match ev {
                RpcEvent::Response(resp) => {
                    if resp.command == "set_model" {
                        if resp.success {
                            return Ok(());
                        } else {
                            anyhow::bail!("set_model failed: {}", resp.error.unwrap_or_else(|| "unknown error".to_string()));
                        }
                    }
                }
                RpcEvent::ExtensionUiRequest { id, method, .. } => {
                    self.handle_ui(&id, &method)?;
                }
                _ => {}
            }
        }
        anyhow::bail!("No set_model response")
    }

    /// Get session stats
    pub fn get_session_stats(&self) -> Result<SessionStats> {
        self.send(RpcCommand::GetSessionStats(GetSessionStatsCommand::new()))?;
        while let Ok(ev) = self.events_rx.recv() {
            match ev {
                RpcEvent::Response(resp) => {
                    if resp.command == "get_session_stats" && resp.success {
                        return Ok(serde_json::from_value(resp.data.unwrap())?);
                    }
                }
                RpcEvent::ExtensionUiRequest { id, method, .. } => {
                    self.handle_ui(&id, &method)?;
                }
                _ => {}
            }
        }
        anyhow::bail!("No stats response")
    }

    /// Execute bash
    pub fn bash(&self, cmd: String) -> Result<BashResult> {
        self.send(RpcCommand::Bash(BashCommand::new(cmd)))?;
        while let Ok(ev) = self.events_rx.recv() {
            match ev {
                RpcEvent::Response(resp) => {
                    if resp.command == "bash" && resp.success {
                        return Ok(serde_json::from_value(resp.data.unwrap())?);
                    }
                }
                RpcEvent::ExtensionUiRequest { id, method, .. } => {
                    self.handle_ui(&id, &method)?;
                }
                _ => {}
            }
        }
        anyhow::bail!("No bash response")
    }

    /// Get commands
    pub fn get_commands(&self) -> Result<Vec<CommandInfo>> {
        self.send(RpcCommand::GetCommands(GetCommandsCommand::new()))?;
        while let Ok(ev) = self.events_rx.recv() {
            match ev {
                RpcEvent::Response(resp) => {
                    if resp.command == "get_commands" && resp.success {
                        let data = resp.data.unwrap();
                        let commands = data.get("commands").unwrap().as_array().unwrap();
                        return Ok(commands.iter().filter_map(|c| serde_json::from_value(c.clone()).ok()).collect());
                    }
                }
                RpcEvent::ExtensionUiRequest { id, method, .. } => {
                    self.handle_ui(&id, &method)?;
                }
                _ => {}
            }
        }
        anyhow::bail!("No commands response")
    }

    /// Abort current operation
    #[allow(dead_code)]
    pub fn abort(&self) -> Result<()> {
        self.send(RpcCommand::Abort(AbortCommand::new()))?;
        Ok(())
    }

    /// Send raw command (for custom use)
    #[allow(dead_code)]
    pub fn send_command(&self, cmd: RpcCommand) -> Result<()> {
        self.send(cmd)
    }
}

/// Simple event handler that prints events
#[allow(dead_code)]
pub struct PrintEventHandler {
    print_tools: bool,
}

impl PrintEventHandler {
    #[allow(dead_code)]
    pub fn new(print_tools: bool) -> Self {
        Self { print_tools }
    }
}

impl crate::types::EventHandler for PrintEventHandler {
    fn on_event(&mut self, event: RpcEvent) {
        match event {
            RpcEvent::AgentStart => {
                println!("[Agent started]");
            }
            RpcEvent::AgentEnd { .. } => {
                println!("[Agent ended]");
            }
            RpcEvent::TurnStart => {
                println!("[Turn started]");
            }
            RpcEvent::TurnEnd { .. } => {
                println!("[Turn ended]");
            }
            RpcEvent::ToolExecutionStart { tool_name, .. } => {
                if self.print_tools {
                    println!("[Tool: {}]", tool_name);
                }
            }
            RpcEvent::ToolExecutionEnd { tool_name, is_error, .. } => {
                if self.print_tools && is_error {
                    println!("[Tool {} failed]", tool_name);
                }
            }
            _ => {}
        }
    }
}