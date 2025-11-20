pub mod routes;

use std::net::SocketAddr;
use tokio::net::TcpListener;
use crate::config::AppConfig;
use crate::pipeline::PipelineSender;
use crate::error::Result;

pub async fn run_server(config: AppConfig, pipeline_tx: PipelineSender) -> Result<()> {
    let router = routes::create_router(config.clone(), pipeline_tx)?;
    
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let listener = TcpListener::bind(addr).await?;
    
    tracing::info!("Server listening on {}", addr);
    
    axum::serve(listener, router)
        .await
        .map_err(|e| crate::error::AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    Ok(())
}
