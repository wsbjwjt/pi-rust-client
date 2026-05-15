# CLAUDE.md - Pi Rust Client 项目说明

## 项目概述

这是一个**研究性项目**，探索 Rust + Pi agents 的集成方案。目标是构建一个轻量级 RPC 客户端，通过 JSONL 协议与 Pi coding agent 进行通信。

## 项目结构

```
pi-rust-client/
├── src/
│   ├── main.rs       # CLI 入口，命令路由
│   ├── config.rs     # 配置管理 (PiConfig, ProviderProfile, ModelPreset)
│   ├── rpc_client.rs # RPC 客户端实现 (PiClient, PiClientConfig)
│   └── types.rs      # 类型定义 (RPC 消息、响应)
├── Cargo.toml        # 依赖: serde, serde_json, tokio, anyhow, tracing
└── ~/.pi-client/config.json  # 用户配置文件
```

## 编码规范

### Rust 约定
- 使用 `anyhow::Result` 进行错误处理
- 使用 `tracing` 进行日志记录 (默认 INFO 级别)
- 遵循 Rust 标准命名: 函数用 snake_case，类型用 CamelCase
- 仅在开发模块中使用 `#![allow(dead_code)]`

### CLI 命令模式
命令在 `main.rs` 中遵循统一模式：
```rust
match command.as_str() {
    "command-name" => {
        if args.len() < N {
            eprintln!("Usage: command-name <args>");
            return Ok(());
        }
        run_command(&args[N..])?;
    }
}
```

### 配置结构
- `ProviderProfile`: name, provider, api, base_url, api_key, default_model, description
- `ModelPreset`: name, model_id, description
- 配置存储在 `~/.pi-client/config.json` (JSON 格式)
- 使用 `skip_serializing_if = "Option::is_none"` 处理可选字段

## 开发流程

### 编译
```bash
cargo build --release
```

### 测试命令
```bash
# 测试 model 列表
pi-client model

# 测试 model 添加
pi-client model add test-model gpt-4-test

# 测试 DashScope 初始化
pi-client init-dashscope sk-test-key
```

### Git 工作流
- 主分支: `main`
- 提交格式: `feat/fix/refactor: 描述`
- 推送到: `git@github.com:wsbjwjt/pi-rust-client.git`

## 环境变量

运行时的关键环境变量：
- `PI_PROVIDER` - Provider 类型 (anthropic, openai)
- `PI_MODEL` - 使用的模型 ID
- `PI_API_KEY` - API 认证密钥
- `PI_BASE_URL` - 自定义端点 URL (用于 DashScope、代理)

## 修改前必读文件

1. **src/main.rs** - 添加新命令前先了解命令路由
2. **src/config.rs** - 修改 profiles 前先了解配置结构
3. **src/rpc_client.rs** - 添加新方法前先了解 RPC 协议

## 禁止的操作

- 不要运行 `cargo publish` (这是研究项目，不发布到 crates.io)
- 不要手动修改 `Cargo.lock`
- 不要提交 `target/` 目录

## 后续功能规划

Phase 2: 交互式配置向导
Phase 3: API 验证和测试
Phase 4: 多会话管理

## Skill 路由规则

在此项目工作时，调用相关 skill：
- 架构变更 → 调用 `/plan-eng-review`
- Bug/错误 → 调用 `/investigate`
- 发布/部署 → 调用 `/ship`