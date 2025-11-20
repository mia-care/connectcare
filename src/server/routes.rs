use axum::{Router, routing::get, http::StatusCode};
use crate::config::AppConfig;
use crate::config::SourceConfig;
use crate::pipeline::PipelineSender;
use crate::sources::jira;
use crate::error::Result;

async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub fn create_router(config: AppConfig, pipeline_tx: PipelineSender) -> Result<Router> {
    let mut router = Router::new()
        .route("/-/healthz", get(health_check))
        .route("/-/ready", get(health_check));
    
    // Register source routes
    for integration in config.integrations {
        router = match integration.source {
            SourceConfig::Jira(jira_config) => {
                jira::register_jira_routes(router, jira_config, pipeline_tx.clone())?
            }
        };
    }
    
    Ok(router)
}
