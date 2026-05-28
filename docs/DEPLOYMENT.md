# 部署与运行说明

本文档描述当前源码下可用的运行模式与部署建议。

## 1. 运行模式

当前主要有两种模式：

1. `agent`：单轮 / 交互式本地会话
2. `gateway`：长驻进程，启动 channels + cron + heartbeat + agent loop

## 2. 本地开发运行

```bash
cargo run -- onboard
cargo run -- agent -m "hello"
cargo run -- gateway
```

建议先执行 `onboard`，确保配置与 workspace 模板已就绪。

## 3. gateway 模式组件

`gateway` 启动后会组装 `RuntimeBundle`，并启动以下组件：

- `MessageBus`：统一收发消息
- `AgentLoop`：主执行循环
- `ChannelManager`：管理 `cli/telegram/discord/feishu(lark)` 适配器（其中 discord 当前为占位）
- `CronService`：定时任务调度
- `HeartbeatService`：按 `HEARTBEAT.md` 决策是否执行提醒任务

## 4. 目录与持久化

默认目录：

- 配置：`~/.nanobot/config.json`
- 工作区：`~/.nanobot/workspace`
- 会话：`~/.nanobot/workspace/sessions/`
- 记忆：`~/.nanobot/workspace/memory/`
- Cron 数据：数据目录下 `cron/jobs.json`

## 5. 通道部署要点

### CLI

- 默认内置，无需额外配置。

### Telegram

至少配置：

```json
{
  "channels": {
    "telegram": {
      "enabled": true,
      "allowFrom": ["*"],
      "token": "<bot-token>"
    }
  }
}
```

可选：

- `apiBase`
- `receiveAck`

注意 `allowFrom` 规则：

- 必须非空
- `*` 与显式 ID 不能并存

### Feishu / Lark（推荐应用模式）

至少配置：

```json
{
  "channels": {
    "feishu": {
      "enabled": true,
      "allowFrom": ["*"],
      "appId": "<feishu-app-id>",
      "appSecret": "<feishu-app-secret>",
      "verifyToken": "<event-subscription-token>"
    }
  }
}
```

可选：

- `webhook` / `webhookUrl` / `url` / `botKey`（Webhook 出站）
- `secret` / `signSecret`
- `apiBase`（仅配 `botKey` 时使用）
- `verifyToken`
- `callbackListen`（默认 `0.0.0.0:19820`）
- `callbackPath`（默认 `/feishu/events`）
- `eventEnabled`（默认 `true`）

## 6. 生产运行建议

- 使用进程管理器（systemd / supervisor / launchd）托管 `nanobot gateway`
- 配置日志采集（建议按 `RUST_LOG` 设定级别）
- 将敏感字段改为环境变量占位（`{{ENV_VAR}}`）
- 为 `tools.exec` 设置合理 `timeout`
- 视需要开启 `tools.restrictToWorkspace`

## 7. 运行与停止

### 启动

```bash
nanobot gateway
```

### 停止

- 前台运行：`Ctrl+C`
- 由外部进程管理器发送终止信号

正常停止流程会依次关闭 channels、agent、heartbeat、cron，以及 provider/MCP 连接。

## 8. 常见问题排查

### gateway 已启动但无消息

- 检查通道 `enabled` 与 `allowFrom`
- 检查 Telegram token 是否有效
- 检查 provider 凭证是否存在

### 心跳不触发

- 检查 `gateway.heartbeat.enabled`
- 检查 `gateway.heartbeat.intervalS`
- 检查 workspace 下是否存在非空 `HEARTBEAT.md`

### 定时任务未回调

- 检查 cron 数据文件是否可写
- 检查 gateway 进程是否持续运行
- 检查任务目标 `channel/chat_id` 是否可达

## 9. GitHub Release 发行

仓库已经配置好 GitHub Actions 自动发行，目标平台包括：

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `x86_64-pc-windows-msvc`

### 触发方式

- 推送 tag：`git tag v0.1.0 && git push origin v0.1.0`
- 手动触发：GitHub Actions 页面选择 `Release` workflow 后执行 `Run workflow`

行为区别：

- `tag push`：构建三平台产物，并创建对应的 GitHub Release
- `workflow_dispatch`：只构建并上传 workflow artifacts，方便先验证打包是否正常

### GitHub 仓库配置要求

- Actions 必须启用
- Workflow 权限需要允许写入 `contents`
- 发布账号需要有创建 release 和上传 artifact 的权限

当前 `release.yml` 已显式声明：

```yaml
permissions:
  contents: write
```

这意味着如果组织策略把 `GITHUB_TOKEN` 默认权限限制得更严格，仍需要仓库管理员确保该 workflow 允许获得 `Contents: Read and write`。

### 发行产物命名

发布包命名规则：

- Linux/macOS：`nanobot-<version>-<target>.tar.gz`
- Windows：`nanobot-<version>-<target>.zip`

例如：

- `nanobot-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- `nanobot-v0.1.0-x86_64-apple-darwin.tar.gz`
- `nanobot-v0.1.0-x86_64-pc-windows-msvc.zip`

### 标准发版步骤

建议按下面顺序执行：

1. 确认工作区已经整理完毕，并准备好要发布的提交
2. 本地执行校验
3. 推送代码到默认分支
4. 创建并推送语义化版本 tag
5. 等待 `Release` workflow 完成
6. 检查 GitHub Release 页面中的三平台产物是否齐全
7. 抽样下载一个包，验证二进制是否可运行

推荐命令：

```bash
just ci
just e2e
git push origin main
git tag v0.1.0
git push origin v0.1.0
```

如果希望先验证打包，而不正式创建 release，可以先在 GitHub Actions 页面手动触发 `Release` workflow。

### 发版前检查清单

- `Cargo.toml` / `Cargo.lock` 已处于预期状态
- `just ci` 已通过
- `just e2e` 已通过
- 文档中与安装、命令、配置相关的内容已同步
- 准备发布的提交已经推送到远端

### 发版后检查清单

- GitHub Release 标题与 tag 对应
- Linux/macOS/Windows 三个平台产物均已上传
- Release Notes 自动生成内容无明显错误
- 下载包后可看到 `nanobot` 或 `nanobot.exe`

### 发版失败时的处理方式

如果只是 workflow 临时失败：

- 修复代码或 workflow
- 重新推送提交
- 删除错误 tag
- 重新打同名 tag，或使用下一个版本号重新打 tag

如果 GitHub Release 已创建但内容错误，建议顺序如下：

1. 删除 GitHub Release
2. 删除远端 tag：`git push origin :refs/tags/v0.1.0`
3. 删除本地 tag：`git tag -d v0.1.0`
4. 修复问题后重新打 tag 并推送

不建议在 release 已经对外可见后直接覆盖同名产物而不变更 tag，这会让用户拿到“同名不同内容”的包。

### 常见失败点

- `contents: write` 权限不足，导致 release 创建失败
- Windows runner 打包时路径或二进制扩展名处理错误
- 测试通过但 `release` 二进制未正确生成，通常是 `--package` / `--bin` 指向错误
- 使用 `workflow_dispatch` 后误以为已经正式发版；实际上它默认只上传 workflow artifact，不创建 GitHub Release
