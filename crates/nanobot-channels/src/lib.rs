//! Channel adapters for nanobot (e.g. CLI, Telegram, Feishu),
//! plus the `ChannelManager` that wires them to the `MessageBus`.

pub mod base;
pub mod cli;
pub mod error;
#[cfg(feature = "channel-feishu")]
pub mod feishu;
pub mod manager;
#[cfg(feature = "channel-telegram")]
pub mod telegram;

pub use error::{ChannelError, ChannelResult};
pub use manager::ChannelManager;
