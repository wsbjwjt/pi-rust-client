@echo off
REM Pi Rust Client - DashScope Configuration
REM Alibaba DashScope API (Anthropic compatible)

set PI_SCRIPT=packages/coding-agent/src/cli.ts
set PI_CWD=D:\aicodes\pi
set PI_PROVIDER=anthropic
set PI_MODEL=qwen-coder-plus
set PI_API_KEY=sk-sp-af0fd9074a734640b63f2c91aaa23b40
set PI_BASE_URL=https://coding.dashscope.aliyuncs.com/apps/anthropic

.\target\release\pi-client.exe %*