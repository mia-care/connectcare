pub mod database;

use crate::error::Result;
use crate::pipeline::event::PipelineEvent;
use serde::{Deserialize, Serialize};

/// Sink configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SinkConfig {
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

/// Trait for event sinks
#[async_trait::async_trait]
pub trait Sink: Send + Sync {
    /// Write an event to the sink
    async fn write(&self, event: &PipelineEvent) -> Result<()>;
}
