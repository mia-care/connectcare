use crate::error::{AppError, Result};
use crate::pipeline::event::PipelineEvent;
use super::Processor;
use cel_interpreter::{Context, Program};

/// Filter processor that evaluates CEL expressions
pub struct FilterProcessor {
    program: Program,
}

impl FilterProcessor {
    pub fn new(cel_expression: &str) -> Result<Self> {
        let program = Program::compile(cel_expression)
            .map_err(|e| AppError::Config(format!("Failed to compile CEL expression: {}", e)))?;
        
        Ok(Self { program })
    }
}

#[async_trait::async_trait]
impl Processor for FilterProcessor {
    async fn process(&self, event: PipelineEvent) -> Result<Option<PipelineEvent>> {
        // Create CEL context with event data
        let mut context = Context::default();
        
        // Add event fields to context
        context.add_variable("eventType", event.event_type.clone())
            .map_err(|e| AppError::Processing(format!("Failed to add eventType to context: {}", e)))?;
        
        // Add the entire body as a variable
        context.add_variable("body", event.body.clone())
            .map_err(|e| AppError::Processing(format!("Failed to add body to context: {}", e)))?;
        
        // If body is an object, add its top-level fields directly
        if let Some(obj) = event.body.as_object() {
            for (key, value) in obj {
                context.add_variable(key, value.clone())
                    .map_err(|e| AppError::Processing(format!("Failed to add field {} to context: {}", key, e)))?;
            }
        }
        
        // Evaluate the expression
        let result = self.program.execute(&context)
            .map_err(|e| AppError::Processing(format!("Failed to evaluate CEL expression: {}", e)))?;
        
        // Check if result is a boolean true
        // CEL interpreter returns a cel_interpreter::Value, check if it's a boolean
        match &result {
            cel_interpreter::Value::Bool(true) => Ok(Some(event)),
            cel_interpreter::Value::Bool(false) => Ok(None),
            _ => Err(AppError::Processing(
                format!("CEL expression did not evaluate to boolean, got: {:?}", result)
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::pipeline::event::{Operation};

    #[tokio::test]
    async fn test_filter_passes() {
        let filter = FilterProcessor::new("eventType == 'test_event'").unwrap();
        
        let event = PipelineEvent::new(
            json!({"data": "test"}),
            "test_event".to_string(),
            vec![],
            Operation::Write,
        );
        
        let result = filter.process(event).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_filter_blocks() {
        let filter = FilterProcessor::new("eventType == 'other_event'").unwrap();
        
        let event = PipelineEvent::new(
            json!({"data": "test"}),
            "test_event".to_string(),
            vec![],
            Operation::Write,
        );
        
        let result = filter.process(event).await.unwrap();
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_filter_with_body_field() {
        let filter = FilterProcessor::new("status == 'active'").unwrap();
        
        let event = PipelineEvent::new(
            json!({"status": "active", "name": "test"}),
            "test_event".to_string(),
            vec![],
            Operation::Write,
        );
        
        let result = filter.process(event).await.unwrap();
        assert!(result.is_some());
    }
}
