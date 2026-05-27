# 面向新手的开源学习路径

本文档用于帮助初学贡献者快速建立“从哪里看、怎么看、先改哪里”的心智模型。  
内容只基于本仓库当前实现与文档结构。

## 1. 文档分层映射

| 本仓库文档主题 | 本仓库对应入口 | 建议先读什么 |
|---|---|---|
| `quick-start.md` | `docs/QUICK_START.md` | 安装、onboard、最小配置、常用命令 |
| `configuration.md` | `docs/QUICK_START.md` + `crates/nanobot-config/src/schema.rs` | 配置字段语义、默认值与校验 |
| `cli-reference.md` | `cargo run -- --help` + `cargo run -- agent --help` + `docs/QUICK_START.md` | CLI 子命令、参数、会话键 |
| `deployment.md` | `docs/ARCHITECTURE.md` + `docs/DEVELOPMENT.md` | gateway、channels、cron、heartbeat 的运行方式 |
| `memory.md` | `docs/MEMORY_SYSTEM_DESIGN.md` | memory/session/consolidation 的职责边界 |

## 2. 推荐阅读顺序（1-2 小时建立全局认知）

1. `docs/QUICK_START.md`：先把程序跑起来，理解“用户如何使用它”。  
2. `docs/ARCHITECTURE.md`：看运行时总览与核心数据流。  
3. `crates/nanobot/src/cli/mod.rs`：看入口命令如何驱动 runtime。  
4. `crates/nanobot/src/runtime/app.rs`：看组件装配（provider/bus/agent/tools）。  
5. `crates/nanobot-agent/src/loop_core.rs`：看核心消息处理与 ReAct 回路。  
6. `docs/DEVELOPMENT.md`：最后回到开发流程与测试命令。

## 3. 新手优先关注的三条主线

### A. CLI 到 Agent 的调用链

`cli::agent` -> `build_runtime` -> `AgentLoop::process_direct / run` -> `process_message`

### B. 工具调用链

`AgentLoop` -> `ReActExecutor` -> `ToolRegistry::execute` -> 具体工具（filesystem/web/shell/message/...）

### C. 会话与记忆链

`SessionManager`（历史读取）-> `ContextBuilder`（构造上下文）-> `save_turn`（落盘）-> memory/consolidation

## 4. 第一个可提交改动（建议）

如果你是首次贡献者，建议优先做这类改动：

- 补充关键模块注释（解释“为什么这样做”，而不是复述代码）。  
- 修正文档与源码不一致项（命令、路径、工具名、配置字段）。  
- 为一个具体行为增加单元测试（例如工具参数校验、会话保存边界）。

这样可以在不大改架构的前提下，快速熟悉代码并提交高质量 PR。

## 5. 注释风格约定（适合教学型开源项目）

- 优先解释**设计意图**（例如“为什么需要 session 锁”）。  
- 对复杂流程说明**阶段边界**（输入、处理、输出）。  
- 避免“变量赋值型注释”（代码已显而易见的内容）。  
- 注释要跟随代码演进，行为改变时同步更新。
