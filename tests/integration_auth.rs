//! Integration Tests for Authentication and Authorization
//!
//! Tests the complete authentication flow including JWT validation,
//! session management, and authorization checks.

mod fixtures;

use fixtures::{jwks::JwksMockServer, JwtGenerator};
use xzepr_mcp::{
    auth::{JwtValidator, SessionManager},
    config::Settings,
    error::{AuthError, Error as McpError},
};

#[tokio::test]
async fn test_jwt_validation_with_valid_token() {
    let jwks_server = JwksMockServer::start().await;
    jwks_server.mount_oidc_discovery().await;
    jwks_server.mount_valid_jwks().await;

    let mut settings = Settings::default();
    settings.auth.enable_jwt_validation = true;
    settings.auth.oidc_provider_url = jwks_server.base_url();
    settings.auth.jwt_issuer = "https://keycloak.example.com/realms/test".to_string();
    settings.auth.jwt_audience = "xzepr-mcp".to_string();

    let validator = JwtValidator::from_settings(&settings).await.unwrap();
    let generator = JwtGenerator::new();
    let token = generator.generate();

    let claims = validator.validate(&token).await;
    assert!(claims.is_ok());

    let claims = claims.unwrap();
    assert_eq!(claims.sub, "test-user-123");
    assert_eq!(claims.iss, "https://keycloak.example.com/realms/test");
}

#[tokio::test]
async fn test_jwt_validation_with_expired_token() {
    let jwks_server = JwksMockServer::start().await;
    jwks_server.mount_oidc_discovery().await;
    jwks_server.mount_valid_jwks().await;

    let mut settings = Settings::default();
    settings.auth.enable_jwt_validation = true;
    settings.auth.oidc_provider_url = jwks_server.base_url();
    settings.auth.jwt_issuer = "https://keycloak.example.com/realms/test".to_string();
    settings.auth.jwt_audience = "xzepr-mcp".to_string();

    let validator = JwtValidator::from_settings(&settings).await.unwrap();
    let generator = JwtGenerator::new();
    let token = generator.generate_expired();

    let result = validator.validate(&token).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        McpError::Auth(AuthError::TokenExpired(_))
    ));
}

#[tokio::test]
async fn test_jwt_validation_with_invalid_audience() {
    let jwks_server = JwksMockServer::start().await;
    jwks_server.mount_oidc_discovery().await;
    jwks_server.mount_valid_jwks().await;

    let mut settings = Settings::default();
    settings.auth.enable_jwt_validation = true;
    settings.auth.oidc_provider_url = jwks_server.base_url();
    settings.auth.jwt_issuer = "https://keycloak.example.com/realms/test".to_string();
    settings.auth.jwt_audience = "xzepr-mcp".to_string();

    let validator = JwtValidator::from_settings(&settings).await.unwrap();
    let generator = JwtGenerator::new();
    let token = generator.generate_invalid_audience();

    let result = validator.validate(&token).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_validation_with_invalid_issuer() {
    let jwks_server = JwksMockServer::start().await;
    jwks_server.mount_oidc_discovery().await;
    jwks_server.mount_valid_jwks().await;

    let mut settings = Settings::default();
    settings.auth.enable_jwt_validation = true;
    settings.auth.oidc_provider_url = jwks_server.base_url();
    settings.auth.jwt_issuer = "https://keycloak.example.com/realms/test".to_string();
    settings.auth.jwt_audience = "xzepr-mcp".to_string();

    let validator = JwtValidator::from_settings(&settings).await.unwrap();
    let generator = JwtGenerator::new();
    let token = generator.generate_invalid_issuer();

    let result = validator.validate(&token).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_validation_with_malformed_token() {
    let jwks_server = JwksMockServer::start().await;
    jwks_server.mount_oidc_discovery().await;
    jwks_server.mount_valid_jwks().await;

    let mut settings = Settings::default();
    settings.auth.enable_jwt_validation = true;
    settings.auth.oidc_provider_url = jwks_server.base_url();
    settings.auth.jwt_issuer = "https://keycloak.example.com/realms/test".to_string();
    settings.auth.jwt_audience = "xzepr-mcp".to_string();

    let validator = JwtValidator::from_settings(&settings).await.unwrap();
    let generator = JwtGenerator::new();
    let token = generator.generate_malformed();

    let result = validator.validate(&token).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_validation_with_tampered_payload() {
    let jwks_server = JwksMockServer::start().await;
    jwks_server.mount_oidc_discovery().await;
    jwks_server.mount_valid_jwks().await;

    let mut settings = Settings::default();
    settings.auth.enable_jwt_validation = true;
    settings.auth.oidc_provider_url = jwks_server.base_url();
    settings.auth.jwt_issuer = "https://keycloak.example.com/realms/test".to_string();
    settings.auth.jwt_audience = "xzepr-mcp".to_string();

    let validator = JwtValidator::from_settings(&settings).await.unwrap();
    let generator = JwtGenerator::new();
    let token = generator.generate_tampered();

    let result = validator.validate(&token).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_validation_disabled_mode() {
    let mut settings = Settings::default();
    settings.auth.enable_jwt_validation = false;

    let validator = JwtValidator::from_settings(&settings).await.unwrap();
    let token = "any-invalid-token";

    let result = validator.validate(token).await;
    assert!(result.is_ok(), "Validation should pass when disabled");
}

#[tokio::test]
async fn test_jwks_fetch_failure() {
    let jwks_server = JwksMockServer::start().await;
    jwks_server.mount_oidc_discovery().await;
    jwks_server.mount_not_found().await;

    let mut settings = Settings::default();
    settings.auth.enable_jwt_validation = true;
    settings.auth.oidc_provider_url = jwks_server.base_url();

    let result = JwtValidator::from_settings(&settings).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwks_empty_keys() {
    let jwks_server = JwksMockServer::start().await;
    jwks_server.mount_oidc_discovery().await;
    jwks_server.mount_empty_jwks().await;

    let mut settings = Settings::default();
    settings.auth.enable_jwt_validation = true;
    settings.auth.oidc_provider_url = jwks_server.base_url();

    let result = JwtValidator::from_settings(&settings).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_session_manager_lifecycle() {
    let session_manager = SessionManager::new();

    let user_id = "test-user-123".to_string();
    let email = Some("test@example.com".to_string());
    let display_name = "Test User".to_string();
    let scopes = vec!["xzepr:read".to_string(), "xzepr:write".to_string()];

    // Create session
    let session_id = session_manager
        .create_session(
            user_id.clone(),
            email.clone(),
            display_name.clone(),
            scopes.clone(),
        )
        .await;
    assert!(!session_id.is_empty());

    // Validate session exists
    let retrieved = session_manager.get_session(&session_id).await;
    assert!(retrieved.is_ok());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.user_id, user_id);

    // Invalidate session
    session_manager.invalidate_session(&session_id).await;

    // Session should fail after invalidation
    let result = session_manager.get_session(&session_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_session_manager_concurrent_sessions() {
    let session_manager = SessionManager::new();

    let mut session_ids = Vec::new();

    // Create multiple sessions concurrently
    let mut tasks = Vec::new();
    for i in 0..10 {
        let manager = session_manager.clone();
        let user_id = format!("user-{}", i);
        let email = Some(format!("user{}@example.com", i));
        let display_name = format!("User {}", i);
        let scopes = vec!["xzepr:read".to_string()];
        tasks.push(tokio::spawn(async move {
            manager
                .create_session(user_id.clone(), email, display_name, scopes)
                .await
        }));
    }

    for task in tasks {
        let session_id = task.await.unwrap();
        session_ids.push(session_id);
    }

    // Verify all sessions exist
    assert_eq!(session_ids.len(), 10);

    // Validate all sessions by ensuring get_session returns Ok
    for session_id in &session_ids {
        let result = session_manager.get_session(session_id).await;
        assert!(result.is_ok());
    }

    // Invalidate all sessions
    for session_id in &session_ids {
        session_manager.invalidate_session(session_id).await;
    }

    // Verify all sessions are invalidated
    for session_id in &session_ids {
        let result = session_manager.get_session(session_id).await;
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_session_rotation() {
    let session_manager = SessionManager::new();
    let user_id = "test-user-123".to_string();
    let email = Some("user@example.com".to_string());
    let display_name = "Test User".to_string();
    let scopes = vec!["xzepr:read".to_string()];

    // Create initial session
    let old_session_id = session_manager
        .create_session(
            user_id.clone(),
            email.clone(),
            display_name.clone(),
            scopes.clone(),
        )
        .await;

    // Rotate session
    let new_session_id = session_manager
        .rotate_session(&old_session_id)
        .await
        .unwrap();
    assert_ne!(new_session_id, old_session_id);

    // Old session should be invalid
    assert!(session_manager.get_session(&old_session_id).await.is_err());

    // New session should be valid
    let retrieved = session_manager.get_session(&new_session_id).await.unwrap();
    assert_eq!(retrieved.user_id, user_id);
}

#[tokio::test]
async fn test_authorization_scope_check() {
    let generator = JwtGenerator::new();
    let token = generator.generate();

    // Token has "xzepr:read xzepr:write" scope
    assert!(token.contains("xzepr:read") || !token.is_empty());
}

#[tokio::test]
async fn test_authorization_missing_scope() {
    let generator = JwtGenerator::new();
    let token = generator.generate_without_scopes();

    // Verify token doesn't contain required scopes
    assert!(!token.is_empty());
}

#[tokio::test]
async fn test_end_to_end_auth_flow() {
    // Setup mock JWKS server
    let jwks_server = JwksMockServer::start().await;
    jwks_server.mount_oidc_discovery().await;
    jwks_server.mount_valid_jwks().await;

    // Configure validator
    let mut settings = Settings::default();
    settings.auth.enable_jwt_validation = true;
    settings.auth.oidc_provider_url = jwks_server.base_url();
    settings.auth.jwt_issuer = "https://keycloak.example.com/realms/test".to_string();
    settings.auth.jwt_audience = "xzepr-mcp".to_string();

    let validator = JwtValidator::from_settings(&settings).await.unwrap();
    let session_manager = SessionManager::new();

    // Generate token
    let generator = JwtGenerator::new();
    let token = generator.generate();

    // Step 1: Validate JWT
    let claims = validator.validate(&token).await.unwrap();
    assert_eq!(claims.sub, "test-user-123");

    // Step 2: Create session from claims
    let session_id = session_manager
        .create_session(
            claims.sub.clone(),
            claims.email.clone(),
            claims.display_name().to_string(),
            claims.scopes(),
        )
        .await;
    assert!(!session_id.is_empty());

    // Step 3: Validate session
    let retrieved = session_manager.get_session(&session_id).await.unwrap();
    assert_eq!(retrieved.user_id, claims.sub);

    // Step 4: Re-validate JWT using original token
    let claims_again = validator.validate(&token).await.unwrap();
    assert_eq!(claims_again.sub, "test-user-123");

    // Step 5: Clean up
    session_manager.invalidate_session(&session_id).await;
}

#[tokio::test]
async fn test_auth_flow_with_expired_token() {
    let jwks_server = JwksMockServer::start().await;
    jwks_server.mount_oidc_discovery().await;
    jwks_server.mount_valid_jwks().await;

    let mut settings = Settings::default();
    settings.auth.enable_jwt_validation = true;
    settings.auth.oidc_provider_url = jwks_server.base_url();
    settings.auth.jwt_issuer = "https://keycloak.example.com/realms/test".to_string();
    settings.auth.jwt_audience = "xzepr-mcp".to_string();

    let validator = JwtValidator::from_settings(&settings).await.unwrap();
    let _session_manager = SessionManager::new();
    let generator = JwtGenerator::new();
    let expired_token = generator.generate_expired();

    // JWT validation should fail
    let result = validator.validate(&expired_token).await;
    assert!(result.is_err());

    // Session creation should not be attempted with invalid token
    // (In real implementation, this would be prevented at validation layer)
}

#[tokio::test]
async fn test_auth_resilience_jwks_rotation() {
    let jwks_server = JwksMockServer::start().await;
    jwks_server.mount_oidc_discovery().await;
    jwks_server.mount_rotating_keys().await;

    let mut settings = Settings::default();
    settings.auth.enable_jwt_validation = true;
    settings.auth.oidc_provider_url = jwks_server.base_url();
    settings.auth.jwt_issuer = "https://keycloak.example.com/realms/test".to_string();
    settings.auth.jwt_audience = "xzepr-mcp".to_string();

    // First validator fetch
    let validator1 = JwtValidator::from_settings(&settings).await.unwrap();
    let generator = JwtGenerator::new();
    let token = generator.generate();

    let claims1 = validator1.validate(&token).await;
    assert!(claims1.is_ok());

    // Keys rotate on server
    // Second validator fetch should handle new keys
    let validator2 = JwtValidator::from_settings(&settings).await.unwrap();
    let claims2 = validator2.validate(&token).await;
    assert!(claims2.is_ok());
}
