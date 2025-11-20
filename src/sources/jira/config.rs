use serde::{Deserialize, Serialize};
use crate::config::secret::SecretSource;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JiraSourceConfig {
    #[serde(default = "default_webhook_path")]
    pub webhook_path: String,
    
    pub authentication: JiraAuthentication,
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
