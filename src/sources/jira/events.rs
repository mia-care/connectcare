use std::collections::HashMap;
use serde_json::Value;
use crate::error::{AppError, Result};
use crate::pipeline::event::{Operation, PkFields};
use crate::sources::webhook::types::get_primary_key_by_path;

pub mod event_types {
    // Issue events
    pub const ISSUE_CREATED: &str = "jira:issue_created";
    pub const ISSUE_UPDATED: &str = "jira:issue_updated";
    pub const ISSUE_DELETED: &str = "jira:issue_deleted";
    
    // Issue link events
    pub const ISSUELINK_CREATED: &str = "issuelink_created";
    pub const ISSUELINK_DELETED: &str = "issuelink_deleted";
    
    // Project events
    pub const PROJECT_CREATED: &str = "project_created";
    pub const PROJECT_UPDATED: &str = "project_updated";
    pub const PROJECT_DELETED: &str = "project_deleted";
    pub const PROJECT_SOFT_DELETED: &str = "project_soft_deleted";
    pub const PROJECT_RESTORED_DELETED: &str = "project_restored_deleted";
    
    // Version events
    pub const VERSION_RELEASED: &str = "jira:version_released";
    pub const VERSION_UNRELEASED: &str = "jira:version_unreleased";
    pub const VERSION_CREATED: &str = "jira:version_created";
    pub const VERSION_UPDATED: &str = "jira:version_updated";
    pub const VERSION_DELETED: &str = "jira:version_deleted";
}

pub struct EventConfig {
    pub operation: Operation,
    pub get_field_id: Box<dyn Fn(&Value) -> Result<PkFields> + Send + Sync>,
}

pub fn get_supported_events() -> HashMap<String, EventConfig> {
    let mut events = HashMap::new();
    
    // Issue events
    events.insert(
        event_types::ISSUE_CREATED.to_string(),
        EventConfig {
            operation: Operation::Write,
            get_field_id: Box::new(get_primary_key_by_path("issue.id")),
        },
    );
    
    events.insert(
        event_types::ISSUE_UPDATED.to_string(),
        EventConfig {
            operation: Operation::Write,
            get_field_id: Box::new(get_primary_key_by_path("issue.id")),
        },
    );
    
    events.insert(
        event_types::ISSUE_DELETED.to_string(),
        EventConfig {
            operation: Operation::Delete,
            get_field_id: Box::new(get_primary_key_by_path("issue.id")),
        },
    );
    
    // Issue link events
    events.insert(
        event_types::ISSUELINK_CREATED.to_string(),
        EventConfig {
            operation: Operation::Write,
            get_field_id: Box::new(get_primary_key_by_path("issueLink.id")),
        },
    );
    
    events.insert(
        event_types::ISSUELINK_DELETED.to_string(),
        EventConfig {
            operation: Operation::Delete,
            get_field_id: Box::new(get_primary_key_by_path("issueLink.id")),
        },
    );
    
    // Project events
    events.insert(
        event_types::PROJECT_CREATED.to_string(),
        EventConfig {
            operation: Operation::Write,
            get_field_id: Box::new(get_primary_key_by_path("project.id")),
        },
    );
    
    events.insert(
        event_types::PROJECT_UPDATED.to_string(),
        EventConfig {
            operation: Operation::Write,
            get_field_id: Box::new(get_primary_key_by_path("project.id")),
        },
    );
    
    events.insert(
        event_types::PROJECT_DELETED.to_string(),
        EventConfig {
            operation: Operation::Delete,
            get_field_id: Box::new(get_primary_key_by_path("project.id")),
        },
    );
    
    events.insert(
        event_types::PROJECT_SOFT_DELETED.to_string(),
        EventConfig {
            operation: Operation::Delete,
            get_field_id: Box::new(get_primary_key_by_path("project.id")),
        },
    );
    
    events.insert(
        event_types::PROJECT_RESTORED_DELETED.to_string(),
        EventConfig {
            operation: Operation::Write,
            get_field_id: Box::new(get_primary_key_by_path("project.id")),
        },
    );
    
    // Version events
    events.insert(
        event_types::VERSION_RELEASED.to_string(),
        EventConfig {
            operation: Operation::Write,
            get_field_id: Box::new(get_primary_key_by_path("version.id")),
        },
    );
    
    events.insert(
        event_types::VERSION_UNRELEASED.to_string(),
        EventConfig {
            operation: Operation::Write,
            get_field_id: Box::new(get_primary_key_by_path("version.id")),
        },
    );
    
    events.insert(
        event_types::VERSION_CREATED.to_string(),
        EventConfig {
            operation: Operation::Write,
            get_field_id: Box::new(get_primary_key_by_path("version.id")),
        },
    );
    
    events.insert(
        event_types::VERSION_UPDATED.to_string(),
        EventConfig {
            operation: Operation::Write,
            get_field_id: Box::new(get_primary_key_by_path("version.id")),
        },
    );
    
    events.insert(
        event_types::VERSION_DELETED.to_string(),
        EventConfig {
            operation: Operation::Delete,
            get_field_id: Box::new(get_primary_key_by_path("version.id")),
        },
    );
    
    events
}

pub fn get_event_type(body: &Value) -> Result<String> {
    body.get("webhookEvent")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or(AppError::EventTypeNotFound)
}
