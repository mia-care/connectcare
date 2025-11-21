pub mod secret;

use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::sources::jira::JiraSourceConfig;
use crate::pipeline::processors::ProcessorConfig;
use crate::pipeline::sinks::SinkConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub integrations: Vec<Integration>,
    #[serde(default)]
    pub mongodb: Option<MongoConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    8080
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MongoConfig {
    pub connection_string: String,
    pub database: String,
    pub collection: String,
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
}
