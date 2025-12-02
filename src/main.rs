use connectcare::{
    config::AppConfig,
    pipeline::{create_pipeline_channel, executor::PipelineExecutor},
    server::run_server,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    
    let use_ansi = atty::is(atty::Stream::Stdout);
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("connectcare={},tower_http=debug", log_level).into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(use_ansi) // Disable ANSI colors in non-terminal environments
        )
        .init();
    
    let config = AppConfig::from_env()?;
    
    let (pipeline_tx, pipeline_rx) = create_pipeline_channel(100);
    
    let executor = PipelineExecutor::new(&config).await?;
    tokio::spawn(async move {
        executor.run(pipeline_rx).await;
    });
    
    run_server(config, pipeline_tx).await?;
    
    Ok(())
}
