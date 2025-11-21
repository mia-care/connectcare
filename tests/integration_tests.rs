use connectcare::{
    config::{AppConfig, ServerConfig, Integration, SourceConfig},
    sources::jira::{JiraSourceConfig, config::{JiraAuthentication}},
    config::secret::SecretSource,
    pipeline::create_pipeline_channel,
    server::routes::create_router,
};
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
async fn test_end_to_end_jira_webhook() {
    let config = AppConfig {
        server: ServerConfig { port: 8080 },
        mongodb: None,
        integrations: vec![Integration {
            source: SourceConfig::Jira(JiraSourceConfig {
                webhook_path: "/jira/webhook".to_string(),
                authentication: JiraAuthentication {
                    secret: SecretSource::Plain("integration_test_secret".to_string()),
                    header_name: "X-Hub-Signature".to_string(),
                },
            }),
            pipelines: vec![],
        }],
    };
    
    let (pipeline_tx, mut pipeline_rx) = create_pipeline_channel(100);
    
    let app = create_router(config, pipeline_tx).unwrap();
    
    // Test issue created event
    let body = r#"{"webhookEvent":"jira:issue_created","issue":{"id":"99291","key":"PROJ-123","fields":{"summary":"Test Issue"}}}"#;
    let signature = generate_signature("integration_test_secret", body.as_bytes());
    
    let response = app.clone()
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
    
    let event = pipeline_rx.recv().await.unwrap();
    assert_eq!(event.event_type, "jira:issue_created");
    assert_eq!(event.pk_fields[0].value, "99291");
    
    // Test health check
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/-/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}
