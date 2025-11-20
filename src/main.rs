use integration_connector_agent::{
    config::AppConfig,
    pipeline::create_pipeline_channel,
    server::run_server,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "integration_connector_agent=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Load configuration
    let config = AppConfig::from_env()?;
    
    // Create pipeline channel
    let (pipeline_tx, mut pipeline_rx) = create_pipeline_channel(100);
    
    // Spawn pipeline processor task
    tokio::spawn(async move {
        while let Some(event) = pipeline_rx.recv().await {
            tracing::info!(
                "Received event: type={}, id={}, operation={:?}",
                event.event_type,
                event.id,
                event.operation
            );
            // TODO: Process event through pipeline processors and sinks
        }
    });
    
    // Run HTTP server
    run_server(config, pipeline_tx).await?;
    
    Ok(())
}
