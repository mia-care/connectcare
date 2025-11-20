use serde_json::Value;
use crate::error::{AppError, Result};
use crate::pipeline::event::{PkField, PkFields};

pub fn extract_value_by_path<'a>(body: &'a Value, path: &str) -> Result<&'a Value> {
    let mut current = body;
    
    for segment in path.split('.') {
        current = current
            .get(segment)
            .ok_or_else(|| AppError::PrimaryKeyPathNotFound(path.to_string()))?;
    }
    
    Ok(current)
}

pub fn get_primary_key_by_path(path: &'static str) -> impl Fn(&Value) -> Result<PkFields> {
    move |body: &Value| -> Result<PkFields> {
        let value = extract_value_by_path(body, path)?;
        
        // Convert value to string
        let value_str = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => value.to_string(),
        };
        
        Ok(vec![PkField {
            key: path.to_string(),
            value: value_str,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_extract_value_by_path() {
        let body = json!({
            "issue": {
                "id": "12345",
                "key": "TEST-123"
            }
        });
        
        let result = extract_value_by_path(&body, "issue.id").unwrap();
        assert_eq!(result, "12345");
    }
    
    #[test]
    fn test_get_primary_key_by_path() {
        let body = json!({
            "issue": {
                "id": "12345"
            }
        });
        
        let extractor = get_primary_key_by_path("issue.id");
        let pk_fields = extractor(&body).unwrap();
        
        assert_eq!(pk_fields.len(), 1);
        assert_eq!(pk_fields[0].key, "issue.id");
        assert_eq!(pk_fields[0].value, "12345");
    }
}
