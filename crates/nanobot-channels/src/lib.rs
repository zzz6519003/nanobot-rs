//! Channel adapters for nanobot: CLI, Telegram, and placeholder adapters,
//! plus the `ChannelManager` that wires them to the `MessageBus`.

pub mod base;
pub mod cli;
pub mod error;
#[cfg(feature = "channel-feishu")]
pub mod feishu;
pub mod manager;
pub mod placeholder;
#[cfg(feature = "channel-telegram")]
pub mod telegram;

pub use error::{ChannelError, ChannelResult};
pub use manager::ChannelManager;
