pub mod filter;
pub mod mapper;

use crate::error::Result;
use crate::pipeline::event::PipelineEvent;
use serde::{Deserialize, Serialize};

/// Processor configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ProcessorConfig {
    #[serde(rename = "filter")]
    Filter { 
        #[serde(rename = "celExpression")]
        cel_expression: String 
    },
    #[serde(rename = "mapper")]
    Mapper { 
        #[serde(rename = "outputEvent")]
        output_event: serde_json::Value 
    },
}

/// Trait for event processors
#[async_trait::async_trait]
pub trait Processor: Send + Sync {
    /// Process an event, returning Some(event) if it should continue, None if filtered out
    async fn process(&self, event: PipelineEvent) -> Result<Option<PipelineEvent>>;
}
