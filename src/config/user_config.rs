use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub provider: String,
    pub api_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub policy_profile: String,
    pub temperature: f64,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),
            api_url: "http://localhost:11434/v1".to_string(),
            api_key: None,
            model: "qwen2.5-coder".to_string(),
            policy_profile: "Standard".to_string(),
            temperature: 0.0,
        }
    }
}

impl UserConfig {
    pub fn config_path() -> PathBuf {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".taintbox").join("config.json")
    }

    pub fn load() -> Option<Self> {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<UserConfig>(&content) {
                    return Some(cfg);
                }
            }
        }

        // Fallback: check environment variables
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            return Some(Self {
                provider: "openai".to_string(),
                api_url: std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
                api_key: Some(key),
                model: std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string()),
                policy_profile: "Standard".to_string(),
                temperature: 0.0,
            });
        }
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            return Some(Self {
                provider: "anthropic".to_string(),
                api_url: "https://api.anthropic.com/v1".to_string(),
                api_key: Some(key),
                model: "claude-3-5-sonnet-20241022".to_string(),
                policy_profile: "Standard".to_string(),
                temperature: 0.0,
            });
        }

        None
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }
}

/// Interactive onboarding and configuration wizard for tbox
pub fn run_setup_wizard() -> anyhow::Result<UserConfig> {
    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                 TAINTBOX HARNESS: AGENT SETUP WIZARD                 ║");
    println!("║       Configure LLM provider, API credentials, and boundary policy   ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    println!("Select your LLM Inference Provider:");
    println!("  [1] Ollama / Local Bunker (default: http://localhost:11434/v1)");
    println!("  [2] OpenAI (https://api.openai.com/v1)");
    println!("  [3] OpenRouter (https://openrouter.ai/api/v1)");
    println!("  [4] DeepSeek (https://api.deepseek.com/v1)");
    println!("  [5] Anthropic Claude (https://api.anthropic.com/v1)");
    println!("  [6] Custom Self-Hosted / VPS Endpoint");
    print!("\nEnter choice [1-6] (default: 1): ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();

    let (provider, default_url, default_model, needs_key) = match choice {
        "2" => ("openai", "https://api.openai.com/v1", "gpt-4o", true),
        "3" => ("openrouter", "https://openrouter.ai/api/v1", "anthropic/claude-3.5-sonnet", true),
        "4" => ("deepseek", "https://api.deepseek.com/v1", "deepseek-chat", true),
        "5" => ("anthropic", "https://api.anthropic.com/v1", "claude-3-5-sonnet-20241022", true),
        "6" => ("custom", "http://localhost:8080/v1", "custom-model", false),
        _ => ("ollama", "http://localhost:11434/v1", "qwen2.5-coder", false),
    };

    // API URL
    print!("Endpoint URL [{}]: ", default_url);
    io::stdout().flush()?;
    let mut url_input = String::new();
    io::stdin().read_line(&mut url_input)?;
    let url_input = url_input.trim();
    let api_url = if url_input.is_empty() { default_url.to_string() } else { url_input.to_string() };

    // API Key
    let api_key = if needs_key {
        print!("API Key: ");
        io::stdout().flush()?;
        let mut key_input = String::new();
        io::stdin().read_line(&mut key_input)?;
        let key_input = key_input.trim();
        if key_input.is_empty() { None } else { Some(key_input.to_string()) }
    } else {
        print!("API Key (optional for Ollama/local, press Enter to skip): ");
        io::stdout().flush()?;
        let mut key_input = String::new();
        io::stdin().read_line(&mut key_input)?;
        let key_input = key_input.trim();
        if key_input.is_empty() { None } else { Some(key_input.to_string()) }
    };

    // Model name
    print!("Model Identifier [{}]: ", default_model);
    io::stdout().flush()?;
    let mut model_input = String::new();
    io::stdin().read_line(&mut model_input)?;
    let model_input = model_input.trim();
    let model = if model_input.is_empty() { default_model.to_string() } else { model_input.to_string() };

    // Policy profile
    println!("\nSelect Boundary Policy Enforcement Profile:");
    println!("  [1] Standard (Blocks unauthorized writes & unallowlisted egress on untrusted data)");
    println!("  [2] Strict (Zero unconfined execution; blocks all untrusted write/exec turns)");
    print!("Choice [1-2] (default: 1): ");
    io::stdout().flush()?;
    let mut pol_input = String::new();
    io::stdin().read_line(&mut pol_input)?;
    let policy_profile = if pol_input.trim() == "2" { "Strict".to_string() } else { "Standard".to_string() };

    let config = UserConfig {
        provider: provider.to_string(),
        api_url,
        api_key,
        model,
        policy_profile,
        temperature: 0.0,
    };

    config.save()?;
    println!("\n[+] Configuration saved to: {}", UserConfig::config_path().display());
    println!("[+] Provider: {} | Model: {}", config.provider, config.model);
    println!("[+] Policy Profile: {}\n", config.policy_profile);

    Ok(config)
}
