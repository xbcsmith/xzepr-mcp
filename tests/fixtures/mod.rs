//! Test Fixtures and Helpers
//!
//! This module provides reusable test fixtures, mock data generators,
//! and helper utilities for unit, integration, and security tests.

#![allow(dead_code)]

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod jwks;
pub mod payloads;
pub mod xzepr;

/// Test JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestClaims {
    pub sub: String,
    pub iss: String,
    pub aud: Vec<String>,
    pub exp: i64,
    pub iat: i64,
    pub azp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_access: Option<serde_json::Value>,
}

impl Default for TestClaims {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            sub: "test-user-123".to_string(),
            iss: "https://keycloak.example.com/realms/test".to_string(),
            aud: vec!["xzepr-mcp".to_string()],
            exp: now + 3600,
            iat: now,
            azp: "xzepr-mcp".to_string(),
            scope: Some("xzepr:read xzepr:write".to_string()),
            resource_access: Some(json!({
                "xzepr-mcp": {
                    "roles": ["user", "admin"]
                }
            })),
        }
    }
}

/// JWT token generator for testing
pub struct JwtGenerator {
    encoding_key: EncodingKey,
    kid: String,
}

impl JwtGenerator {
    /// Create a new JWT generator with RS256 algorithm
    pub fn new() -> Self {
        let private_key = include_bytes!("test_private_key.pem");
        let encoding_key = EncodingKey::from_rsa_pem(private_key).expect("Invalid private key");

        Self {
            encoding_key,
            kid: "test-key-1".to_string(),
        }
    }

    /// Create generator with custom key ID
    pub fn with_kid(mut self, kid: impl Into<String>) -> Self {
        self.kid = kid.into();
        self
    }

    /// Generate a valid JWT token with default claims
    pub fn generate(&self) -> String {
        self.generate_with_claims(TestClaims::default())
    }

    /// Generate JWT with custom claims
    pub fn generate_with_claims(&self, claims: TestClaims) -> String {
        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some(self.kid.clone());

        jsonwebtoken::encode(&header, &claims, &self.encoding_key).expect("Failed to encode JWT")
    }

    /// Generate an expired token
    pub fn generate_expired(&self) -> String {
        let mut claims = TestClaims::default();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        claims.iat = now - 7200;
        claims.exp = now - 3600;

        self.generate_with_claims(claims)
    }

    /// Generate token without required scopes
    pub fn generate_without_scopes(&self) -> String {
        let claims = TestClaims {
            scope: None,
            ..TestClaims::default()
        };

        self.generate_with_claims(claims)
    }

    /// Generate token with invalid audience
    pub fn generate_invalid_audience(&self) -> String {
        let claims = TestClaims {
            aud: vec!["wrong-audience".to_string()],
            ..TestClaims::default()
        };

        self.generate_with_claims(claims)
    }

    /// Generate token with invalid issuer
    pub fn generate_invalid_issuer(&self) -> String {
        let claims = TestClaims {
            iss: "https://attacker.com".to_string(),
            ..TestClaims::default()
        };

        self.generate_with_claims(claims)
    }

    /// Generate a malformed token (invalid signature)
    pub fn generate_malformed(&self) -> String {
        let token = self.generate();
        let parts: Vec<&str> = token.split('.').collect();
        format!("{}.{}.invalid_signature", parts[0], parts[1])
    }

    /// Generate token with tampered payload
    pub fn generate_tampered(&self) -> String {
        let token = self.generate();
        let parts: Vec<&str> = token.split('.').collect();

        let tampered_payload = json!({
            "sub": "attacker",
            "iss": "https://keycloak.example.com/realms/test",
            "aud": "xzepr-mcp",
            "exp": 9999999999i64,
            "iat": 1000000000i64,
            "azp": "xzepr-mcp",
            "scope": "xzepr:admin",
        });

        let encoded_payload =
            URL_SAFE_NO_PAD.encode(serde_json::to_vec(&tampered_payload).unwrap());
        format!("{}.{}.{}", parts[0], encoded_payload, parts[2])
    }

    /// Get the public key for validation
    pub fn public_key() -> DecodingKey {
        let public_key = include_bytes!("test_public_key.pem");
        DecodingKey::from_rsa_pem(public_key).expect("Invalid public key")
    }

    /// Create validation settings for testing
    pub fn validation() -> Validation {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_audience(&["xzepr-mcp"]);
        validation.set_issuer(&["https://keycloak.example.com/realms/test"]);
        validation
    }
}

impl Default for JwtGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create test configuration
pub fn test_config() -> String {
    r#"
server:
  host: "127.0.0.1"
  port: 3000
  transport: "stdio"

xzepr:
  base_url: "https://xzepr.example.com"
  timeout_secs: 30
  max_retries: 3

auth:
  enable_jwt_validation: false
  jwks_url: "https://keycloak.example.com/realms/test/protocol/openid-connect/certs"
  issuer: "https://keycloak.example.com/realms/test"
  audience: "xzepr-mcp"

rate_limit:
  enabled: true
  requests_per_second: 100
  burst_size: 200

observability:
  log_level: "debug"
  enable_tracing: false
  enable_metrics: false
"#
    .to_string()
}

/// Helper to create mock XZepr API responses
pub mod mock_responses {
    use serde_json::json;

    pub fn event_created() -> serde_json::Value {
        json!({
            "id": "01HQKZ5VJ8QXR9BZVW3TN8C6XE",
            "name": "test-event",
            "version": "1.0.0",
            "release": "dev",
            "platform": "linux",
            "arch": "x86_64",
            "commit": "abc123",
            "timestamp": "2024-01-15T10:30:00Z"
        })
    }

    pub fn event_retrieved() -> serde_json::Value {
        json!({
            "id": "01HQKZ5VJ8QXR9BZVW3TN8C6XE",
            "name": "test-event",
            "version": "1.0.0",
            "release": "dev",
            "platform": "linux",
            "arch": "x86_64",
            "commit": "abc123",
            "timestamp": "2024-01-15T10:30:00Z",
            "metadata": {}
        })
    }

    pub fn events_list() -> serde_json::Value {
        json!({
            "events": [
                event_retrieved(),
                {
                    "id": "01HQKZ5VJ8QXR9BZVW3TN8C6XF",
                    "name": "another-event",
                    "version": "2.0.0",
                    "release": "prod",
                    "platform": "darwin",
                    "arch": "arm64",
                    "commit": "def456",
                    "timestamp": "2024-01-15T11:00:00Z",
                    "metadata": {}
                }
            ],
            "total": 2,
            "page": 1,
            "page_size": 10
        })
    }

    pub fn error_not_found() -> serde_json::Value {
        json!({
            "error": "NotFound",
            "message": "Event not found",
            "code": 404
        })
    }

    pub fn error_validation() -> serde_json::Value {
        json!({
            "error": "ValidationError",
            "message": "Invalid event data",
            "code": 400,
            "details": {
                "field": "version",
                "issue": "Invalid semver format"
            }
        })
    }

    pub fn error_unauthorized() -> serde_json::Value {
        json!({
            "error": "Unauthorized",
            "message": "Invalid or missing authentication token",
            "code": 401
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generator_creates_valid_token() {
        let generator = JwtGenerator::new();
        let token = generator.generate();

        assert!(!token.is_empty());
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn test_jwt_generator_with_custom_kid() {
        let generator = JwtGenerator::new().with_kid("custom-key-id");
        let token = generator.generate();

        let header = jsonwebtoken::decode_header(&token).expect("Failed to decode header");
        assert_eq!(header.kid, Some("custom-key-id".to_string()));
    }

    #[test]
    fn test_jwt_generator_expired_token() {
        let generator = JwtGenerator::new();
        let token = generator.generate_expired();

        let result = jsonwebtoken::decode::<TestClaims>(
            &token,
            &JwtGenerator::public_key(),
            &JwtGenerator::validation(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_generator_invalid_audience() {
        let generator = JwtGenerator::new();
        let token = generator.generate_invalid_audience();

        let result = jsonwebtoken::decode::<TestClaims>(
            &token,
            &JwtGenerator::public_key(),
            &JwtGenerator::validation(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_default_claims_valid() {
        let claims = TestClaims::default();
        assert_eq!(claims.sub, "test-user-123");
        assert_eq!(claims.iss, "https://keycloak.example.com/realms/test");
        assert_eq!(claims.aud, vec!["xzepr-mcp".to_string()]);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_config_helper_valid_yaml() {
        let config = test_config();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&config).expect("Invalid YAML");

        assert!(parsed.get("server").is_some());
        assert!(parsed.get("xzepr").is_some());
        assert!(parsed.get("auth").is_some());
    }

    #[test]
    fn test_mock_response_event_created() {
        let response = mock_responses::event_created();
        assert_eq!(response["name"], "test-event");
        assert_eq!(response["version"], "1.0.0");
    }
}
