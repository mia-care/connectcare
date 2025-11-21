use crate::error::{AppError, Result};
use crate::pipeline::event::{PipelineEvent, Operation};
use super::Sink;
use mongodb::{Client, Collection, bson::{self, doc}};
use serde_json::Value;

/// MongoDB database sink
pub struct DatabaseSink {
    client: Client,
    database: String,
    collection: String,
}

impl DatabaseSink {
    pub async fn new(connection_string: &str, database: &str, collection: &str) -> Result<Self> {
        let client = Client::with_uri_str(connection_string)
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to MongoDB: {}", e)))?;
        
        Ok(Self {
            client,
            database: database.to_string(),
            collection: collection.to_string(),
        })
    }
    
    fn get_collection(&self) -> Collection<bson::Document> {
        self.client
            .database(&self.database)
            .collection(&self.collection)
    }
    
    /// Convert serde_json::Value to bson::Document
    fn json_to_bson(&self, value: &Value) -> Result<bson::Document> {
        let bson_value = bson::to_bson(value)
            .map_err(|e| AppError::Processing(format!("Failed to convert JSON to BSON: {}", e)))?;
        
        match bson_value {
            bson::Bson::Document(doc) => Ok(doc),
            _ => Err(AppError::Processing("Expected JSON object for BSON conversion".to_string())),
        }
    }
}

#[async_trait::async_trait]
impl Sink for DatabaseSink {
    async fn write(&self, event: &PipelineEvent) -> Result<()> {
        let collection = self.get_collection();
        
        match event.operation {
            Operation::Write => {
                // Convert the event body to BSON
                let mut document = self.json_to_bson(&event.body)?;
                
                // Add the event ID and type to the document
                document.insert("_id", &event.id);
                document.insert("_eventType", &event.event_type);
                
                // Upsert the document
                collection
                    .replace_one(doc! { "_id": &event.id }, document)
                    .upsert(true)
                    .await
                    .map_err(|e| AppError::Database(format!("Failed to write to MongoDB: {}", e)))?;
            }
            Operation::Delete => {
                // Delete the document by ID
                collection
                    .delete_one(doc! { "_id": &event.id })
                    .await
                    .map_err(|e| AppError::Database(format!("Failed to delete from MongoDB: {}", e)))?;
            }
        }
        
        Ok(())
    }
}
