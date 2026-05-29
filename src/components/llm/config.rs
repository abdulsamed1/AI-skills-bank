use crate::components::llm::error::LlmError;
use std::env;

#[derive(Debug, Clone)]
pub struct LlmClientConfig {
    pub provider: String,
    pub api_key: String,
    pub api_url: Option<String>,
    pub model: Option<String>,
    pub ca_cert_path: Option<String>,
}

impl LlmClientConfig {
    pub fn from_env() -> Result<Self, LlmError> {
        // Prefer explicit LLM_PROVIDER, but if FreeLLMAPI env vars are present
        // force the provider to `freellmapi` so the tool only connects via the proxy.
        let mut provider = env::var("LLM_PROVIDER").or_else(|_| env::var("GV_PROVIDER_PRIMARY")).unwrap_or_default();

        let freellm_key = env::var("FREELLMAPI_API_KEY").ok();
        let freellm_url = env::var("FREELLMAPI_URL").ok();
        if freellm_key.is_some() || freellm_url.is_some() {
            provider = "freellmapi".to_string();
        }

        // If no provider was explicitly configured, default to FreeLLMAPI
        if provider.is_empty() {
            provider = "freellmapi".to_string();
        }

        let model = env::var("LLM_MODEL").ok();

        // If LLM_API_KEY is set explicitly, it overrides the provider-specific credentials
        if let Ok(key) = env::var("LLM_API_KEY") {
            let api_url = env::var("LLM_API_URL").ok();
            let ca_cert_path = env::var("LLM_CA_CERT_PATH").ok();
            return Ok(Self {
                provider,
                api_key: key,
                api_url,
                model,
                ca_cert_path,
            });
        }

        Self::for_provider(&provider)
            .map(|mut cfg| {
                if cfg.model.is_none() {
                    cfg.model = model;
                }
                cfg
            })
            .ok_or_else(|| LlmError::ConfigError(format!("API key not set for provider: {}", provider)))
    }

    pub fn for_provider(provider_name: &str) -> Option<Self> {
        let provider = provider_name.to_string();
        let provider_lower = provider_name.to_ascii_lowercase();
        let provider_upper = provider_name.to_ascii_uppercase().replace('-', "_");

        // 1. Resolve API key
        let api_key = match provider_lower.as_str() {
            "groq" => env::var("GROQ_API_KEY").ok(),
            "sambanova" => env::var("SAMBANOVA_API_KEY").ok(),
            "cerebras" => env::var("CEREBRAS_API_KEY").ok(),
            "hyperbolic" => env::var("HYPERBOLIC_API_TOKEN").ok(),
            "cometapi" => env::var("COMETAPI_KEY").ok(),
            "mistral" => env::var("MISTRAL_API_KEY").ok(),
            "github" | "github_models" => env::var("GITHUB_TOKEN").ok(),
            "openrouter" => env::var("OPENROUTER_API_KEY").ok(),
            "vercel" => env::var("AI_GATEWAY_API_KEY").or_else(|_| env::var("VERCEL_API_KEY")).ok(),
            "cloudflare" => env::var("CLOUDFLARE_API_KEY").ok(),
            "bedrock" | "aws" => env::var("AWS_BEARER_TOKEN_BEDROCK").or_else(|_| env::var("OPENAI_API_KEY")).ok(),
            "openai" => env::var("OPENAI_API_KEY").ok(),
            "claude" => env::var("ANTHROPIC_API_KEY").ok(),
            "gemini" => env::var("GEMINI_API_KEY").ok(),
            "freellmapi" | "custom" => env::var("LLM_API_KEY").ok().or_else(|| env::var("FREELLMAPI_API_KEY").ok()),
            _ => None,
        };

        let api_key = match api_key {
            Some(k) if !k.is_empty() => k,
            _ => return None,
        };

        // 2. Resolve API URL
        let api_url = match provider_lower.as_str() {
            "groq" => Some("https://api.groq.com/openai/v1/chat/completions".to_string()),
            "sambanova" => Some("https://api.sambanova.ai/v1/chat/completions".to_string()),
            "cerebras" => Some("https://api.cerebras.ai/v1/chat/completions".to_string()),
            "hyperbolic" => Some("https://api.hyperbolic.xyz/v1/chat/completions".to_string()),
            "cometapi" => Some("https://api.cometapi.com/v1/chat/completions".to_string()),
            "mistral" => Some("https://api.mistral.ai/v1/chat/completions".to_string()),
            "github" | "github_models" => Some("https://models.github.ai/inference/chat/completions".to_string()),
            "openrouter" => Some("https://openrouter.ai/api/v1/chat/completions".to_string()),
            "vercel" => Some("https://ai-gateway.vercel.sh/v1/chat/completions".to_string()),
            "cloudflare" => {
                let account_id = env::var("CLOUDFLARE_ACCOUNT_ID").unwrap_or_default();
                if !account_id.is_empty() {
                    Some(format!("https://api.cloudflare.com/client/v4/accounts/{}/ai/v1/chat/completions", account_id))
                } else {
                    None
                }
            }
            "bedrock" | "aws" => {
                let region = env::var("AWS_REGION").or_else(|_| env::var("AWS_DEFAULT_REGION")).unwrap_or_else(|_| "us-east-1".to_string());
                Some(format!("https://bedrock-mantle.{}.api.aws/v1/chat/completions", region))
            }
            "openai" => Some("https://api.openai.com/v1/chat/completions".to_string()),
            "claude" => Some("https://api.anthropic.com/v1/messages".to_string()),
            "gemini" => Some("https://generativelanguage.googleapis.com/v1beta/models".to_string()),
            "freellmapi" | "custom" => {
                // Support both LLM_API_URL and FREELLMAPI_URL
                env::var("LLM_API_URL")
                    .or_else(|_| env::var("FREELLMAPI_URL"))
                    .ok()
                    .or_else(|| Some("http://localhost:3001/v1/chat/completions".to_string()))
            }
            _ => None,
        };

        let ca_cert_path = env::var("LLM_CA_CERT_PATH").ok();
        
        // 3. Resolve Model (support {PROVIDER}_MODEL, e.g. GROQ_MODEL)
        let model = match provider_lower.as_str() {
            "groq" => env::var("GROQ_MODEL").ok().or_else(|| Some("meta-llama/llama-4-scout-17b-16e-instruct".to_string())),
            "sambanova" => env::var("SAMBANOVA_MODEL").ok().or_else(|| Some("Meta-Llama-3.3-70B-Instruct".to_string())),
            "cerebras" => env::var("CEREBRAS_MODEL").ok().or_else(|| Some("llama3.1-8b".to_string())),
            "hyperbolic" => env::var("HYPERBOLIC_MODEL").ok().or_else(|| Some("meta-llama/Llama-3.2-3B-Instruct".to_string())),
            "mistral" => env::var("MISTRAL_MODEL").ok().or_else(|| Some("mistral-small-latest".to_string())),
            "cloudflare" => env::var("CLOUDFLARE_MODEL").ok().or_else(|| Some("@cf/meta/llama-3.3-70b-instruct-awq".to_string())),
            "github" | "github_models" => env::var("GITHUB_MODEL").ok().or_else(|| Some("gpt-4o-mini".to_string())),
            _ => env::var(format!("{}_MODEL", provider_upper)).ok(),
        };

        Some(Self {
            provider,
            api_key,
            api_url,
            model,
            ca_cert_path,
        })
    }
}
