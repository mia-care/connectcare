use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Write,
    Delete,
}

#[derive(Debug, Clone)]
pub struct PkField {
    pub key: String,
    pub value: String,
}

pub type PkFields = Vec<PkField>;

#[derive(Debug, Clone)]
pub struct PipelineEvent {
    pub id: String,
    pub body: Value,
    pub event_type: String,
    pub pk_fields: PkFields,
    pub operation: Operation,
}

impl PipelineEvent {
    pub fn new(
        body: Value,
        event_type: String,
        pk_fields: PkFields,
        operation: Operation,
    ) -> Self {
        let id = Self::generate_id(&pk_fields);
        
        Self {
            id,
            body,
            event_type,
            pk_fields,
            operation,
        }
    }
    
    fn generate_id(pk_fields: &PkFields) -> String {
        let mut hasher = Sha256::new();
        
        for field in pk_fields {
            hasher.update(field.key.as_bytes());
            hasher.update(b":");
            hasher.update(field.value.as_bytes());
            hasher.update(b";");
        }
        
        let result = hasher.finalize();
        hex::encode(result)
    }
}
