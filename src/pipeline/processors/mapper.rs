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
                    
                    // Special case: @this returns the entire context
                    if inner == "@this" {
                        return Ok(context.clone());
                    }
                    
                    // Check if it's a pure variable reference (no filters, no string concatenation)
                    if !inner.contains('|') && !trimmed.contains("{{") || trimmed.matches("{{").count() == 1 {
                        if let Some(raw_value) = self.extract_value_from_path(inner, context) {
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
                // Check if this is a casting object with "value" and "castTo" fields
                if map.contains_key("value") && map.contains_key("castTo") {
                    let value_template = &map["value"];
                    let cast_to = map["castTo"].as_str()
                        .ok_or_else(|| AppError::Processing(
                            "castTo must be a string".to_string()
                        ))?;
                    
                    let rendered_value = self.render_value(value_template, context)?;
                    
                    return self.cast_value(&rendered_value, cast_to);
                }
                
                // Otherwise, recursively render all values in the object
                let mut result = serde_json::Map::new();
                for (key, val) in map {
                    result.insert(key.clone(), self.render_value(val, context)?);
                }
                Ok(Value::Object(result))
            }
            Value::Array(arr) => {
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
    
    fn cast_value(&self, value: &Value, cast_to: &str) -> Result<Value> {
        match cast_to.to_lowercase().as_str() {
            "string" => {
                match value {
                    Value::String(s) => Ok(Value::String(s.clone())),
                    Value::Number(n) => Ok(Value::String(n.to_string())),
                    Value::Bool(b) => Ok(Value::String(b.to_string())),
                    Value::Null => Ok(Value::String(String::new())),
                    _ => Err(AppError::Processing(
                        format!("Cannot cast complex type to string: {:?}", value)
                    )),
                }
            }
            "number" => {
                match value {
                    Value::Number(n) => Ok(Value::Number(n.clone())),
                    Value::String(s) => {
                        // Try to parse as integer first, then as float
                        if let Ok(i) = s.parse::<i64>() {
                            Ok(serde_json::json!(i))
                        } else if let Ok(f) = s.parse::<f64>() {
                            Ok(serde_json::json!(f))
                        } else {
                            Err(AppError::Processing(
                                format!("Cannot parse '{}' as number", s)
                            ))
                        }
                    }
                    Value::Bool(b) => Ok(serde_json::json!(if *b { 1 } else { 0 })),
                    _ => Err(AppError::Processing(
                        format!("Cannot cast type to number: {:?}", value)
                    )),
                }
            }
            _ => Err(AppError::Processing(
                format!("Unsupported cast type: '{}'. Supported types are: string, number", cast_to)
            )),
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
    
    #[tokio::test]
    async fn test_cast_string_to_number() {
        let template = json!({
            "issueId": {
                "value": "{{ issue.id }}",
                "castTo": "number"
            },
            "count": {
                "value": "{{ issue.count }}",
                "castTo": "number"
            }
        });
        
        let mapper = MapperProcessor::new(template).unwrap();
        
        let event = PipelineEvent::new(
            json!({
                "issue": {
                    "id": "12345",
                    "count": "42"
                }
            }),
            "test_event".to_string(),
            vec![],
            Operation::Write,
        );
        
        let result = mapper.process(event).await.unwrap();
        assert!(result.is_some());
        
        let result_event = result.unwrap();
        assert!(result_event.body["issueId"].is_number());
        assert_eq!(result_event.body["issueId"], 12345);
        assert!(result_event.body["count"].is_number());
        assert_eq!(result_event.body["count"], 42);
    }
    
    #[tokio::test]
    async fn test_cast_number_to_string() {
        let template = json!({
            "priorityLabel": {
                "value": "{{ issue.priority }}",
                "castTo": "string"
            },
            "statusCode": {
                "value": "{{ issue.status }}",
                "castTo": "string"
            }
        });
        
        let mapper = MapperProcessor::new(template).unwrap();
        
        let event = PipelineEvent::new(
            json!({
                "issue": {
                    "priority": 3,
                    "status": 200
                }
            }),
            "test_event".to_string(),
            vec![],
            Operation::Write,
        );
        
        let result = mapper.process(event).await.unwrap();
        assert!(result.is_some());
        
        let result_event = result.unwrap();
        assert!(result_event.body["priorityLabel"].is_string());
        assert_eq!(result_event.body["priorityLabel"], "3");
        assert!(result_event.body["statusCode"].is_string());
        assert_eq!(result_event.body["statusCode"], "200");
    }
    
    #[tokio::test]
    async fn test_mixed_casting_and_normal_fields() {
        let template = json!({
            "key": "{{ issue.key }}",
            "issueId": {
                "value": "{{ issue.id }}",
                "castTo": "number"
            },
            "summary": "{{ issue.summary }}"
        });
        
        let mapper = MapperProcessor::new(template).unwrap();
        
        let event = PipelineEvent::new(
            json!({
                "issue": {
                    "key": "TEST-123",
                    "id": "9999",
                    "summary": "Test issue"
                }
            }),
            "test_event".to_string(),
            vec![],
            Operation::Write,
        );
        
        let result = mapper.process(event).await.unwrap();
        assert!(result.is_some());
        
        let result_event = result.unwrap();
        assert_eq!(result_event.body["key"], "TEST-123");
        assert!(result_event.body["issueId"].is_number());
        assert_eq!(result_event.body["issueId"], 9999);
        assert_eq!(result_event.body["summary"], "Test issue");
    }
    
    #[tokio::test]
    async fn test_plain_value_mappings() {
        let template = json!({
            "staticText": "My var text",
            "staticNumber": 42,
            "staticBool": true,
            "staticNull": null,
            "staticArray": [1, 2, 3],
            "staticObject": {
                "nested": "value"
            },
            "dynamicValue": "{{ issue.id }}"
        });
        
        let mapper = MapperProcessor::new(template).unwrap();
        
        let event = PipelineEvent::new(
            json!({
                "issue": {
                    "id": "12345"
                }
            }),
            "test_event".to_string(),
            vec![],
            Operation::Write,
        );
        
        let result = mapper.process(event).await.unwrap();
        assert!(result.is_some());
        
        let result_event = result.unwrap();
        
        assert_eq!(result_event.body["staticText"], "My var text");
        assert_eq!(result_event.body["staticNumber"], 42);
        assert_eq!(result_event.body["staticBool"], true);
        assert!(result_event.body["staticNull"].is_null());
        assert!(result_event.body["staticArray"].is_array());
        assert_eq!(result_event.body["staticArray"], json!([1, 2, 3]));
        assert!(result_event.body["staticObject"].is_object());
        assert_eq!(result_event.body["staticObject"]["nested"], "value");
        assert_eq!(result_event.body["dynamicValue"], "12345");
    }
}
