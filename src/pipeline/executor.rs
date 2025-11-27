use crate::config::{AppConfig, Pipeline};
use crate::pipeline::processors::ProcessorConfig;
use crate::error::Result;
use crate::pipeline::event::PipelineEvent;
use crate::pipeline::processors::{Processor, filter::FilterProcessor, mapper::MapperProcessor};
use crate::pipeline::sinks::{Sink, database::DatabaseSink, DatabaseProvider};
use crate::pipeline::PipelineReceiver;
use std::sync::Arc;
use tracing::{info, error, debug};

pub struct PipelineExecutor {
    pipelines: Vec<PipelineInstance>,
}

struct PipelineInstance {
    processors: Vec<Box<dyn Processor>>,
    sinks: Vec<Arc<dyn Sink>>,
}

impl PipelineExecutor {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        let mut pipelines = Vec::new();
        
        for integration in &config.integrations {
            for pipeline_config in &integration.pipelines {
                let pipeline = Self::create_pipeline(config, pipeline_config).await?;
                pipelines.push(pipeline);
            }
        }
        
        Ok(Self { pipelines })
    }
    
    async fn create_pipeline(_config: &AppConfig, pipeline_config: &Pipeline) -> Result<PipelineInstance> {
        // Build processors
        let mut processors: Vec<Box<dyn Processor>> = Vec::new();
        
        for processor_config in &pipeline_config.processors {
            match processor_config {
                ProcessorConfig::Filter { cel_expression } => {
                    let filter = FilterProcessor::new(cel_expression)?;
                    processors.push(Box::new(filter));
                }
                ProcessorConfig::Mapper { output_event } => {
                    let mapper = MapperProcessor::new(output_event.clone())?;
                    processors.push(Box::new(mapper));
                }
            }
        }
        
        let mut sinks: Vec<Arc<dyn Sink>> = Vec::new();
        
        for sink_config in &pipeline_config.sinks {
            match sink_config {
                crate::pipeline::sinks::SinkConfig::Mongo { url, collection, insert_only: _ } => {
                    let mongo_url = url.resolve()?;
                    
                    let (base_url, database) = Self::parse_mongo_url_for_sink(&mongo_url)?;
                    let sink = DatabaseSink::with_collection(&base_url, &database, collection).await?;
                    
                    sinks.push(Arc::new(sink));
                }
                crate::pipeline::sinks::SinkConfig::Database { provider } => {
                    match provider {
                        DatabaseProvider::Mongo => {
                            let mongo_url = crate::config::AppConfig::mongodb_url()?;
                            
                            let sink = DatabaseSink::new(&mongo_url).await?;
                            
                            sinks.push(Arc::new(sink));
                        }
                    }
                }
            }
        }
        
        Ok(PipelineInstance { processors, sinks })
    }
    
    fn parse_mongo_url_for_sink(url: &str) -> Result<(String, String)> {
        let url_without_protocol = url.strip_prefix("mongodb://")
            .or_else(|| url.strip_prefix("mongodb+srv://"))
            .ok_or_else(|| crate::error::AppError::Config(
                "Invalid MongoDB URL: must start with mongodb:// or mongodb+srv://".to_string()
            ))?;
        
        if let Some(slash_pos) = url_without_protocol.find('/') {
            let base = &url[..url.len() - url_without_protocol.len() + slash_pos];
            let path = &url_without_protocol[slash_pos + 1..];
            
            let database = path.split('?').next().unwrap_or(path).split('/').next().unwrap_or("");
            
            if database.is_empty() {
                return Err(crate::error::AppError::Config(
                    "MongoDB URL must include database name (format: mongodb://host:port/database)".to_string()
                ));
            }
            
            Ok((base.to_string(), database.to_string()))
        } else {
            Err(crate::error::AppError::Config(
                "MongoDB URL must include database name (format: mongodb://host:port/database)".to_string()
            ))
        }
    }
    
    pub async fn run(self, mut receiver: PipelineReceiver) {
        info!("Pipeline executor started with {} pipelines", self.pipelines.len());
        
        while let Some(event) = receiver.recv().await {
            debug!("Received event: id={}, type={}", event.id, event.event_type);
            
            // Process the event through all pipelines
            for (idx, pipeline) in self.pipelines.iter().enumerate() {
                if let Err(e) = self.process_event(&event, pipeline, idx).await {
                    error!("Error processing event in pipeline {}: {}", idx, e);
                }
            }
        }
        
        info!("Pipeline executor stopped");
    }
    
    async fn process_event(&self, event: &PipelineEvent, pipeline: &PipelineInstance, pipeline_idx: usize) -> Result<()> {
        let mut current_event = event.clone();
        
        // Process through all processors
        for (idx, processor) in pipeline.processors.iter().enumerate() {
            match processor.process(current_event).await? {
                Some(processed_event) => {
                    current_event = processed_event;
                    debug!("Event passed through processor {} in pipeline {}", idx, pipeline_idx);
                }
                None => {
                    debug!("Event filtered out by processor {} in pipeline {}", idx, pipeline_idx);
                    return Ok(()); // Event was filtered out
                }
            }
        }
        
        // Write to all sinks
        for (idx, sink) in pipeline.sinks.iter().enumerate() {
            match sink.write(&current_event).await {
                Ok(_) => {
                    debug!("Event written to sink {} in pipeline {}", idx, pipeline_idx);
                }
                Err(e) => {
                    error!("Failed to write event to sink {} in pipeline {}: {}", idx, pipeline_idx, e);
                    // Continue to other sinks even if one fails
                }
            }
        }
        
        Ok(())
    }
}
