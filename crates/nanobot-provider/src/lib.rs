pub mod anthropic;
mod anthropic_types;
pub mod base;
pub mod error;
pub mod fallback;
pub mod openai_compat;
mod openai_types;
pub mod proxy;
pub mod registry;
pub mod streaming;
pub mod tool_name;
pub mod traits;

use std::sync::Arc;

use nanobot_config::Config;
use nanobot_config::schema::ProviderType;

pub use crate::base::*;
pub use error::{ProviderError, ProviderResult};
pub use nanobot_types::provider::*;

use anthropic::AnthropicProvider;
use fallback::FallbackProvider;
pub use nanobot_types::tool_name::ToolName;
use openai_compat::OpenAICompatProvider;

/// Constructs an `LLMProvider` from the given configuration.
///
/// Selects the appropriate provider backend (Anthropic, OpenAI-compatible, etc.)
/// based on `config.agents.defaults.model` and `config.agents.defaults.provider`.
/// If `fallback_providers` are configured, wraps providers in a `FallbackProvider`.
pub fn make_provider(config: &Config) -> ProviderResult<Arc<dyn LLMProvider>> {
    let provider_name = config.get_provider_name(None).ok_or_else(|| {
        ProviderError::InvalidConfig("no provider matched for current configuration".to_string())
    })?;
    let model = config.model_for_provider(&provider_name);

    if config.provider_type(&provider_name) == ProviderType::OAuth {
        return Err(ProviderError::InvalidConfig(format!(
            "OAuth provider '{}' is not supported as LLM provider. Use ACP as a tool instead.",
            provider_name
        )));
    }

    // Check if fallback providers are configured
    if let Some(fallback_names) = &config.agents.defaults.fallback_providers
        && !fallback_names.is_empty()
    {
        let mut providers = Vec::new();

        // Add primary provider
        providers.push(create_single_provider(config, &provider_name)?);

        // Add fallback providers
        for fallback_name in fallback_names {
            let fallback_provider = create_single_provider(config, fallback_name)?;
            providers.push(fallback_provider);
        }

        return Ok(Arc::new(FallbackProvider::new(providers, model)));
    }

    // No fallback configured, return single provider
    create_single_provider(config, &provider_name)
}

fn create_single_provider(
    config: &Config,
    provider_name: &str,
) -> ProviderResult<Arc<dyn LLMProvider>> {
    let provider_cfg = config
        .provider_config(provider_name)
        .cloned()
        .unwrap_or_default();
    let provider_type = config.provider_type(provider_name);
    let wire_api = config.wire_api(provider_name);
    let model = config.model_for_provider(provider_name);

    tracing::debug!(
        "Creating provider '{}' for model '{}', api_key set: {}, api_base: {:?}",
        provider_name,
        model,
        !provider_cfg.api_key.trim().is_empty(),
        provider_cfg.api_base
    );

    if provider_type != ProviderType::OAuth
        && provider_name != "custom"
        && provider_cfg.api_key.trim().is_empty()
        && !model.starts_with("bedrock/")
    {
        return Err(ProviderError::InvalidConfig(format!(
            "no API key configured for provider '{}' (model: {})",
            provider_name, model
        )));
    }

    let api_base = if provider_name == "custom" {
        Some(
            provider_cfg
                .api_base
                .clone()
                .unwrap_or_else(|| "http://localhost:8000/v1".to_string()),
        )
    } else {
        provider_cfg
            .api_base
            .clone()
            .filter(|base| !base.trim().is_empty())
    };

    let extra_headers = provider_cfg.extra_headers.unwrap_or_default();

    // TODO: this is a bit hacky,
    // we should have a more robust way to determine provider type from model name or config
    match provider_type {
        ProviderType::Anthropic => Ok(Arc::new(AnthropicProvider::new(
            provider_cfg.api_key,
            api_base,
            model,
            extra_headers,
        ))),
        ProviderType::OpenAiCompatible => Ok(Arc::new(OpenAICompatProvider::new(
            provider_cfg.api_key,
            api_base,
            model,
            provider_name.to_string(),
            wire_api,
            extra_headers,
        ))),
        ProviderType::OAuth => Err(ProviderError::InvalidConfig(format!(
            "OAuth provider '{}' is not supported as LLM provider. Use ACP as a tool instead.",
            provider_name
        ))),
    }
}
