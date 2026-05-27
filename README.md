# nanobot

`nanobot` 是一个 Rust 实现的本地 Agent 运行时，提供：

- CLI 单轮与交互式对话
- Gateway 模式与渠道分发
- 原生 `Anthropic Messages` / `OpenAI Responses` / 自定义 `Responses-compatible` provider
- 文件、Shell、Web、消息、子代理、Cron、MCP、ACP 等工具能力
- 本地会话持久化、记忆、心跳与定时任务

## 文档

- `docs/README.md`：文档索引与维护原则
- `docs/QUICK_START.md`：安装、配置与日常使用
- `docs/ARCHITECTURE.md`：当前系统架构与设计边界
- `docs/DEVELOPMENT.md`：开发、测试与调试流程
- `docs/OPEN_SOURCE_LEARNING_PATH.md`：面向新手贡献者的阅读与上手路径
- `docs/CONFIGURATION.md`：配置字段与默认行为参考
- `docs/CLI_REFERENCE.md`：命令行与聊天内建命令参考
- `docs/DEPLOYMENT.md`：gateway 部署与运行说明

## 快速开始

```bash
cargo run -- onboard
$EDITOR ~/.nanobot/config.json
cargo run -- agent -m "Hello!"
```

## CI / Release

仓库已配置 GitHub Actions：

- `CI`：在 Linux / macOS / Windows 上执行格式、lint、测试
- `Release`：在 Linux / macOS / Windows 上构建发行包；推送 `v*` tag 时自动创建 GitHub Release

具体发布说明见 `docs/DEPLOYMENT.md`。

## 开发命令

仓库包含 `justfile`：

```bash
just                 # 查看可用命令
just ci              # 本地 CI 校验
just e2e             # 离线端到端验证
just agent -m "hi"   # 启动 agent
```
