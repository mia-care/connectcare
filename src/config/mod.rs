pub mod secret;

use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::sources::jira::JiraSourceConfig;
use crate::pipeline::processors::ProcessorConfig;
use crate::pipeline::sinks::SinkConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub integrations: Vec<Integration>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Integration {
    pub source: SourceConfig,
    #[serde(default)]
    pub pipelines: Vec<Pipeline>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pipeline {
    #[serde(default)]
    pub processors: Vec<ProcessorConfig>,
    pub sinks: Vec<SinkConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SourceConfig {
    #[serde(rename = "jira")]
    Jira(JiraSourceConfig),
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    pub fn from_env() -> Result<Self> {
        let config_path = std::env::var("CONFIGURATION_PATH")
            .unwrap_or_else(|_| "config/config.json".to_string());
        Self::from_file(&config_path)
    }
    
    pub fn get_port() -> u16 {
        std::env::var("HTTP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3000)
    }
    
    pub fn mongodb_url() -> Result<String> {
        std::env::var("MONGO_URL")
            .map_err(|_| crate::error::AppError::Config("MONGO_URL environment variable is required".to_string()))
    }
}
