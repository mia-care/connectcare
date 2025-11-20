use serde::{Deserialize, Serialize};
use crate::error::{AppError, Result};
use std::fs;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SecretSource {
    Plain(String),
    FromEnv { 
        #[serde(rename = "fromEnv")]
        from_env: String 
    },
    FromFile { 
        #[serde(rename = "fromFile")]
        from_file: String 
    },
}

impl SecretSource {
    pub fn resolve(&self) -> Result<String> {
        match self {
            SecretSource::Plain(value) => Ok(value.clone()),
            SecretSource::FromEnv { from_env } => {
                std::env::var(from_env)
                    .map_err(|_| AppError::SecretNotFound(from_env.clone()))
            }
            SecretSource::FromFile { from_file } => {
                fs::read_to_string(from_file)
                    .map(|s| s.trim().to_string())
                    .map_err(|_| AppError::SecretNotFound(from_file.clone()))
            }
        }
    }
}
