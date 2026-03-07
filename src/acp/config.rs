//! ACP configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACPConfig {
    pub enabled: bool,
    pub default_agent: String,
    pub allowed_agents: Vec<String>,
    pub agents: HashMap<String, AgentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub command: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl Default for ACPConfig {
    fn default() -> Self {
        let mut agents = HashMap::new();
        
        agents.insert("codex".to_string(), AgentConfig {
            command: "codex".to_string(),
            env: HashMap::new(),
        });
        
        Self {
            enabled: true,
            default_agent: "codex".to_string(),
            allowed_agents: vec!["codex".to_string()],
            agents,
        }
    }
}
