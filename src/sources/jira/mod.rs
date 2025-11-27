pub mod config;
pub mod events;
pub mod handler;

use axum::{Router, routing::post};
use std::sync::Arc;
use crate::error::Result;
use crate::pipeline::PipelineSender;
use crate::sources::webhook::hmac::HmacValidator;
use events::get_supported_events;
use handler::{handle_jira_webhook, JiraWebhookState};

pub use config::JiraSourceConfig;

pub fn register_jira_routes(
    router: Router,
    config: JiraSourceConfig,
    pipeline_tx: PipelineSender,
) -> Result<Router> {
    // Resolve secret
    let secret = config.authentication.secret.resolve()?;
    
    // Create HMAC validator
    let validator = HmacValidator::new(
        secret,
        config.authentication.header_name.clone(),
    );
    
    // Get supported events
    let events = get_supported_events();
    
    let state = Arc::new(JiraWebhookState {
        validator,
        events,
        pipeline_tx,
    });
    
    let webhook_path = config.get_webhook_path();
    let router = router.route(
        &webhook_path,
        post(handle_jira_webhook).with_state(state),
    );
    
    tracing::info!("Registered Jira webhook at: {}", webhook_path);
    
    Ok(router)
}

#[cfg(test)]
mod tests;
