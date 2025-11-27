use serde::{Deserialize, Serialize};
use crate::config::secret::SecretSource;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraSourceConfig {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_path: Option<String>,
    
    pub authentication: JiraAuthentication,
}

impl JiraSourceConfig {
    pub fn get_webhook_path(&self) -> String {
        self.webhook_path.clone().unwrap_or_else(default_webhook_path)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraAuthentication {
    pub secret: SecretSource,
    
    #[serde(default = "default_header_name")]
    pub header_name: String,
}

fn default_webhook_path() -> String {
    "/jira/webhook".to_string()
}

fn default_header_name() -> String {
    "X-Hub-Signature".to_string()
}
