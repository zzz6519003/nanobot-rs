use async_trait::async_trait;

use crate::session::SessionManager;
use crate::types::provider::ChatMessage;

#[async_trait]
pub trait ContextProvider: Send + Sync {
    /// Builds the message history for the LLM request.
    ///
    /// This method constructs the complete message array including:
    /// - System prompt
    /// - Historical messages
    /// - Current user message with runtime context
    ///
    /// # Arguments
    ///
    /// * `session_manager` - Session manager for retrieving memory context
    /// * `session_key` - Current session key for memory lookup
    /// * `history` - Previous conversation messages
    /// * `current_message` - The new user message text
    /// * `media` - Optional media attachments (URLs or paths)
    /// * `channel` - Optional channel name for runtime context
    /// * `chat_id` - Optional chat ID for runtime context
    ///
    /// # Returns
    ///
    /// Returns a vector of chat messages ready for the LLM API.
    async fn build_messages(
        &self,
        session_manager: &SessionManager,
        session_key: &str,
        history: Vec<ChatMessage>,
        current_message: &str,
        media: Option<&[String]>,
        channel: Option<&str>,
        chat_id: Option<&str>,
    ) -> Vec<ChatMessage>;
}
