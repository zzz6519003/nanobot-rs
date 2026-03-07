//! ACP (Agent Client Protocol) integration module
//!
//! This module provides integration with ACP agents (codex, claude, pi, gemini, opencode)
//! as tools that can be called by the nanobot-rs agent.

pub mod client;
pub mod config;

pub use client::ACPClient;
pub use config::{ACPConfig, AgentConfig};
