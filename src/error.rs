use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("HMAC validation failed")]
    HmacValidation,
    
    #[error("Missing signature header")]
    MissingSignature,
    
    #[error("Invalid signature format")]
    InvalidSignatureFormat,
    
    #[error("Event type not found in payload")]
    EventTypeNotFound,
    
    #[error("Unsupported event type: {0}")]
    UnsupportedEvent(String),
    
    #[error("Primary key path not found: {0}")]
    PrimaryKeyPathNotFound(String),
    
    #[error("Failed to send event to pipeline")]
    PipelineSend,
    
    #[error("Processing error: {0}")]
    Processing(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Secret not found: {0}")]
    SecretNotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::HmacValidation => (StatusCode::UNAUTHORIZED, "Invalid signature".to_string()),
            AppError::MissingSignature => (StatusCode::BAD_REQUEST, "Missing signature header".to_string()),
            AppError::InvalidSignatureFormat => (StatusCode::BAD_REQUEST, "Invalid signature format".to_string()),
            AppError::EventTypeNotFound => (StatusCode::BAD_REQUEST, "Event type not found".to_string()),
            AppError::UnsupportedEvent(event) => (StatusCode::BAD_REQUEST, format!("Unsupported event: {}", event)),
            AppError::PrimaryKeyPathNotFound(path) => (StatusCode::BAD_REQUEST, format!("Path not found: {}", path)),
            AppError::PipelineSend => (StatusCode::INTERNAL_SERVER_ERROR, "Pipeline error".to_string()),
            AppError::Processing(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Processing error: {}", e)),
            AppError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)),
            AppError::JsonParse(e) => (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)),
            AppError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("IO error: {}", e)),
            AppError::SecretNotFound(name) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Secret not found: {}", name)),
        };
        
        (status, message).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
