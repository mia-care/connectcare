use connectcare::{
    config::AppConfig,
    pipeline::{create_pipeline_channel, executor::PipelineExecutor},
    server::run_server,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("connectcare={},tower_http=debug", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Load configuration
    let config = AppConfig::from_env()?;
    
    // Create pipeline channel
    let (pipeline_tx, pipeline_rx) = create_pipeline_channel(100);
    
    // Create and spawn pipeline executor
    let executor = PipelineExecutor::new(&config).await?;
    tokio::spawn(async move {
        executor.run(pipeline_rx).await;
    });
    
    // Run HTTP server
    run_server(config, pipeline_tx).await?;
    
    Ok(())
}
