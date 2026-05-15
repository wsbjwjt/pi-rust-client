# Pi Rust Client

一个使用 Rust 语言与 Pi coding agent 进行 RPC 通信的研究性项目。

## 项目简介

Pi Rust Client 是一个轻量级的 Rust RPC 客户端，用于与 Pi coding agent（一个 AI 编程助手）进行通信。项目采用 JSONL RPC 协议，支持多种大模型提供商，包括 Anthropic Claude、DashScope（阿里百炼）等。

**研究目标：**
- 探索 Rust 与 TypeScript 编程助手的高效通信方式
- 测试不同大模型提供商的 API 兼容性
- 验证 CLI 工具在 AI 编程场景下的实用性

## 功能特性

### 核心功能
- ✅ **RPC 通信** - 通过 JSONL 协议与 Pi agent 通信
- ✅ **多 Provider 支持** - Anthropic、DashScope、OpenAI 等
- ✅ **配置管理** - Profile 和 Model Preset 系统
- ✅ **交互模式** - REPL 和单次对话模式
- ✅ **环境变量控制** - PI_PROVIDER、PI_MODEL、PI_API_KEY 等

### CLI 命令

| 命令 | 说明 |
|------|------|
| `prompt <message>` | 发送消息并等待响应 |
| `chat` | 启动交互式对话 |
| `chat <message>` | 单次消息对话 |
| `interactive` | 完整 REPL 模式 |
| `state` | 获取会话状态 |
| `models` | 列出可用模型 |
| `stats` | 获取会话统计 |
| `bash <cmd>` | 执行 bash 命令 |
| `demo` | 运行完整演示 |

### 配置命令

| 命令 | 说明 |
|------|------|
| `config` | 显示当前配置 |
| `profile` | 列出所有 provider profiles |
| `profile add <name> <provider> [base-url]` | 添加新 profile |
| `use-profile <name>` | 切换到指定 profile |
| `model` | 列出所有 model presets |
| `model add <name> <model-id>` | 添加 model preset |
| `model remove <name>` | 删除 model preset |
| `use-model <preset>` | 使用 model preset |
| `init-dashscope <api-key>` | 初始化 DashScope 配置 |

## 安装与使用

### 前置要求

1. Rust 1.70+ (edition 2021)
2. Pi coding agent (需要先安装 Pi)

### 编译

```bash
cd pi-rust-client
cargo build --release
```

编译后的二进制文件位于 `target/release/pi-client.exe`

### 快速开始

**Windows CMD:**
```cmd
# 设置环境变量
set PI_PROVIDER=anthropic
set PI_MODEL=claude-sonnet-4-6
set PI_API_KEY=your-api-key

# 运行
pi-client chat
```

**DashScope (阿里百炼):**
```cmd
# 初始化 DashScope 配置
pi-client init-dashscope sk-your-dashscope-key

# 配置完成后
set PI_PROVIDER=anthropic
set PI_BASE_URL=https://coding.dashscope.aliyuncs.com/apps/anthropic
set PI_API_KEY=sk-your-dashscope-key
set PI_MODEL=qwen-coder-plus

pi-client chat
```

### 配置文件

配置存储在 `~/.pi-client/config.json`：

```json
{
  "default_profile": "dashscope",
  "profiles": [
    {
      "name": "anthropic",
      "provider": "anthropic",
      "description": "Anthropic Claude API"
    },
    {
      "name": "dashscope",
      "provider": "anthropic",
      "api": "anthropic-messages",
      "base_url": "https://coding.dashscope.aliyuncs.com/apps/anthropic",
      "api_key": "sk-sp-...",
      "default_model": "qwen-coder-plus",
      "description": "DashScope API (Anthropic compatible)"
    }
  ],
  "model_presets": [
    { "name": "opus", "model_id": "claude-opus-4-7" },
    { "name": "sonnet", "model_id": "claude-sonnet-4-6" },
    { "name": "haiku", "model_id": "claude-haiku-4-5-20251001" },
    { "name": "qwen", "model_id": "qwen-coder-plus" }
  ]
}
```

### 环境变量

| 变量 | 说明 |
|------|------|
| `PI_PROVIDER` | Provider 类型 (anthropic, openai 等) |
| `PI_MODEL` | 模型 ID 或模式 |
| `PI_API_KEY` | API 密钥 |
| `PI_BASE_URL` | 自定义 Base URL (兼容 API) |
| `PI_PATH` | Pi 可执行文件路径 |
| `PI_SCRIPT` | Pi TypeScript 脚本路径 |
| `PI_CWD` | Pi 运行目录 |
| `PI_WORKING_DIR` | 工作目录 |

## 项目结构

```
pi-rust-client/
├── src/
│   ├── main.rs       # CLI 入口和命令处理
│   ├── config.rs     # 配置管理模块
│   ├── rpc_client.rs # RPC 客户端实现
│   └── types.rs      # 类型定义
├── Cargo.toml        # Rust 项目配置
├── pi.cmd            # Windows 启动脚本
├── run-client.cmd    # DashScope 运行脚本
└── README.md         # 本文档
```

## 更新日志

### v0.1.0 (2026-05-15)
- ✅ 完成 RPC 客户端基础框架
- ✅ 实现 `model` 命令 (list/add/remove/use)
- ✅ 实现 `chat` 命令 (交互式和单次消息)
- ✅ 添加 ProviderProfile 的 `api` 和 `default_model` 字段
- ✅ DashScope (阿里百炼) 集成支持
- ✅ 配置文件系统 (~/.pi-client/config.json)

### 计划中的功能
- 🔲 交互式配置向导 (Provider 选择、字段验证)
- 🔲 API Key 格式验证
- 🔲 URL 可达性测试
- 🔲 配置加密存储
- 🔲 多会话管理
- 🔲 配置导入/导出

## 许可证

MIT License

## 联系方式

GitHub: https://github.com/wsbjwjt/pi-rust-client