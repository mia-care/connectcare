#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::secret::SecretSource;
    use crate::pipeline::create_pipeline_channel;
    use crate::sources::jira::config::{JiraSourceConfig, JiraAuthentication};
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    
    type HmacSha256 = Hmac<Sha256>;
    
    fn generate_signature(secret: &str, body: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        hex::encode(mac.finalize().into_bytes())
    }
    
    #[tokio::test]
    async fn test_jira_issue_created() {
        let (tx, mut rx) = create_pipeline_channel(100);
        
        let config = JiraSourceConfig {
            webhook_path: "/jira/webhook".to_string(),
            authentication: JiraAuthentication {
                secret: SecretSource::Plain("test_secret".to_string()),
                header_name: "X-Hub-Signature".to_string(),
            },
        };
        
        let app = Router::new();
        let app = register_jira_routes(app, config, tx).unwrap();
        
        let body = r#"{"webhookEvent":"jira:issue_created","issue":{"id":"12345","key":"TEST-123"}}"#;
        let signature = generate_signature("test_secret", body.as_bytes());
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/jira/webhook")
                    .header("X-Hub-Signature", format!("sha256={}", signature))
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let event = rx.recv().await.unwrap();
        assert_eq!(event.event_type, "jira:issue_created");
        assert_eq!(event.pk_fields[0].key, "issue.id");
        assert_eq!(event.pk_fields[0].value, "12345");
    }
    
    #[tokio::test]
    async fn test_jira_invalid_signature() {
        let (tx, _rx) = create_pipeline_channel(100);
        
        let config = JiraSourceConfig {
            webhook_path: "/jira/webhook".to_string(),
            authentication: JiraAuthentication {
                secret: SecretSource::Plain("test_secret".to_string()),
                header_name: "X-Hub-Signature".to_string(),
            },
        };
        
        let app = Router::new();
        let app = register_jira_routes(app, config, tx).unwrap();
        
        let body = r#"{"webhookEvent":"jira:issue_created","issue":{"id":"12345"}}"#;
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/jira/webhook")
                    .header("X-Hub-Signature", "sha256=invalidsignature")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    
    #[tokio::test]
    async fn test_jira_missing_signature() {
        let (tx, _rx) = create_pipeline_channel(100);
        
        let config = JiraSourceConfig {
            webhook_path: "/jira/webhook".to_string(),
            authentication: JiraAuthentication {
                secret: SecretSource::Plain("test_secret".to_string()),
                header_name: "X-Hub-Signature".to_string(),
            },
        };
        
        let app = Router::new();
        let app = register_jira_routes(app, config, tx).unwrap();
        
        let body = r#"{"webhookEvent":"jira:issue_created","issue":{"id":"12345"}}"#;
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/jira/webhook")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
