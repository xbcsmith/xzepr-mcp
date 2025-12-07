//! XZepr API Mock Server
//!
//! Provides mock XZepr API endpoints for integration testing
//! without requiring a real XZepr instance.

#![allow(dead_code)]

use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use wiremock::{
    matchers::{header, method, path, path_regex},
    Mock, MockServer, Request, ResponseTemplate,
};

/// XZepr API mock server with state tracking
pub struct XzeprMockServer {
    server: MockServer,
    state: Arc<RwLock<ServerState>>,
}

#[derive(Debug, Clone, Default)]
struct ServerState {
    events: Vec<serde_json::Value>,
    request_count: usize,
}

impl XzeprMockServer {
    /// Start a new XZepr mock server
    pub async fn start() -> Self {
        let server = MockServer::start().await;
        let state = Arc::new(RwLock::new(ServerState::default()));

        Self { server, state }
    }

    /// Get the base URL for this mock server
    pub fn url(&self) -> String {
        self.server.uri()
    }

    /// Mount all standard XZepr API endpoints
    pub async fn mount_all(&self) {
        self.mount_health_check().await;
        self.mount_create_event().await;
        self.mount_get_event().await;
        self.mount_list_events().await;
        self.mount_search_events().await;
        self.mount_update_event().await;
        self.mount_delete_event().await;
    }

    /// Mount health check endpoint
    pub async fn mount_health_check(&self) {
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "healthy",
                "version": "1.0.0"
            })))
            .mount(&self.server)
            .await;
    }

    /// Mount create event endpoint (POST /api/v1/events)
    pub async fn mount_create_event(&self) {
        let state = self.state.clone();

        Mock::given(method("POST"))
            .and(path("/api/v1/events"))
            .and(header("Authorization", "Bearer valid-token"))
            .respond_with(move |req: &Request| {
                let body: serde_json::Value = req.body_json().unwrap_or(json!({}));

                // Generate ULID for new event
                let event_id = "01HQKZ5VJ8QXR9BZVW3TN8C6XE";
                let mut event = body.clone();
                event["id"] = json!(event_id);
                event["timestamp"] = json!("2024-01-15T10:30:00Z");

                // Store in state
                let event_for_state = event.clone();
                let state_clone = state.clone();
                tokio::spawn(async move {
                    let mut state = state_clone.write().await;
                    state.events.push(event_for_state);
                    state.request_count += 1;
                });

                ResponseTemplate::new(201).set_body_json(event)
            })
            .mount(&self.server)
            .await;
    }

    /// Mount create event with validation errors
    pub async fn mount_create_event_validation_error(&self) {
        Mock::given(method("POST"))
            .and(path("/api/v1/events"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                "error": "ValidationError",
                "message": "Invalid event data",
                "details": {
                    "field": "version",
                    "issue": "Invalid semver format"
                }
            })))
            .mount(&self.server)
            .await;
    }

    /// Mount create event with authentication error
    pub async fn mount_create_event_auth_error(&self) {
        Mock::given(method("POST"))
            .and(path("/api/v1/events"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authentication token"
            })))
            .mount(&self.server)
            .await;
    }

    /// Mount get event endpoint (GET /api/v1/events/:id)
    pub async fn mount_get_event(&self) {
        Mock::given(method("GET"))
            .and(path_regex("/api/v1/events/[A-Z0-9]+"))
            .and(header("Authorization", "Bearer valid-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "01HQKZ5VJ8QXR9BZVW3TN8C6XE",
                "name": "test-event",
                "version": "1.0.0",
                "release": "dev",
                "platform": "linux",
                "arch": "x86_64",
                "commit": "abc123",
                "timestamp": "2024-01-15T10:30:00Z",
                "metadata": {}
            })))
            .mount(&self.server)
            .await;
    }

    /// Mount get event not found
    pub async fn mount_get_event_not_found(&self) {
        Mock::given(method("GET"))
            .and(path_regex("/api/v1/events/[A-Z0-9]+"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({
                "error": "NotFound",
                "message": "Event not found"
            })))
            .mount(&self.server)
            .await;
    }

    /// Mount list events endpoint (GET /api/v1/events)
    pub async fn mount_list_events(&self) {
        Mock::given(method("GET"))
            .and(path("/api/v1/events"))
            .and(header("Authorization", "Bearer valid-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "events": [
                    {
                        "id": "01HQKZ5VJ8QXR9BZVW3TN8C6XE",
                        "name": "test-event-1",
                        "version": "1.0.0",
                        "release": "dev",
                        "platform": "linux",
                        "arch": "x86_64",
                        "commit": "abc123",
                        "timestamp": "2024-01-15T10:30:00Z"
                    },
                    {
                        "id": "01HQKZ5VJ8QXR9BZVW3TN8C6XF",
                        "name": "test-event-2",
                        "version": "2.0.0",
                        "release": "prod",
                        "platform": "darwin",
                        "arch": "arm64",
                        "commit": "def456",
                        "timestamp": "2024-01-15T11:00:00Z"
                    }
                ],
                "total": 2,
                "page": 1,
                "page_size": 10
            })))
            .mount(&self.server)
            .await;
    }

    /// Mount search events endpoint (POST /api/v1/events/search)
    pub async fn mount_search_events(&self) {
        Mock::given(method("POST"))
            .and(path("/api/v1/events/search"))
            .and(header("Authorization", "Bearer valid-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "events": [
                    {
                        "id": "01HQKZ5VJ8QXR9BZVW3TN8C6XE",
                        "name": "matching-event",
                        "version": "1.0.0",
                        "release": "dev",
                        "platform": "linux",
                        "arch": "x86_64",
                        "commit": "abc123",
                        "timestamp": "2024-01-15T10:30:00Z"
                    }
                ],
                "total": 1,
                "page": 1,
                "page_size": 10
            })))
            .mount(&self.server)
            .await;
    }

    /// Mount update event endpoint (PUT /api/v1/events/:id)
    pub async fn mount_update_event(&self) {
        Mock::given(method("PUT"))
            .and(path_regex("/api/v1/events/[A-Z0-9]+"))
            .and(header("Authorization", "Bearer valid-token"))
            .respond_with(move |req: &Request| {
                let body: serde_json::Value = req.body_json().unwrap_or(json!({}));
                let mut event = body.clone();
                event["id"] = json!("01HQKZ5VJ8QXR9BZVW3TN8C6XE");
                event["timestamp"] = json!("2024-01-15T10:30:00Z");

                ResponseTemplate::new(200).set_body_json(event)
            })
            .mount(&self.server)
            .await;
    }

    /// Mount delete event endpoint (DELETE /api/v1/events/:id)
    pub async fn mount_delete_event(&self) {
        Mock::given(method("DELETE"))
            .and(path_regex("/api/v1/events/[A-Z0-9]+"))
            .and(header("Authorization", "Bearer valid-token"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&self.server)
            .await;
    }

    /// Mount endpoint that returns 500 error
    pub async fn mount_server_error(&self) {
        Mock::given(method("POST"))
            .and(path("/api/v1/events"))
            .respond_with(ResponseTemplate::new(500).set_body_json(json!({
                "error": "InternalServerError",
                "message": "An internal error occurred"
            })))
            .mount(&self.server)
            .await;
    }

    /// Mount endpoint with rate limit error
    pub async fn mount_rate_limit_error(&self) {
        Mock::given(method("POST"))
            .and(path("/api/v1/events"))
            .respond_with(
                ResponseTemplate::new(429)
                    .set_body_json(json!({
                        "error": "TooManyRequests",
                        "message": "Rate limit exceeded"
                    }))
                    .insert_header("Retry-After", "60"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mount endpoint with timeout (delayed response)
    pub async fn mount_timeout(&self) {
        Mock::given(method("POST"))
            .and(path("/api/v1/events"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"id": "01HQKZ5VJ8QXR9BZVW3TN8C6XE"}))
                    .set_delay(std::time::Duration::from_secs(60)),
            )
            .mount(&self.server)
            .await;
    }

    /// Mount endpoint that rejects malicious payloads
    pub async fn mount_injection_detection(&self) {
        Mock::given(method("POST"))
            .and(path("/api/v1/events"))
            .respond_with(move |req: &Request| {
                let body: serde_json::Value = req.body_json().unwrap_or(json!({}));
                let body_str = body.to_string();

                // Check for common injection patterns
                let suspicious_patterns = [
                    "DROP TABLE",
                    "<script>",
                    "'; --",
                    "../",
                    "&&",
                    "||",
                    "__proto__",
                ];

                for pattern in &suspicious_patterns {
                    if body_str.contains(pattern) {
                        return ResponseTemplate::new(400).set_body_json(json!({
                            "error": "ValidationError",
                            "message": "Suspicious input detected",
                            "details": {
                                "reason": "Potential injection attack"
                            }
                        }));
                    }
                }

                // Valid request
                ResponseTemplate::new(201).set_body_json(json!({
                    "id": "01HQKZ5VJ8QXR9BZVW3TN8C6XE",
                    "name": body["name"],
                    "version": body["version"]
                }))
            })
            .mount(&self.server)
            .await;
    }

    /// Get request count from state
    pub async fn request_count(&self) -> usize {
        self.state.read().await.request_count
    }

    /// Get stored events from state
    pub async fn stored_events(&self) -> Vec<serde_json::Value> {
        self.state.read().await.events.clone()
    }

    /// Reset server state
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        state.events.clear();
        state.request_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_xzepr_mock_server_starts() {
        let server = XzeprMockServer::start().await;
        assert!(server.url().starts_with("http://"));
    }

    #[tokio::test]
    async fn test_health_check_endpoint() {
        let server = XzeprMockServer::start().await;
        server.mount_health_check().await;

        let response = reqwest::get(&format!("{}/health", server.url()))
            .await
            .expect("Request failed");

        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.expect("Invalid JSON");
        assert_eq!(body["status"], "healthy");
    }

    #[tokio::test]
    async fn test_create_event_success() {
        let server = XzeprMockServer::start().await;
        server.mount_create_event().await;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/v1/events", server.url()))
            .header("Authorization", "Bearer valid-token")
            .json(&json!({
                "name": "test-event",
                "version": "1.0.0"
            }))
            .send()
            .await
            .expect("Request failed");

        assert_eq!(response.status(), 201);
        let body: serde_json::Value = response.json().await.expect("Invalid JSON");
        assert!(body["id"].is_string());
    }

    #[tokio::test]
    async fn test_get_event_success() {
        let server = XzeprMockServer::start().await;
        server.mount_get_event().await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "{}/api/v1/events/01HQKZ5VJ8QXR9BZVW3TN8C6XE",
                server.url()
            ))
            .header("Authorization", "Bearer valid-token")
            .send()
            .await
            .expect("Request failed");

        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.expect("Invalid JSON");
        assert_eq!(body["name"], "test-event");
    }

    #[tokio::test]
    async fn test_list_events_success() {
        let server = XzeprMockServer::start().await;
        server.mount_list_events().await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/api/v1/events", server.url()))
            .header("Authorization", "Bearer valid-token")
            .send()
            .await
            .expect("Request failed");

        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.expect("Invalid JSON");
        assert_eq!(body["events"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_server_state_tracking() {
        let server = XzeprMockServer::start().await;
        assert_eq!(server.request_count().await, 0);
        assert_eq!(server.stored_events().await.len(), 0);
    }
}
