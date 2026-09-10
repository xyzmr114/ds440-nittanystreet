use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_window: usize,
    pub max_output_tokens: usize,
    pub tool_format: String, // "xml", "native_json"
    pub cost_input_per_million: f64,
    pub cost_output_per_million: f64,
}

impl ModelSpec {
    pub fn new(
        id: &str,
        name: &str,
        provider: &str,
        context_window: usize,
        max_output_tokens: usize,
        tool_format: &str,
        cost_in: f64,
        cost_out: f64,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            provider: provider.to_string(),
            context_window,
            max_output_tokens,
            tool_format: tool_format.to_string(),
            cost_input_per_million: cost_in,
            cost_output_per_million: cost_out,
        }
    }
}

/// Dynamic Model Catalog powered by models.dev standards
pub struct ModelCatalog;

impl ModelCatalog {
    /// Returns default models for a given provider
    pub fn get_models_for_provider(provider: &str) -> Vec<ModelSpec> {
        let catalog = Self::get_full_catalog();
        let prov_lower = provider.to_ascii_lowercase();
        let matches: Vec<ModelSpec> = catalog
            .into_iter()
            .filter(|m| m.provider.to_ascii_lowercase() == prov_lower)
            .collect();

        if matches.is_empty() {
            vec![ModelSpec::new(
                "default",
                "Default Model",
                provider,
                32_768,
                4_096,
                "xml",
                0.0,
                0.0,
            )]
        } else {
            matches
        }
    }

    /// Lookup a specific model specification by ID
    pub fn find_model(model_id: &str) -> Option<ModelSpec> {
        let id_lower = model_id.to_ascii_lowercase();
        Self::get_full_catalog()
            .into_iter()
            .find(|m| m.id.to_ascii_lowercase() == id_lower || id_lower.contains(&m.id.to_ascii_lowercase()))
    }

    /// Complete models.dev catalog covering local, open-weights, and frontier providers
    pub fn get_full_catalog() -> Vec<ModelSpec> {
        vec![
            // Ollama / Local Bunker
            ModelSpec::new("qwen2.5-coder:7b", "Qwen 2.5 Coder 7B", "ollama", 32_768, 8_192, "xml", 0.0, 0.0),
            ModelSpec::new("qwen2.5-coder:14b", "Qwen 2.5 Coder 14B", "ollama", 32_768, 8_192, "xml", 0.0, 0.0),
            ModelSpec::new("qwen2.5-coder:32b", "Qwen 2.5 Coder 32B", "ollama", 32_768, 8_192, "xml", 0.0, 0.0),
            ModelSpec::new("deepseek-r1:8b", "DeepSeek R1 Distill 8B", "ollama", 65_536, 8_192, "xml", 0.0, 0.0),
            ModelSpec::new("deepseek-r1:14b", "DeepSeek R1 Distill 14B", "ollama", 65_536, 8_192, "xml", 0.0, 0.0),
            ModelSpec::new("llama3.3:70b", "Llama 3.3 70B Instruct", "ollama", 131_072, 8_192, "xml", 0.0, 0.0),
            ModelSpec::new("codellama:13b", "CodeLlama 13B", "ollama", 16_384, 4_096, "xml", 0.0, 0.0),

            // OpenAI
            ModelSpec::new("gpt-4o", "GPT-4o (Omni Frontier)", "openai", 128_000, 16_384, "native_json", 2.50, 10.00),
            ModelSpec::new("gpt-4o-mini", "GPT-4o Mini", "openai", 128_000, 16_384, "native_json", 0.15, 0.60),
            ModelSpec::new("o1", "OpenAI o1 Reasoning", "openai", 200_000, 100_000, "native_json", 15.00, 60.00),
            ModelSpec::new("o1-mini", "OpenAI o1 Mini", "openai", 128_000, 65_536, "native_json", 3.00, 12.00),
            ModelSpec::new("o3-mini", "OpenAI o3 Mini", "openai", 200_000, 100_000, "native_json", 1.10, 4.40),

            // Anthropic
            ModelSpec::new("claude-3-7-sonnet-20250219", "Claude 3.7 Sonnet (Hybrid Thinking)", "anthropic", 200_000, 64_000, "native_json", 3.00, 15.00),
            ModelSpec::new("claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet v2", "anthropic", 200_000, 8_192, "native_json", 3.00, 15.00),
            ModelSpec::new("claude-3-5-haiku-20241022", "Claude 3.5 Haiku", "anthropic", 200_000, 8_192, "native_json", 0.80, 4.00),
            ModelSpec::new("claude-3-opus-20240229", "Claude 3 Opus", "anthropic", 200_000, 4_096, "native_json", 15.00, 75.00),

            // DeepSeek
            ModelSpec::new("deepseek-chat", "DeepSeek V3", "deepseek", 64_000, 8_192, "xml", 0.14, 0.28),
            ModelSpec::new("deepseek-reasoner", "DeepSeek R1", "deepseek", 64_000, 8_192, "xml", 0.55, 2.19),

            // OpenRouter Aggregator
            ModelSpec::new("anthropic/claude-3.7-sonnet", "OpenRouter: Claude 3.7 Sonnet", "openrouter", 200_000, 64_000, "native_json", 3.00, 15.00),
            ModelSpec::new("anthropic/claude-3.5-sonnet", "OpenRouter: Claude 3.5 Sonnet", "openrouter", 200_000, 8_192, "native_json", 3.00, 15.00),
            ModelSpec::new("deepseek/deepseek-r1", "OpenRouter: DeepSeek R1", "openrouter", 128_000, 8_192, "xml", 0.55, 2.19),
            ModelSpec::new("meta-llama/llama-3.3-70b-instruct", "OpenRouter: Llama 3.3 70B", "openrouter", 131_072, 8_192, "native_json", 0.40, 0.40),
            ModelSpec::new("google/gemini-2.0-flash-001", "OpenRouter: Gemini 2.0 Flash", "openrouter", 1_048_576, 8_192, "native_json", 0.10, 0.40),

            // Custom / Self-Hosted VPS
            ModelSpec::new("custom-vps", "Custom Self-Hosted VLLM / SGLang", "custom", 65_536, 8_192, "xml", 0.0, 0.0),
        ]
    }
}
