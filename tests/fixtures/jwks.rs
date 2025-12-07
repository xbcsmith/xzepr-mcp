//! JWKS Mock Server and Fixtures
//!
//! Provides mock JWKS endpoints and key management for testing
//! JWT validation without requiring a real Keycloak instance.

#![allow(dead_code)]

use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// JWKS mock server for testing
pub struct JwksMockServer {
    server: MockServer,
}

impl JwksMockServer {
    /// Start a new JWKS mock server
    pub async fn start() -> Self {
        let server = MockServer::start().await;
        Self { server }
    }

    /// Get the base OIDC provider URL for this mock server
    pub fn base_url(&self) -> String {
        self.server.uri().to_string()
    }

    /// Get the JWKS URL for this mock server
    pub fn url(&self) -> String {
        format!(
            "{}/realms/test/protocol/openid-connect/certs",
            self.server.uri()
        )
    }

    /// Mount OIDC discovery document
    pub async fn mount_oidc_discovery(&self) {
        let discovery = json!({
            "issuer": "https://keycloak.example.com/realms/test",
            "authorization_endpoint": format!("{}/realms/test/protocol/openid-connect/auth", self.server.uri()),
            "token_endpoint": format!("{}/realms/test/protocol/openid-connect/token", self.server.uri()),
            "userinfo_endpoint": format!("{}/realms/test/protocol/openid-connect/userinfo", self.server.uri()),
            "jwks_uri": format!("{}/realms/test/protocol/openid-connect/certs", self.server.uri()),
            "response_types_supported": ["code", "token", "id_token"],
            "subject_types_supported": ["public"],
            "id_token_signing_alg_values_supported": ["RS256"],
            "scopes_supported": ["openid", "profile", "email"]
        });

        Mock::given(method("GET"))
            .and(path("/.well-known/openid-configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(discovery))
            .mount(&self.server)
            .await;
    }

    /// Mount valid JWKS response
    pub async fn mount_valid_jwks(&self) {
        let jwks = valid_jwks_response();

        Mock::given(method("GET"))
            .and(path("/realms/test/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(jwks))
            .mount(&self.server)
            .await;
    }

    /// Mount empty JWKS response
    pub async fn mount_empty_jwks(&self) {
        let jwks = json!({
            "keys": []
        });

        Mock::given(method("GET"))
            .and(path("/realms/test/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(jwks))
            .mount(&self.server)
            .await;
    }

    /// Mount invalid JWKS response (malformed JSON)
    pub async fn mount_invalid_jwks(&self) {
        Mock::given(method("GET"))
            .and(path("/realms/test/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
            .mount(&self.server)
            .await;
    }

    /// Mount 404 not found response
    pub async fn mount_not_found(&self) {
        Mock::given(method("GET"))
            .and(path("/realms/test/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&self.server)
            .await;
    }

    /// Mount 500 internal server error
    pub async fn mount_server_error(&self) {
        Mock::given(method("GET"))
            .and(path("/realms/test/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&self.server)
            .await;
    }

    /// Mount timeout response (delayed)
    pub async fn mount_timeout(&self) {
        Mock::given(method("GET"))
            .and(path("/realms/test/protocol/openid-connect/certs"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(valid_jwks_response())
                    .set_delay(std::time::Duration::from_secs(60)),
            )
            .mount(&self.server)
            .await;
    }

    /// Mount JWKS with wrong key (different from test keys)
    pub async fn mount_wrong_key(&self) {
        let jwks = json!({
            "keys": [
                {
                    "kty": "RSA",
                    "use": "sig",
                    "kid": "wrong-key-id",
                    "alg": "RS256",
                    "n": "xGOr-H7A-PWxjJRjnpJwzk5fj0p9zMFYZlHKb0pW3TxJhZQ_IvBzQzMmJhFzJnGx",
                    "e": "AQAB"
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/realms/test/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(jwks))
            .mount(&self.server)
            .await;
    }

    /// Mount JWKS that changes between requests (key rotation simulation)
    pub async fn mount_rotating_keys(&self) {
        let jwks1 = valid_jwks_response();
        let jwks2 = json!({
            "keys": [
                jwks1["keys"][0].clone(),
                {
                    "kty": "RSA",
                    "use": "sig",
                    "kid": "new-key-id",
                    "alg": "RS256",
                    "n": "xGOr-H7A-PWxjJRjnpJwzk5fj0p9zMFYZlHKb0pW3TxJhZQ_IvBzQzMmJhFzJnGx",
                    "e": "AQAB"
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/realms/test/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(jwks1))
            .up_to_n_times(1)
            .mount(&self.server)
            .await;

        Mock::given(method("GET"))
            .and(path("/realms/test/protocol/openid-connect/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(jwks2))
            .mount(&self.server)
            .await;
    }
}

/// Generate valid JWKS response matching test keys
pub fn valid_jwks_response() -> serde_json::Value {
    let public_key_pem = include_str!("test_public_key.pem");
    let n = extract_modulus_from_pem(public_key_pem);
    let e = "AQAB"; // Standard RSA exponent (65537)

    json!({
        "keys": [
            {
                "kty": "RSA",
                "use": "sig",
                "kid": "test-key-1",
                "alg": "RS256",
                "n": n,
                "e": e
            }
        ]
    })
}

/// Extract modulus (n) from PEM formatted public key
/// This is a simplified extraction for testing purposes
fn extract_modulus_from_pem(_pem: &str) -> String {
    // Pre-computed base64url-encoded modulus for the test RSA public key
    // This was extracted using: openssl rsa -pubin -in test_public_key.pem -modulus -noout
    // Then converted to base64url encoding
    //
    // The modulus (n) and exponent (e=AQAB/65537) together form the RSA public key
    // that matches the test_private_key.pem used for signing test JWTs
    "qDeFRO7aROdN88YHtlm4nfIc_eGKUn4D1YHZPsv62athkdUMFa3LvPZN2zxJ9uxu2pkmsUjh8jMtUDF0Ly6jj373O9_dEn8Jw25vBFCZap7t13Q2-RcWJiQY-ohuWDbJ6lYKtp774wqnZKohcRiuLIBWMx58Mgzcg1wuzf46Pf2rfLPxypXUl2iEpDZpX6vnUojtCqeRhMEbfeW8v6S8fWl8L_INCss5Pr2oVHI8WjEn7-b9Og8ZbetXo8u1LbL0XIWQixEv9mLvxfxHFMqWnxI-i_7_Wq6hiGWarn2j7QMZRBZQBKX3MCkjFIsu-KKsIHOMlG_dNyYRgFfCteFM2w".to_string()
}

/// Create a minimal valid JWK for testing
pub fn test_jwk() -> serde_json::Value {
    json!({
        "kty": "RSA",
        "use": "sig",
        "kid": "test-key-1",
        "alg": "RS256",
        "n": "xGOr-H7A-PWxjJRjnpJwzk5fj0p9zMFYZlHKb0pW3TxJhZQ_IvBzQzMmJhFzJnGx",
        "e": "AQAB"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jwks_mock_server_starts() {
        let server = JwksMockServer::start().await;
        assert!(server.url().starts_with("http://"));
        assert!(server
            .url()
            .contains("/realms/test/protocol/openid-connect/certs"));
    }

    #[tokio::test]
    async fn test_mount_valid_jwks() {
        let server = JwksMockServer::start().await;
        server.mount_valid_jwks().await;

        let response = reqwest::get(&server.url()).await.expect("Request failed");
        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.expect("Invalid JSON");
        assert!(body["keys"].is_array());
        assert!(!body["keys"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_mount_empty_jwks() {
        let server = JwksMockServer::start().await;
        server.mount_empty_jwks().await;

        let response = reqwest::get(&server.url()).await.expect("Request failed");
        let body: serde_json::Value = response.json().await.expect("Invalid JSON");
        assert_eq!(body["keys"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_mount_not_found() {
        let server = JwksMockServer::start().await;
        server.mount_not_found().await;

        let response = reqwest::get(&server.url()).await.expect("Request failed");
        assert_eq!(response.status(), 404);
    }

    #[tokio::test]
    async fn test_valid_jwks_response_structure() {
        let jwks = valid_jwks_response();

        assert!(jwks["keys"].is_array());
        let keys = jwks["keys"].as_array().unwrap();
        assert_eq!(keys.len(), 1);

        let key = &keys[0];
        assert_eq!(key["kty"], "RSA");
        assert_eq!(key["use"], "sig");
        assert_eq!(key["kid"], "test-key-1");
        assert_eq!(key["alg"], "RS256");
        assert!(key["n"].is_string());
        assert_eq!(key["e"], "AQAB");
    }

    #[test]
    fn test_jwk_structure() {
        let jwk = test_jwk();
        assert_eq!(jwk["kty"], "RSA");
        assert_eq!(jwk["kid"], "test-key-1");
    }

    #[tokio::test]
    async fn test_mount_oidc_discovery() {
        let server = JwksMockServer::start().await;
        server.mount_oidc_discovery().await;

        let response = reqwest::get(format!(
            "{}/.well-known/openid-configuration",
            server.base_url()
        ))
        .await
        .expect("Request failed");
        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.expect("Invalid JSON");
        assert_eq!(body["issuer"], "https://keycloak.example.com/realms/test");
        assert!(body["jwks_uri"]
            .as_str()
            .unwrap()
            .contains("/realms/test/protocol/openid-connect/certs"));
    }
}
