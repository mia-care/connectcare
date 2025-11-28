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
                // Check if this is a simple variable reference (e.g., "{{ foo }}" or "{{ foo.bar }}")
                let trimmed = s.trim();
                if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
                    let inner = trimmed[2..trimmed.len()-2].trim();
                    
                    // Check if it's a pure variable reference (no filters, no string concatenation)
                    if !inner.contains('|') && !trimmed.contains("{{") || trimmed.matches("{{").count() == 1 {
                        // Try to extract the raw value from context
                        if let Some(raw_value) = self.extract_value_from_path(inner, context) {
                            // Return the raw value preserving its type
                            return Ok(raw_value.clone());
                        }
                    }
                }
                
                // Otherwise, render as string template
                let rendered = self.handlebars.render_template(s, context)
                    .map_err(|e| AppError::Processing(format!("Template rendering failed: {}", e)))?;
                
                // Try to parse as JSON if the result looks like JSON
                if (rendered.starts_with('{') && rendered.ends_with('}')) 
                    || (rendered.starts_with('[') && rendered.ends_with(']')) {
                    if let Ok(parsed) = serde_json::from_str::<Value>(&rendered) {
                        return Ok(parsed);
                    }
                }
                
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
    
    /// Extract a value from a JSON path like "foo.bar.0.baz"
    fn extract_value_from_path<'a>(&self, path: &str, context: &'a Value) -> Option<&'a Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = context;
        
        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part)?;
                }
                Value::Array(arr) => {
                    let index: usize = part.parse().ok()?;
                    current = arr.get(index)?;
                }
                _ => return None,
            }
        }
        
        Some(current)
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
    use crate::pipeline::event::{Operation};

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
    
    #[tokio::test]
    async fn test_type_preservation_for_objects() {
        let template = json!({
            "status": "{{ issue.fields.status }}",
            "labels": "{{ issue.fields.labels }}",
            "priority": "{{ issue.fields.priority }}"
        });
        
        let mapper = MapperProcessor::new(template).unwrap();
        
        let event = PipelineEvent::new(
            json!({
                "issue": {
                    "fields": {
                        "status": {
                            "id": "1",
                            "name": "Open"
                        },
                        "labels": ["bug", "urgent"],
                        "priority": 4
                    }
                }
            }),
            "test_event".to_string(),
            vec![],
            Operation::Write,
        );
        
        let result = mapper.process(event).await.unwrap();
        assert!(result.is_some());
        
        let result_event = result.unwrap();
        
        // Check that object is preserved
        assert!(result_event.body["status"].is_object());
        assert_eq!(result_event.body["status"]["id"], "1");
        assert_eq!(result_event.body["status"]["name"], "Open");
        
        // Check that array is preserved
        assert!(result_event.body["labels"].is_array());
        assert_eq!(result_event.body["labels"][0], "bug");
        assert_eq!(result_event.body["labels"][1], "urgent");
        
        // Check that number is preserved
        assert!(result_event.body["priority"].is_number());
        assert_eq!(result_event.body["priority"], 4);
    }
    
    #[tokio::test]
    async fn test_nested_path_extraction() {
        let template = json!({
            "versionId": "{{ issue.fields.fixVersions.0.id }}",
            "versionName": "{{ issue.fields.fixVersions.0.name }}"
        });
        
        let mapper = MapperProcessor::new(template).unwrap();
        
        let event = PipelineEvent::new(
            json!({
                "issue": {
                    "fields": {
                        "fixVersions": [
                            {
                                "id": "12345",
                                "name": "v1.0.0"
                            }
                        ]
                    }
                }
            }),
            "test_event".to_string(),
            vec![],
            Operation::Write,
        );
        
        let result = mapper.process(event).await.unwrap();
        assert!(result.is_some());
        
        let result_event = result.unwrap();
        assert_eq!(result_event.body["versionId"], "12345");
        assert_eq!(result_event.body["versionName"], "v1.0.0");
    }
}
