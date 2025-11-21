use crate::error::{AppError, Result};
use crate::pipeline::event::PipelineEvent;
use super::Processor;
use handlebars::Handlebars;
use serde_json::Value;

/// Mapper processor that transforms events using Handlebars templates
pub struct MapperProcessor {
    handlebars: Handlebars<'static>,
    template: Value,
}

impl MapperProcessor {
    pub fn new(template: Value) -> Result<Self> {
        let handlebars = Handlebars::new();
        Ok(Self { handlebars, template })
    }
    
    /// Recursively render a template value
    fn render_value(&self, value: &Value, context: &Value) -> Result<Value> {
        match value {
            Value::String(s) => {
                // Render the template string
                let rendered = self.handlebars.render_template(s, context)
                    .map_err(|e| AppError::Processing(format!("Template rendering failed: {}", e)))?;
                Ok(Value::String(rendered))
            }
            Value::Object(map) => {
                // Recursively render all values in the object
                let mut result = serde_json::Map::new();
                for (key, val) in map {
                    result.insert(key.clone(), self.render_value(val, context)?);
                }
                Ok(Value::Object(result))
            }
            Value::Array(arr) => {
                // Recursively render all values in the array
                let mut result = Vec::new();
                for val in arr {
                    result.push(self.render_value(val, context)?);
                }
                Ok(Value::Array(result))
            }
            // For other types (null, bool, number), return as-is
            _ => Ok(value.clone()),
        }
    }
}

#[async_trait::async_trait]
impl Processor for MapperProcessor {
    async fn process(&self, mut event: PipelineEvent) -> Result<Option<PipelineEvent>> {
        // Render the template with the event body as context
        let new_body = self.render_value(&self.template, &event.body)?;
        
        // Update the event body
        event.body = new_body;
        
        Ok(Some(event))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::pipeline::event::{Operation, PkField};

    #[tokio::test]
    async fn test_simple_mapping() {
        let template = json!({
            "name": "{{ deployment.name }}",
            "status": "{{ deployment.status }}"
        });
        
        let mapper = MapperProcessor::new(template).unwrap();
        
        let event = PipelineEvent::new(
            json!({
                "deployment": {
                    "name": "app-v1",
                    "status": "success"
                }
            }),
            "test_event".to_string(),
            vec![],
            Operation::Write,
        );
        
        let result = mapper.process(event).await.unwrap();
        assert!(result.is_some());
        
        let result_event = result.unwrap();
        assert_eq!(result_event.body["name"], "app-v1");
        assert_eq!(result_event.body["status"], "success");
    }

    #[tokio::test]
    async fn test_mapping_with_top_level_field() {
        let template = json!({
            "timestamp": "{{ timestamp }}"
        });
        
        let mapper = MapperProcessor::new(template).unwrap();
        
        let event = PipelineEvent::new(
            json!({
                "timestamp": "2023-01-01T00:00:00Z"
            }),
            "test_event".to_string(),
            vec![],
            Operation::Write,
        );
        
        let result = mapper.process(event).await.unwrap();
        assert!(result.is_some());
        
        let result_event = result.unwrap();
        assert_eq!(result_event.body["timestamp"], "2023-01-01T00:00:00Z");
    }
}
