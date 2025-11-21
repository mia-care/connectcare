use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use std::sync::Arc;
use serde_json::Value;
use crate::error::{AppError, Result};
use crate::pipeline::{PipelineSender, event::PipelineEvent};
use crate::sources::webhook::hmac::HmacValidator;
use super::events::{EventConfig, get_event_type};
use std::collections::HashMap;

pub struct JiraWebhookState {
    pub validator: HmacValidator,
    pub events: HashMap<String, EventConfig>,
    pub pipeline_tx: PipelineSender,
}

pub async fn handle_jira_webhook(
    State(state): State<Arc<JiraWebhookState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse> {
    // Step 1: Validate HMAC signature
    let signature = headers
        .get(state.validator.header_name())
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::MissingSignature)?;
    
    state.validator.validate(&body, signature)?;
    
    // Step 2: Parse JSON body
    let json_body: Value = serde_json::from_slice(&body)?;
    
    // Step 3: Extract event type
    let event_type = get_event_type(&json_body)?;
    
    // Step 4: Get event configuration (skip if not configured)
    let event_config = match state.events.get(&event_type) {
        Some(config) => config,
        None => {
            tracing::debug!("Event type not configured, accepting but will be filtered: {}", event_type);
            return Ok(StatusCode::OK);
        }
    };
    
    // Step 5: Extract primary keys
    let pk_fields = (event_config.get_field_id)(&json_body)?;
    
    // Step 6: Create pipeline event
    let event = PipelineEvent::new(
        json_body,
        event_type.clone(),
        pk_fields,
        event_config.operation.clone(),
    );
    
    // Step 7: Send to pipeline
    state.pipeline_tx
        .send(event)
        .await
        .map_err(|_| AppError::PipelineSend)?;
    
    tracing::info!("Successfully processed Jira event: {}", event_type);
    
    Ok(StatusCode::OK)
}
