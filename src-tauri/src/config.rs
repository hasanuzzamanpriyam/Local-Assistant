use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use dirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub model: ModelConfig,
    pub sidecar: SidecarConfig,
    pub overlay: OverlayConfig,
    pub esc_guard: EscGuardConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub default: String,
    pub context_size: u32,
    pub threads: u32,
    pub temperature: f32,
    pub top_p: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarConfig {
    pub python: String,
    pub auto_start: bool,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub enabled: bool,
    pub aura_color: String,
    pub comet_color: String,
    pub hud_position: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscGuardConfig {
    pub enabled: bool,
    pub fallback_shortcut: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let default = Config::default();
            default.save()?;
            Ok(default)
        }
    }

    pub fn config_path() -> anyhow::Result<PathBuf> {
        let base = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("No data dir"))?
            .join("assistant");
        std::fs::create_dir_all(&base)?;
        Ok(base.join("config.toml"))
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(Self::config_path()?, content)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: ModelConfig { 
                default: "Qwen2.5-3B-Instruct-Q4_K_M.gguf".into(), 
                context_size: 4096, 
                threads: 8, 
                temperature: 0.3, 
                top_p: 0.9 
            },
            sidecar: SidecarConfig { 
                python: "python".into(), 
                auto_start: true, 
                log_level: "INFO".into() 
            },
            overlay: OverlayConfig { 
                enabled: true, 
                aura_color: "#007AFF".into(), 
                comet_color: "#00D4FF".into(), 
                hud_position: "top-center".into() 
            },
            esc_guard: EscGuardConfig { 
                enabled: true, 
                fallback_shortcut: "Ctrl+Alt+Esc".into() 
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.model.default, "Qwen2.5-3B-Instruct-Q4_K_M.gguf");
        assert_eq!(config.model.threads, 8);
    }

    #[test]
    fn test_toml_roundtrip() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();
        assert_eq!(config.model.default, parsed.model.default);
    }
}