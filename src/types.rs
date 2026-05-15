//! RPC protocol types for Pi coding agent
//! Based on packages/coding-agent/docs/rpc.md
//!
//! This module defines all RPC commands and events for completeness.
//! Some types may not be used in the CLI demo but are available for library users.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

// ============================================================================
// Commands (sent to pi via stdin)
// ============================================================================

/// Base command structure with optional id for correlation
#[derive(Debug, Serialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum RpcCommand {
    Prompt(PromptCommand),
    Steer(SteerCommand),
    FollowUp(FollowUpCommand),
    Abort(AbortCommand),
    GetState(GetStateCommand),
    GetMessages(GetMessagesCommand),
    SetModel(SetModelCommand),
    CycleModel(CycleModelCommand),
    GetAvailableModels(GetAvailableModelsCommand),
    SetThinkingLevel(SetThinkingLevelCommand),
    Compact(CompactCommand),
    SetAutoCompaction(SetAutoCompactionCommand),
    Bash(BashCommand),
    AbortBash(AbortBashCommand),
    GetSessionStats(GetSessionStatsCommand),
    NewSession(NewSessionCommand),
    SwitchSession(SwitchSessionCommand),
    Fork(ForkCommand),
    Clone(CloneCommand),
    GetForkMessages(GetForkMessagesCommand),
    GetCommands(GetCommandsCommand),
    ExtensionUiResponse(ExtensionUiResponse),
}

#[derive(Debug, Serialize)]
pub struct PromptCommand {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<ImageContent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming_behavior: Option<String>,
}

impl PromptCommand {
    pub fn new(message: String) -> Self {
        Self {
            id: None,
            type_: "prompt".to_string(),
            message,
            images: None,
            streaming_behavior: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SteerCommand {
    #[serde(rename = "type")]
    pub type_: String,
    pub message: String,
}

impl SteerCommand {
    pub fn new(message: String) -> Self {
        Self { type_: "steer".to_string(), message }
    }
}

#[derive(Debug, Serialize)]
pub struct AbortCommand {
    #[serde(rename = "type")]
    pub type_: String,
}

impl AbortCommand {
    pub fn new() -> Self { Self { type_: "abort".to_string() } }
}

#[derive(Debug, Serialize)]
pub struct GetStateCommand {
    #[serde(rename = "type")]
    pub type_: String,
}

impl GetStateCommand {
    pub fn new() -> Self { Self { type_: "get_state".to_string() } }
}

#[derive(Debug, Serialize)]
pub struct SetModelCommand {
    #[serde(rename = "type")]
    pub type_: String,
    pub provider: String,
    pub model_id: String,
}

impl SetModelCommand {
    pub fn new(provider: String, model_id: String) -> Self {
        Self { type_: "set_model".to_string(), provider, model_id }
    }
}

#[derive(Debug, Serialize)]
pub struct GetAvailableModelsCommand {
    #[serde(rename = "type")]
    pub type_: String,
}

impl GetAvailableModelsCommand {
    pub fn new() -> Self { Self { type_: "get_available_models".to_string() } }
}

#[derive(Debug, Serialize)]
pub struct BashCommand {
    #[serde(rename = "type")]
    pub type_: String,
    pub command: String,
}

impl BashCommand {
    pub fn new(command: String) -> Self {
        Self { type_: "bash".to_string(), command }
    }
}

#[derive(Debug, Serialize)]
pub struct GetSessionStatsCommand {
    #[serde(rename = "type")]
    pub type_: String,
}

impl GetSessionStatsCommand {
    pub fn new() -> Self { Self { type_: "get_session_stats".to_string() } }
}

#[derive(Debug, Serialize)]
pub struct GetCommandsCommand {
    #[serde(rename = "type")]
    pub type_: String,
}

impl GetCommandsCommand {
    pub fn new() -> Self { Self { type_: "get_commands".to_string() } }
}

// Additional commands (for completeness, some may not be used in main.rs)
#[derive(Debug, Serialize)]
pub struct FollowUpCommand {
    #[serde(rename = "type")]
    pub type_: String,
    pub message: String,
}

impl FollowUpCommand {
    pub fn new(message: String) -> Self {
        Self { type_: "follow_up".to_string(), message }
    }
}

#[derive(Debug, Serialize)]
pub struct GetMessagesCommand {
    #[serde(rename = "type")]
    pub type_: String,
}

impl GetMessagesCommand {
    pub fn new() -> Self { Self { type_: "get_messages".to_string() } }
}

#[derive(Debug, Serialize)]
pub struct CycleModelCommand {
    #[serde(rename = "type")]
    pub type_: String,
}

impl CycleModelCommand {
    pub fn new() -> Self { Self { type_: "cycle_model".to_string() } }
}

#[derive(Debug, Serialize)]
pub struct SetThinkingLevelCommand {
    #[serde(rename = "type")]
    pub type_: String,
    pub level: String,
}

impl SetThinkingLevelCommand {
    pub fn new(level: String) -> Self {
        Self { type_: "set_thinking_level".to_string(), level }
    }
}

#[derive(Debug, Serialize)]
pub struct CompactCommand {
    #[serde(rename = "type")]
    pub type_: String,
}

impl CompactCommand {
    pub fn new() -> Self { Self { type_: "compact".to_string() } }
}

#[derive(Debug, Serialize)]
pub struct SetAutoCompactionCommand {
    #[serde(rename = "type")]
    pub type_: String,
    pub enabled: bool,
}

impl SetAutoCompactionCommand {
    pub fn new(enabled: bool) -> Self {
        Self { type_: "set_auto_compaction".to_string(), enabled }
    }
}

#[derive(Debug, Serialize)]
pub struct AbortBashCommand {
    #[serde(rename = "type")]
    pub type_: String,
}

impl AbortBashCommand {
    pub fn new() -> Self { Self { type_: "abort_bash".to_string() } }
}

#[derive(Debug, Serialize)]
pub struct NewSessionCommand {
    #[serde(rename = "type")]
    pub type_: String,
}

impl NewSessionCommand {
    pub fn new() -> Self { Self { type_: "new_session".to_string() } }
}

#[derive(Debug, Serialize)]
pub struct SwitchSessionCommand {
    #[serde(rename = "type")]
    pub type_: String,
    pub session_id: String,
}

impl SwitchSessionCommand {
    pub fn new(session_id: String) -> Self {
        Self { type_: "switch_session".to_string(), session_id }
    }
}

#[derive(Debug, Serialize)]
pub struct ForkCommand {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_message_id: Option<String>,
}

impl ForkCommand {
    pub fn new() -> Self { Self { type_: "fork".to_string(), from_message_id: None } }
}

#[derive(Debug, Serialize)]
pub struct CloneCommand {
    #[serde(rename = "type")]
    pub type_: String,
}

impl CloneCommand {
    pub fn new() -> Self { Self { type_: "clone".to_string() } }
}

#[derive(Debug, Serialize)]
pub struct GetForkMessagesCommand {
    #[serde(rename = "type")]
    pub type_: String,
}

impl GetForkMessagesCommand {
    pub fn new() -> Self { Self { type_: "get_fork_messages".to_string() } }
}

// Extension UI Response
#[derive(Debug, Serialize)]
pub struct ExtensionUiResponse {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled: Option<bool>,
}

impl ExtensionUiResponse {
    pub fn value(id: String, value: String) -> Self {
        Self { type_: "extension_ui_response".to_string(), id, value: Some(value), confirmed: None, cancelled: None }
    }
    pub fn cancelled(id: String) -> Self {
        Self { type_: "extension_ui_response".to_string(), id, value: None, confirmed: None, cancelled: Some(true) }
    }
}

// ============================================================================
// Events (streamed from pi via stdout)
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum RpcEvent {
    #[serde(rename = "agent_start")]
    AgentStart,
    #[serde(rename = "agent_end")]
    AgentEnd { messages: Vec<serde_json::Value> },
    #[serde(rename = "turn_start")]
    TurnStart,
    #[serde(rename = "turn_end")]
    TurnEnd { message: serde_json::Value, tool_results: Vec<serde_json::Value> },
    #[serde(rename = "message_update")]
    MessageUpdate { message: serde_json::Value, assistant_message_event: AssistantMessageEvent },
    #[serde(rename = "tool_execution_start")]
    ToolExecutionStart { tool_call_id: String, tool_name: String, args: serde_json::Value },
    #[serde(rename = "tool_execution_end")]
    ToolExecutionEnd { tool_call_id: String, tool_name: String, result: ToolResult, is_error: bool },
    #[serde(rename = "extension_ui_request")]
    ExtensionUiRequest { id: String, method: String, #[serde(flatten)] params: serde_json::Value },
    #[serde(rename = "response")]
    Response(RpcResponse),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum AssistantMessageEvent {
    #[serde(rename = "text_delta")]
    TextDelta { content_index: u32, delta: String },
    #[serde(rename = "thinking_delta")]
    ThinkingDelta { delta: String },
    #[serde(rename = "toolcall_start")]
    ToolCallStart,
    #[serde(rename = "toolcall_end")]
    ToolCallEnd { tool_call: serde_json::Value },
    #[serde(rename = "done")]
    Done { reason: String },
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ToolResult {
    pub content: Vec<TextContent>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TextContent {
    #[serde(rename = "type")]
    pub type_: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RpcResponse {
    pub command: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageContent {
    #[serde(rename = "type")]
    pub type_: String,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SessionState {
    #[serde(rename = "isStreaming")]
    pub is_streaming: bool,
    #[serde(rename = "isCompacting", default)]
    pub is_compacting: bool,
    #[serde(rename = "thinkingLevel")]
    pub thinking_level: String,
    #[serde(rename = "steeringMode", default)]
    pub steering_mode: String,
    #[serde(rename = "followUpMode", default)]
    pub follow_up_mode: String,
    #[serde(rename = "messageCount")]
    pub message_count: u32,
    #[serde(rename = "pendingMessageCount", default)]
    pub pending_message_count: u32,
    #[serde(rename = "autoCompactionEnabled", default)]
    pub auto_compaction_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<Model>,
    #[serde(rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(rename = "sessionFile", skip_serializing_if = "Option::is_none")]
    pub session_file: Option<String>,
    #[serde(rename = "sessionName", skip_serializing_if = "Option::is_none")]
    pub session_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub provider: String,
    #[serde(default)]
    pub reasoning: bool,
    #[serde(rename = "contextWindow")]
    pub context_window: u64,
}

#[derive(Debug, Deserialize)]
pub struct SessionStats {
    #[serde(rename = "userMessages", default)]
    pub user_messages: u32,
    #[serde(rename = "assistantMessages", default)]
    pub assistant_messages: u32,
    #[serde(rename = "toolCalls", default)]
    pub tool_calls: u32,
    #[serde(rename = "toolResults", default)]
    pub tool_results: u32,
    #[serde(rename = "totalMessages")]
    pub total_messages: u32,
    pub tokens: TokenUsage,
    pub cost: f64,
    #[serde(rename = "contextUsage", skip_serializing_if = "Option::is_none")]
    pub context_usage: Option<ContextUsage>,
}

#[derive(Debug, Deserialize)]
pub struct TokenUsage {
    #[serde(default)]
    pub input: u64,
    #[serde(default)]
    pub output: u64,
    #[serde(rename = "cacheRead", default)]
    pub cache_read: u64,
    #[serde(rename = "cacheWrite", default)]
    pub cache_write: u64,
    pub total: u64,
}

#[derive(Debug, Deserialize)]
pub struct ContextUsage {
    pub tokens: u64,
    #[serde(rename = "contextWindow")]
    pub context_window: u64,
    pub percent: f64,
}

#[derive(Debug, Deserialize)]
pub struct CommandInfo {
    pub name: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BashResult {
    pub output: String,
    #[serde(rename = "exitCode")]
    pub exit_code: i32,
    #[serde(default)]
    pub truncated: bool,
    #[serde(rename = "fullOutputPath", skip_serializing_if = "Option::is_none")]
    pub full_output_path: Option<String>,
}

/// Event handler trait for custom event processing
#[allow(dead_code)]
pub trait EventHandler {
    fn on_event(&mut self, event: RpcEvent);
}