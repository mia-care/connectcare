pub mod database;

use crate::error::Result;
use crate::pipeline::event::PipelineEvent;
use crate::config::secret::SecretSource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SinkConfig {
    Mongo {
        url: SecretSource,
        collection: String,
        #[serde(default)]
        insert_only: bool,
    },
    #[serde(rename = "database")]
    Database { 
        provider: DatabaseProvider 
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DatabaseProvider {
    Mongo,
}

#[async_trait::async_trait]
pub trait Sink: Send + Sync {
    async fn write(&self, event: &PipelineEvent) -> Result<()>;
}
