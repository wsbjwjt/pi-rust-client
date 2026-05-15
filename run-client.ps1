# Pi Rust Client - DashScope Configuration
# Alibaba DashScope API (Anthropic compatible)

$env:PI_SCRIPT = "packages/coding-agent/src/cli.ts"
$env:PI_CWD = "D:\aicodes\pi"
$env:PI_PROVIDER = "anthropic"
$env:PI_MODEL = "qwen3.6-plus"
$env:PI_API_KEY = $env:DASHSCOPE_API_KEY
$env:PI_BASE_URL = "https://coding.dashscope.aliyuncs.com/apps/anthropic"

& ".\target\release\pi-client.exe" $args