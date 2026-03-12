# LLMProvider 统一流式响应设计

## 设计目标

1. **统一抽象层**：上层代码（Agent Loop）只依赖统一接口，不关心底层 provider 差异
2. **可扩展性**：轻松添加新的 provider（OpenAI、Anthropic、Gemini 等）
3. **向后兼容**：现有非流式代码无需修改
4. **类型安全**：编译期保证正确性

## 架构设计

```
┌─────────────────────────────────────────┐
│         Agent Loop / Application        │
│    (只依赖 LLMProvider trait)            │
└─────────────────┬───────────────────────┘
                  │
                  │ chat_stream() -> StreamResponse
                  │
┌─────────────────▼───────────────────────┐
│      Unified Streaming Abstraction      │
│  - StreamEvent (统一事件类型)            │
│  - StreamResponse (统一流类型)           │
│  - StreamAccumulator (状态管理)         │
└─────────────────┬───────────────────────┘
                  │
        ┌─────────┴─────────┐
        │                   │
┌───────▼────────┐  ┌──────▼──────────┐
│  SSE Adapter   │  │  OpenAI Adapter │
│  (Anthropic)   │  │  (OpenAI/Azure) │
└───────┬────────┘  └──────┬──────────┘
        │                   │
┌───────▼────────┐  ┌──────▼──────────┐
│ Anthropic API  │  │   OpenAI API    │
│  (SSE format)  │  │ (data: [DONE])  │
└────────────────┘  └─────────────────┘
```

## 核心类型定义

### 1. 统一事件类型

```rust
// src/provider/streaming/events.rs

/// 统一的流式事件类型
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// 文本内容增量
    TextDelta {
        content: String,
        /// 内容块索引（用于多个内容块的场景）
        index: usize,
    },

    /// 思考内容增量（reasoning/thinking）
    ThinkingDelta {
        content: String,
    },

    /// 工具调用开始
    ToolCallStart {
        id: String,
        name: String,
        index: usize,
    },

    /// 工具调用参数增量
    ToolCallArgumentsDelta {
        id: String,
        arguments_json: String,
        index: usize,
    },

    /// 工具调用结束
    ToolCallEnd {
        id: String,
        index: usize,
    },

    /// 使用统计更新
    UsageUpdate {
        input_tokens: Option<i32>,
        output_tokens: Option<i32>,
        total_tokens: Option<i32>,
    },

    /// 完成原因更新
    FinishReasonUpdate {
        reason: String,
    },

    /// 流结束（包含完整的累积响应）
    Done {
        response: LLMResponse,
    },

    /// 错误事件
    Error {
        message: String,
    },
}

/// 统一的流式响应类型
pub type StreamResponse = Pin<Box<dyn Stream<Item = Result<StreamEvent, StreamError>> + Send>>;

/// 流式错误类型
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Stream interrupted")]
    Interrupted,
}
```

### 2. 流式累积器（状态管理）

```rust
// src/provider/streaming/accumulator.rs

/// 累积流式事件，构建完整响应
pub struct StreamAccumulator {
    content_blocks: Vec<String>,
    thinking_blocks: Vec<String>,
    tool_calls: HashMap<String, ToolCallBuilder>,
    usage: UsageStats,
    finish_reason: Option<String>,
}

struct ToolCallBuilder {
    id: String,
    name: String,
    arguments_json: String,
}

impl StreamAccumulator {
    pub fn new() -> Self {
        Self {
            content_blocks: Vec::new(),
            thinking_blocks: Vec::new(),
            tool_calls: HashMap::new(),
            usage: UsageStats::default(),
            finish_reason: None,
        }
    }

    /// 处理流式事件，更新内部状态
    pub fn process_event(&mut self, event: &StreamEvent) {
        match event {
            StreamEvent::TextDelta { content, index } => {
                self.ensure_content_block(*index);
                self.content_blocks[*index].push_str(content);
            }
            StreamEvent::ThinkingDelta { content } => {
                if self.thinking_blocks.is_empty() {
                    self.thinking_blocks.push(String::new());
                }
                self.thinking_blocks.last_mut().unwrap().push_str(content);
            }
            StreamEvent::ToolCallStart { id, name, .. } => {
                self.tool_calls.insert(id.clone(), ToolCallBuilder {
                    id: id.clone(),
                    name: name.clone(),
                    arguments_json: String::new(),
                });
            }
            StreamEvent::ToolCallArgumentsDelta { id, arguments_json, .. } => {
                if let Some(builder) = self.tool_calls.get_mut(id) {
                    builder.arguments_json.push_str(arguments_json);
                }
            }
            StreamEvent::UsageUpdate { input_tokens, output_tokens, total_tokens } => {
                if let Some(tokens) = input_tokens {
                    self.usage.prompt_tokens = Some(*tokens);
                }
                if let Some(tokens) = output_tokens {
                    self.usage.completion_tokens = Some(*tokens);
                }
                if let Some(tokens) = total_tokens {
                    self.usage.total_tokens = Some(*tokens);
                }
            }
            StreamEvent::FinishReasonUpdate { reason } => {
                self.finish_reason = Some(reason.clone());
            }
            _ => {}
        }
    }

    /// 构建最终的 LLMResponse
    pub fn build_response(self) -> LLMResponse {
        let content = if self.content_blocks.is_empty() {
            None
        } else {
            Some(self.content_blocks.join("\n\n"))
        };

        let thinking_blocks = if self.thinking_blocks.is_empty() {
            None
        } else {
            Some(self.thinking_blocks)
        };

        let tool_calls = self.tool_calls.into_values()
            .map(|builder| ToolCallRequest {
                id: builder.id,
                name: builder.name.into(),
                arguments_json: builder.arguments_json,
            })
            .collect();

        LLMResponse {
            content,
            tool_calls,
            finish_reason: self.finish_reason.unwrap_or_else(|| "stop".to_string()),
            usage: self.usage,
            reasoning_content: None,
            thinking_blocks,
        }
    }

    fn ensure_content_block(&mut self, index: usize) {
        while self.content_blocks.len() <= index {
            self.content_blocks.push(String::new());
        }
    }
}
```

### 3. Provider Adapter Trait

```rust
// src/provider/streaming/adapter.rs

/// Provider 特定的流式适配器
#[async_trait]
pub trait StreamAdapter: Send + Sync {
    /// 将原始字节流转换为统一的 StreamEvent
    async fn adapt_stream(
        &self,
        response: reqwest::Response,
    ) -> Result<StreamResponse, StreamError>;
}
```

### 4. SSE Adapter (Anthropic)

```rust
// src/provider/streaming/sse_adapter.rs

use bytes::Bytes;
use futures::StreamExt;

/// SSE 格式适配器（用于 Anthropic）
pub struct SseAdapter;

#[async_trait]
impl StreamAdapter for SseAdapter {
    async fn adapt_stream(
        &self,
        response: reqwest::Response,
    ) -> Result<StreamResponse, StreamError> {
        let bytes_stream = response.bytes_stream();

        let event_stream = bytes_stream
            .map(|chunk_result| {
                chunk_result.map_err(|e| StreamError::Network(e.to_string()))
            })
            .scan(SseParser::new(), |parser, chunk_result| {
                async move {
                    match chunk_result {
                        Ok(chunk) => Some(parser.parse_chunk(chunk)),
                        Err(e) => Some(vec![Err(e)]),
                    }
                }
            })
            .flat_map(futures::stream::iter);

        Ok(Box::pin(event_stream))
    }
}

/// SSE 解析器（状态机）
struct SseParser {
    buffer: String,
    current_event: Option<SseEvent>,
}

struct SseEvent {
    event_type: Option<String>,
    data: String,
}

impl SseParser {
    fn new() -> Self {
        Self {
            buffer: String::new(),
            current_event: None,
        }
    }

    fn parse_chunk(&mut self, chunk: Bytes) -> Vec<Result<StreamEvent, StreamError>> {
        self.buffer.push_str(&String::from_utf8_lossy(&chunk));

        let mut events = Vec::new();

        // 按行分割
        while let Some(line_end) = self.buffer.find('\n') {
            let line = self.buffer[..line_end].trim_end_matches('\r');
            self.buffer.drain(..=line_end);

            if line.is_empty() {
                // 空行表示事件结束
                if let Some(sse_event) = self.current_event.take() {
                    if let Some(stream_event) = self.parse_sse_event(sse_event) {
                        events.push(stream_event);
                    }
                }
            } else if let Some(data) = line.strip_prefix("data: ") {
                // 数据行
                let event = self.current_event.get_or_insert_with(|| SseEvent {
                    event_type: None,
                    data: String::new(),
                });
                if !event.data.is_empty() {
                    event.data.push('\n');
                }
                event.data.push_str(data);
            } else if let Some(event_type) = line.strip_prefix("event: ") {
                // 事件类型行
                let event = self.current_event.get_or_insert_with(|| SseEvent {
                    event_type: None,
                    data: String::new(),
                });
                event.event_type = Some(event_type.to_string());
            }
            // 忽略其他行（id:, retry: 等）
        }

        events
    }

    fn parse_sse_event(&self, sse_event: SseEvent) -> Option<Result<StreamEvent, StreamError>> {
        // 解析 JSON data
        let value: serde_json::Value = match serde_json::from_str(&sse_event.data) {
            Ok(v) => v,
            Err(e) => return Some(Err(StreamError::Parse(e.to_string()))),
        };

        let event_type = sse_event.event_type.as_deref().unwrap_or("message");

        match event_type {
            "message_start" => {
                // Anthropic: 消息开始
                None // 不需要发送事件
            }
            "content_block_start" => {
                // Anthropic: 内容块开始
                let index = value["index"].as_u64().unwrap_or(0) as usize;
                let block_type = value["content_block"]["type"].as_str()?;

                match block_type {
                    "tool_use" => {
                        let id = value["content_block"]["id"].as_str()?.to_string();
                        let name = value["content_block"]["name"].as_str()?.to_string();
                        Some(Ok(StreamEvent::ToolCallStart { id, name, index }))
                    }
                    _ => None
                }
            }
            "content_block_delta" => {
                // Anthropic: 内容增量
                let index = value["index"].as_u64().unwrap_or(0) as usize;
                let delta = &value["delta"];
                let delta_type = delta["type"].as_str()?;

                match delta_type {
                    "text_delta" => {
                        let content = delta["text"].as_str()?.to_string();
                        Some(Ok(StreamEvent::TextDelta { content, index }))
                    }
                    "input_json_delta" => {
                        // 工具调用参数增量
                        let arguments_json = delta["partial_json"].as_str()?.to_string();
                        // 需要从上下文获取 tool call id
                        // 这里简化处理，实际需要维护状态
                        None
                    }
                    _ => None
                }
            }
            "content_block_stop" => {
                // Anthropic: 内容块结束
                let index = value["index"].as_u64().unwrap_or(0) as usize;
                // 可以发送 ToolCallEnd 事件
                None
            }
            "message_delta" => {
                // Anthropic: 消息元数据更新
                if let Some(usage) = value["usage"].as_object() {
                    let output_tokens = usage["output_tokens"].as_i64().map(|v| v as i32);
                    return Some(Ok(StreamEvent::UsageUpdate {
                        input_tokens: None,
                        output_tokens,
                        total_tokens: None,
                    }));
                }

                if let Some(reason) = value["delta"]["stop_reason"].as_str() {
                    return Some(Ok(StreamEvent::FinishReasonUpdate {
                        reason: reason.to_string(),
                    }));
                }

                None
            }
            "message_stop" => {
                // Anthropic: 消息结束
                // 这里不发送 Done 事件，由上层累积器处理
                None
            }
            "error" => {
                let message = value["error"]["message"].as_str()
                    .unwrap_or("Unknown error")
                    .to_string();
                Some(Err(StreamError::Provider(message)))
            }
            _ => None
        }
    }
}
```

### 5. OpenAI Adapter

```rust
// src/provider/streaming/openai_adapter.rs

/// OpenAI 格式适配器
pub struct OpenAiAdapter;

#[async_trait]
impl StreamAdapter for OpenAiAdapter {
    async fn adapt_stream(
        &self,
        response: reqwest::Response,
    ) -> Result<StreamResponse, StreamError> {
        let bytes_stream = response.bytes_stream();

        let event_stream = bytes_stream
            .map(|chunk_result| {
                chunk_result.map_err(|e| StreamError::Network(e.to_string()))
            })
            .scan(OpenAiParser::new(), |parser, chunk_result| {
                async move {
                    match chunk_result {
                        Ok(chunk) => Some(parser.parse_chunk(chunk)),
                        Err(e) => Some(vec![Err(e)]),
                    }
                }
            })
            .flat_map(futures::stream::iter);

        Ok(Box::pin(event_stream))
    }
}

struct OpenAiParser {
    buffer: String,
}

impl OpenAiParser {
    fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    fn parse_chunk(&mut self, chunk: Bytes) -> Vec<Result<StreamEvent, StreamError>> {
        self.buffer.push_str(&String::from_utf8_lossy(&chunk));

        let mut events = Vec::new();

        // OpenAI 格式: data: {...}\n\n
        while let Some(data_start) = self.buffer.find("data: ") {
            let data_content_start = data_start + 6;

            // 查找行结束
            if let Some(line_end) = self.buffer[data_content_start..].find('\n') {
                let line_end_abs = data_content_start + line_end;
                let data_line = self.buffer[data_content_start..line_end_abs].trim();

                // 移除已处理的部分
                self.buffer.drain(..=line_end_abs);

                // 检查是否是结束标记
                if data_line == "[DONE]" {
                    // OpenAI 流结束标记
                    continue;
                }

                // 解析 JSON
                match serde_json::from_str::<serde_json::Value>(data_line) {
                    Ok(value) => {
                        if let Some(event) = self.parse_openai_chunk(value) {
                            events.push(event);
                        }
                    }
                    Err(e) => {
                        events.push(Err(StreamError::Parse(e.to_string())));
                    }
                }
            } else {
                // 不完整的行，等待更多数据
                break;
            }
        }

        events
    }

    fn parse_openai_chunk(&self, value: serde_json::Value) -> Option<Result<StreamEvent, StreamError>> {
        let choices = value["choices"].as_array()?;
        let choice = choices.first()?;
        let delta = &choice["delta"];
        let index = choice["index"].as_u64().unwrap_or(0) as usize;

        // 文本内容
        if let Some(content) = delta["content"].as_str() {
            return Some(Ok(StreamEvent::TextDelta {
                content: content.to_string(),
                index,
            }));
        }

        // 工具调用
        if let Some(tool_calls) = delta["tool_calls"].as_array() {
            for tool_call in tool_calls {
                let tc_index = tool_call["index"].as_u64().unwrap_or(0) as usize;

                if let Some(id) = tool_call["id"].as_str() {
                    let name = tool_call["function"]["name"].as_str()?.to_string();
                    return Some(Ok(StreamEvent::ToolCallStart {
                        id: id.to_string(),
                        name,
                        index: tc_index,
                    }));
                }

                if let Some(arguments) = tool_call["function"]["arguments"].as_str() {
                    // 需要从上下文获取 id
                    return Some(Ok(StreamEvent::ToolCallArgumentsDelta {
                        id: String::new(), // 需要状态管理
                        arguments_json: arguments.to_string(),
                        index: tc_index,
                    }));
                }
            }
        }

        // 完成原因
        if let Some(reason) = choice["finish_reason"].as_str() {
            return Some(Ok(StreamEvent::FinishReasonUpdate {
                reason: reason.to_string(),
            }));
        }

        None
    }
}
```

### 6. LLMProvider Trait 更新

```rust
// src/provider/base.rs

use crate::provider::streaming::{StreamResponse, StreamError};

#[async_trait]
pub trait LLMProvider: Send + Sync {
    fn default_model(&self) -> &str;

    /// 非流式调用（保持向后兼容）
    async fn chat(&self, req: ChatRequest) -> LLMResponse;

    /// 流式调用（统一接口）
    async fn chat_stream(&self, req: ChatRequest) -> Result<StreamResponse, StreamError> {
        // 默认实现：调用非流式接口，包装成单事件流
        let response = self.chat(req).await;
        Ok(Box::pin(futures::stream::once(async move {
            Ok(StreamEvent::Done { response })
        })))
    }

    async fn reset_session(&self, _session_key: &SessionKey) {}

    async fn close(&self) {}
}
```

### 7. Provider 实现

```rust
// src/provider/anthropic.rs

use crate::provider::streaming::{SseAdapter, StreamAdapter, StreamResponse, StreamError};

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn chat_stream(&self, req: ChatRequest) -> Result<StreamResponse, StreamError> {
        let model = req.model.as_deref().unwrap_or(&self.default_model);
        let mut payload = self.build_payload(model.to_string(), req);

        // 启用流式
        payload.stream = Some(true);

        let endpoint = self.endpoint();
        let response = self.send_request(&endpoint, &serde_json::to_value(payload).unwrap())
            .await
            .map_err(|e| StreamError::Network(e))?;

        // 使用 SSE 适配器
        let adapter = SseAdapter;
        adapter.adapt_stream(response).await
    }
}

// src/provider/openai_compat.rs

use crate::provider::streaming::{OpenAiAdapter, StreamAdapter, StreamResponse, StreamError};

#[async_trait]
impl LLMProvider for OpenAICompatProvider {
    async fn chat_stream(&self, req: ChatRequest) -> Result<StreamResponse, StreamError> {
        let model = self.resolve_model(req.model.as_deref().unwrap_or(&self.default_model));
        let mut payload = self.build_responses_payload(model, req);

        // 启用流式
        payload.stream = Some(true);

        let endpoint = self.endpoint();
        let response = self.send_request_with_proxy_fallback(&endpoint, &serde_json::to_value(payload).unwrap())
            .await
            .map_err(|e| StreamError::Network(e))?;

        // 使用 OpenAI 适配器
        let adapter = OpenAiAdapter;
        adapter.adapt_stream(response).await
    }
}
```

### 8. Agent Loop 使用

```rust
// src/agent/loop_core.rs

impl AgentLoop {
    /// 流式执行一轮对话
    pub async fn run_turn_streaming(&self, session_key: &SessionKey) -> Result<()> {
        let req = self.build_chat_request(session_key).await?;

        // 获取流式响应
        let mut stream = self.provider.chat_stream(req).await
            .map_err(|e| NanobotError::provider(format!("Stream error: {}", e)))?;

        // 累积器
        let mut accumulator = StreamAccumulator::new();
        let mut last_text_len = 0;

        // 处理流式事件
        while let Some(event_result) = stream.next().await {
            let event = event_result
                .map_err(|e| NanobotError::provider(format!("Stream event error: {}", e)))?;

            match &event {
                StreamEvent::TextDelta { content, .. } => {
                    // 实时输出增量文本
                    self.bus.publish_outbound(OutboundMessage {
                        channel: session.channel.clone(),
                        chat_id: session.chat_id.clone(),
                        content: content.clone(),
                        is_partial: true,
                        ..Default::default()
                    })?;
                }
                StreamEvent::Done { response } => {
                    // 流结束，保存完整响应
                    self.sessions.append_assistant_message(session_key, response.clone()).await?;
                    return Ok(());
                }
                StreamEvent::Error { message } => {
                    return Err(NanobotError::provider(message.clone()));
                }
                _ => {}
            }

            // 更新累积器
            accumulator.process_event(&event);
        }

        // 如果流没有发送 Done 事件，手动构建响应
        let response = accumulator.build_response();
        self.sessions.append_assistant_message(session_key, response).await?;

        Ok(())
    }

    /// 非流式执行（保持向后兼容）
    pub async fn run_turn(&self, session_key: &SessionKey) -> Result<()> {
        let req = self.build_chat_request(session_key).await?;
        let response = self.provider.chat(req).await;
        self.sessions.append_assistant_message(session_key, response).await?;
        Ok(())
    }
}
```

## 模块结构

```
src/provider/
├── base.rs                    # LLMProvider trait
├── streaming/
│   ├── mod.rs                 # 导出公共接口
│   ├── events.rs              # StreamEvent, StreamResponse, StreamError
│   ├── accumulator.rs         # StreamAccumulator
│   ├── adapter.rs             # StreamAdapter trait
│   ├── sse_adapter.rs         # SseAdapter (Anthropic)
│   └── openai_adapter.rs      # OpenAiAdapter (OpenAI)
├── anthropic.rs               # AnthropicProvider
├── openai_compat.rs           # OpenAICompatProvider
└── ...
```

## 实现阶段

### Phase 1: 基础设施
- [ ] 创建 `src/provider/streaming/` 模块
- [ ] 定义 `StreamEvent`, `StreamError`, `StreamResponse`
- [ ] 实现 `StreamAccumulator`
- [ ] 定义 `StreamAdapter` trait

### Phase 2: SSE Adapter (Anthropic)
- [ ] 实现 `SseParser` 状态机
- [ ] 实现 `SseAdapter`
- [ ] 添加单元测试（mock SSE 响应）

### Phase 3: OpenAI Adapter
- [ ] 实现 `OpenAiParser`
- [ ] 实现 `OpenAiAdapter`
- [ ] 添加单元测试

### Phase 4: Provider 集成
- [ ] 在 `LLMProvider` trait 中添加 `chat_stream()` 方法
- [ ] `AnthropicProvider` 实现 `chat_stream()`
- [ ] `OpenAICompatProvider` 实现 `chat_stream()`

### Phase 5: Agent Loop 集成
- [ ] 实现 `run_turn_streaming()`
- [ ] 添加配置选项（是否启用流式）
- [ ] 集成测试

### Phase 6: 优化
- [ ] 错误恢复（网络中断）
- [ ] 性能优化（减少分配）
- [ ] 监控指标

## 优势

1. **统一抽象**：上层代码不关心 provider 差异
2. **易于扩展**：添加新 provider 只需实现 `StreamAdapter`
3. **类型安全**：编译期保证事件类型正确
4. **状态管理**：`StreamAccumulator` 统一处理累积逻辑
5. **向后兼容**：现有代码无需修改

## 测试策略

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sse_adapter_parses_text_delta() {
        let mock_response = mock_sse_response(vec![
            "event: content_block_delta\n",
            "data: {\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n",
            "\n",
        ]);

        let adapter = SseAdapter;
        let mut stream = adapter.adapt_stream(mock_response).await.unwrap();

        let event = stream.next().await.unwrap().unwrap();
        assert!(matches!(event, StreamEvent::TextDelta { content, .. } if content == "Hello"));
    }

    #[tokio::test]
    async fn test_accumulator_builds_response() {
        let mut acc = StreamAccumulator::new();

        acc.process_event(&StreamEvent::TextDelta {
            content: "Hello".to_string(),
            index: 0,
        });
        acc.process_event(&StreamEvent::TextDelta {
            content: " world".to_string(),
            index: 0,
        });

        let response = acc.build_response();
        assert_eq!(response.content.as_deref(), Some("Hello world"));
    }
}
```

## 参考资料

- [Anthropic Streaming Messages](https://docs.anthropic.com/claude/reference/messages-streaming)
- [OpenAI Streaming](https://platform.openai.com/docs/api-reference/streaming)
- [tokio-stream](https://docs.rs/tokio-stream)
- [futures::Stream](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)
