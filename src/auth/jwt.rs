//! JWT validation and JWKS management
//!
//! This module provides JWT token validation with JWKS (JSON Web Key Set) caching
//! and automatic refresh. It validates tokens against OIDC providers like Keycloak.
//!
//! # Security Features
//!
//! - Signature verification using JWKS public keys
//! - Issuer (`iss`) validation
//! - Audience (`aud`) validation (must contain "xzepr-mcp")
//! - Expiration (`exp`) validation
//! - Not-before (`nbf`) validation
//! - JWKS caching with TTL and automatic refresh
//! - Unknown `kid` triggers JWKS refresh

use crate::error::{AuthError, Result};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,

    /// Issuer
    pub iss: String,

    /// Audience
    pub aud: Vec<String>,

    /// Expiration time (Unix timestamp)
    pub exp: i64,

    /// Not before time (Unix timestamp)
    pub nbf: Option<i64>,

    /// Issued at time (Unix timestamp)
    pub iat: Option<i64>,

    /// Scopes (space-separated string)
    #[serde(default)]
    pub scope: String,

    /// Email
    #[serde(default)]
    pub email: Option<String>,

    /// Preferred username
    #[serde(default)]
    pub preferred_username: Option<String>,
}

impl Claims {
    /// Check if the token has the required scope
    ///
    /// # Arguments
    ///
    /// * `required_scope` - The scope to check for (e.g., "xzepr:read")
    ///
    /// # Returns
    ///
    /// Returns `true` if the token contains the required scope
    pub fn has_scope(&self, required_scope: &str) -> bool {
        self.scope.split_whitespace().any(|s| s == required_scope)
    }

    /// Get all scopes as a vector
    pub fn scopes(&self) -> Vec<String> {
        self.scope.split_whitespace().map(String::from).collect()
    }

    /// Get user identifier (sub claim)
    pub fn user_id(&self) -> &str {
        &self.sub
    }

    /// Get display name (preferred username or email or sub)
    pub fn display_name(&self) -> &str {
        self.preferred_username
            .as_deref()
            .or(self.email.as_deref())
            .unwrap_or(&self.sub)
    }
}

/// JWKS (JSON Web Key Set) response
#[derive(Debug, Clone, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

/// JSON Web Key
#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    /// Key ID
    kid: String,
    /// Key type
    kty: String,
    /// Algorithm
    #[allow(dead_code)]
    alg: Option<String>,
    /// Modulus (RSA)
    n: Option<String>,
    /// Exponent (RSA)
    e: Option<String>,
    /// Use (sig or enc)
    #[serde(rename = "use")]
    key_use: Option<String>,
}

/// JWT validator with JWKS caching
#[derive(Clone)]
pub struct JwtValidator {
    /// OIDC provider URL
    oidc_provider_url: String,

    /// Expected issuer
    expected_issuer: String,

    /// Expected audience
    expected_audience: String,

    /// JWKS cache (kid -> DecodingKey)
    jwks_cache: Arc<Cache<String, Arc<DecodingKey>>>,

    /// HTTP client for JWKS fetching
    http_client: reqwest::Client,

    /// JWKS cache TTL
    #[allow(dead_code)]
    cache_ttl: Duration,

    /// Validation enabled flag
    validation_enabled: bool,
}

impl JwtValidator {
    /// Create a new JWT validator
    ///
    /// # Arguments
    ///
    /// * `oidc_provider_url` - OIDC provider URL (e.g., https://keycloak.example.com/realms/xzepr)
    /// * `expected_issuer` - Expected JWT issuer
    /// * `expected_audience` - Expected audience value
    /// * `cache_ttl` - JWKS cache TTL duration
    /// * `validation_enabled` - Enable JWT validation (should be true in production)
    ///
    /// # Returns
    ///
    /// Returns a new `JwtValidator` instance
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xzepr_mcp::auth::JwtValidator;
    /// use std::time::Duration;
    ///
    /// let validator = JwtValidator::new(
    ///     "https://keycloak.example.com/realms/xzepr".to_string(),
    ///     "https://keycloak.example.com/realms/xzepr".to_string(),
    ///     "xzepr-mcp".to_string(),
    ///     Duration::from_secs(3600),
    ///     true,
    /// );
    /// ```
    pub fn new(
        oidc_provider_url: String,
        expected_issuer: String,
        expected_audience: String,
        cache_ttl: Duration,
        validation_enabled: bool,
    ) -> Self {
        let jwks_cache = Cache::builder()
            .time_to_live(cache_ttl)
            .max_capacity(100)
            .build();

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            oidc_provider_url,
            expected_issuer,
            expected_audience,
            jwks_cache: Arc::new(jwks_cache),
            http_client,
            cache_ttl,
            validation_enabled,
        }
    }

    /// Validate a JWT token
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token string (without "Bearer " prefix)
    ///
    /// # Returns
    ///
    /// Returns the decoded claims if validation succeeds
    ///
    /// # Errors
    ///
    /// Returns `AuthError` if validation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use xzepr_mcp::auth::JwtValidator;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let validator = JwtValidator::new(
    ///     "https://keycloak.example.com/realms/xzepr".to_string(),
    ///     "https://keycloak.example.com/realms/xzepr".to_string(),
    ///     "xzepr-mcp".to_string(),
    ///     Duration::from_secs(3600),
    ///     true,
    /// );
    ///
    /// let claims = validator.validate("eyJhbGc...").await?;
    /// println!("User: {}", claims.sub);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate(&self, token: &str) -> Result<Claims> {
        if !self.validation_enabled {
            warn!("JWT validation is disabled - this should only be used in development");
            // Return dummy claims for testing
            return Ok(Claims {
                sub: "test-user".to_string(),
                iss: self.expected_issuer.clone(),
                aud: vec![self.expected_audience.clone()],
                exp: (chrono::Utc::now().timestamp() + 3600),
                nbf: None,
                iat: Some(chrono::Utc::now().timestamp()),
                scope: "xzepr:read xzepr:write".to_string(),
                email: Some("test@example.com".to_string()),
                preferred_username: Some("testuser".to_string()),
            });
        }

        // Extract kid from token header
        let header = decode_header(token).map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        let kid = header
            .kid
            .ok_or_else(|| AuthError::InvalidToken("Missing kid in token header".to_string()))?;

        debug!("Validating token with kid: {}", kid);

        // Get decoding key (from cache or fetch JWKS)
        let decoding_key = self.get_decoding_key(&kid).await?;

        // Set up validation parameters
        let mut validation = Validation::new(
            header
                .alg
                .try_into()
                .map_err(|_| AuthError::InvalidToken("Unsupported algorithm".to_string()))?,
        );
        validation.set_issuer(&[&self.expected_issuer]);
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);

        // Decode and validate token
        let token_data =
            decode::<Claims>(token, &decoding_key, &validation).map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    AuthError::TokenExpired("Token has expired".to_string())
                }
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => AuthError::InvalidIssuer {
                    expected: self.expected_issuer.clone(),
                    actual: "unknown".to_string(),
                },
                _ => AuthError::InvalidToken(e.to_string()),
            })?;

        let claims = token_data.claims;

        // Validate audience contains required value
        if !claims.aud.iter().any(|aud| aud == &self.expected_audience) {
            return Err(AuthError::InvalidAudience.into());
        }

        // Validate nbf (not before) if present
        if let Some(nbf) = claims.nbf {
            let now = chrono::Utc::now().timestamp();
            if now < nbf {
                return Err(AuthError::TokenNotYetValid.into());
            }
        }

        info!(
            "Token validated successfully for user: {} ({})",
            claims.sub,
            claims.display_name()
        );

        Ok(claims)
    }

    /// Get decoding key from cache or fetch JWKS
    async fn get_decoding_key(&self, kid: &str) -> Result<Arc<DecodingKey>> {
        // Check cache first
        if let Some(key) = self.jwks_cache.get(kid).await {
            debug!("Using cached JWKS key for kid: {}", kid);
            return Ok(key);
        }

        debug!(
            "JWKS key not in cache, fetching from provider for kid: {}",
            kid
        );

        // Fetch JWKS and cache all keys
        self.refresh_jwks().await?;

        // Try to get key from cache again
        self.jwks_cache
            .get(kid)
            .await
            .ok_or_else(|| AuthError::KeyIdNotFound(kid.to_string()).into())
    }

    /// Refresh JWKS from provider
    ///
    /// # Errors
    ///
    /// Returns `AuthError::JwksFetchFailed` if JWKS fetch fails
    pub async fn refresh_jwks(&self) -> Result<()> {
        let jwks_url = format!(
            "{}/.well-known/openid-configuration",
            self.oidc_provider_url
        );

        debug!("Fetching OIDC configuration from: {}", jwks_url);

        // Fetch OIDC discovery document
        let oidc_config: serde_json::Value = self
            .http_client
            .get(&jwks_url)
            .send()
            .await
            .map_err(|e| AuthError::JwksFetchFailed(format!("Failed to fetch OIDC config: {}", e)))?
            .json()
            .await
            .map_err(|e| {
                AuthError::JwksFetchFailed(format!("Failed to parse OIDC config: {}", e))
            })?;

        let jwks_endpoint = oidc_config["jwks_uri"].as_str().ok_or_else(|| {
            AuthError::JwksFetchFailed("Missing jwks_uri in OIDC config".to_string())
        })?;

        debug!("Fetching JWKS from: {}", jwks_endpoint);

        // Fetch JWKS
        let jwks: JwksResponse = self
            .http_client
            .get(jwks_endpoint)
            .send()
            .await
            .map_err(|e| AuthError::JwksFetchFailed(format!("Failed to fetch JWKS: {}", e)))?
            .json()
            .await
            .map_err(|e| AuthError::JwksFetchFailed(format!("Failed to parse JWKS: {}", e)))?;

        info!("Fetched {} keys from JWKS", jwks.keys.len());

        // Cache all keys
        for jwk in jwks.keys {
            if jwk.kty != "RSA" {
                debug!("Skipping non-RSA key: {}", jwk.kid);
                continue;
            }

            if jwk.key_use.as_deref() == Some("enc") {
                debug!("Skipping encryption key: {}", jwk.kid);
                continue;
            }

            let n = jwk
                .n
                .as_ref()
                .ok_or_else(|| AuthError::KeyDecodeFailed("Missing n parameter".to_string()))?;
            let e = jwk
                .e
                .as_ref()
                .ok_or_else(|| AuthError::KeyDecodeFailed("Missing e parameter".to_string()))?;

            let decoding_key = DecodingKey::from_rsa_components(n, e)
                .map_err(|e| AuthError::KeyDecodeFailed(e.to_string()))?;

            self.jwks_cache
                .insert(jwk.kid.clone(), Arc::new(decoding_key))
                .await;

            debug!("Cached key: {}", jwk.kid);
        }

        Ok(())
    }

    /// Extract token from Authorization header
    ///
    /// # Arguments
    ///
    /// * `auth_header` - The Authorization header value
    ///
    /// # Returns
    ///
    /// Returns the token string without "Bearer " prefix
    ///
    /// # Errors
    ///
    /// Returns `AuthError::MissingAuthHeader` if header is invalid
    pub fn extract_token(auth_header: &str) -> Result<&str> {
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::MissingAuthHeader)?;

        if token.is_empty() {
            return Err(AuthError::MissingAuthHeader.into());
        }

        Ok(token)
    }

    /// Check if token has required scope
    ///
    /// # Arguments
    ///
    /// * `claims` - The JWT claims
    /// * `required_scope` - Required scope (e.g., "xzepr:read")
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InsufficientPermissions` if scope is missing
    pub fn check_scope(claims: &Claims, required_scope: &str) -> Result<()> {
        if !claims.has_scope(required_scope) {
            return Err(AuthError::InsufficientPermissions {
                required: required_scope.to_string(),
            }
            .into());
        }
        Ok(())
    }

    /// Check if JWKS cache has any keys
    ///
    /// # Returns
    ///
    /// Returns `true` if at least one key is cached
    pub fn has_cached_jwks(&self) -> bool {
        self.jwks_cache.entry_count() > 0
    }

    /// Fetch JWKS from provider (alias for refresh_jwks)
    ///
    /// # Errors
    ///
    /// Returns `AuthError::JwksFetchFailed` if JWKS fetch fails
    pub async fn fetch_jwks(&self) -> Result<()> {
        self.refresh_jwks().await
    }

    /// Validate token (alias for validate)
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token string
    ///
    /// # Returns
    ///
    /// Returns the decoded claims if validation succeeds
    ///
    /// # Errors
    ///
    /// Returns `AuthError` if validation fails
    pub async fn validate_token(&self, token: &str) -> Result<Claims> {
        self.validate(token).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_has_scope() {
        let claims = Claims {
            sub: "user123".to_string(),
            iss: "https://issuer.example.com".to_string(),
            aud: vec!["xzepr-mcp".to_string()],
            exp: 1_234_567_890,
            nbf: None,
            iat: Some(1_234_567_800),
            scope: "xzepr:read xzepr:write openid".to_string(),
            email: Some("user@example.com".to_string()),
            preferred_username: Some("user123".to_string()),
        };

        assert!(claims.has_scope("xzepr:read"));
        assert!(claims.has_scope("xzepr:write"));
        assert!(claims.has_scope("openid"));
        assert!(!claims.has_scope("xzepr:admin"));
    }

    #[test]
    fn test_claims_scopes() {
        let claims = Claims {
            sub: "user123".to_string(),
            iss: "https://issuer.example.com".to_string(),
            aud: vec!["xzepr-mcp".to_string()],
            exp: 1_234_567_890,
            nbf: None,
            iat: Some(1_234_567_800),
            scope: "xzepr:read xzepr:write".to_string(),
            email: None,
            preferred_username: None,
        };

        let scopes = claims.scopes();
        assert_eq!(scopes.len(), 2);
        assert!(scopes.contains(&"xzepr:read".to_string()));
        assert!(scopes.contains(&"xzepr:write".to_string()));
    }

    #[test]
    fn test_claims_display_name() {
        let claims = Claims {
            sub: "user123".to_string(),
            iss: "https://issuer.example.com".to_string(),
            aud: vec!["xzepr-mcp".to_string()],
            exp: 1_234_567_890,
            nbf: None,
            iat: Some(1_234_567_800),
            scope: "".to_string(),
            email: Some("user@example.com".to_string()),
            preferred_username: Some("testuser".to_string()),
        };

        assert_eq!(claims.display_name(), "testuser");

        let claims_no_username = Claims {
            preferred_username: None,
            ..claims.clone()
        };
        assert_eq!(claims_no_username.display_name(), "user@example.com");

        let claims_no_email = Claims {
            email: None,
            preferred_username: None,
            ..claims
        };
        assert_eq!(claims_no_email.display_name(), "user123");
    }

    #[test]
    fn test_extract_token_valid() {
        let auth_header = "Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...";
        let token = JwtValidator::extract_token(auth_header).unwrap();
        assert_eq!(token, "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...");
    }

    #[test]
    fn test_extract_token_invalid() {
        assert!(JwtValidator::extract_token("eyJhbGciOiJSUzI1NiJ9...").is_err());
        assert!(JwtValidator::extract_token("Bearer ").is_err());
        assert!(JwtValidator::extract_token("").is_err());
    }

    #[test]
    fn test_check_scope_success() {
        let claims = Claims {
            sub: "user123".to_string(),
            iss: "https://issuer.example.com".to_string(),
            aud: vec!["xzepr-mcp".to_string()],
            exp: 1_234_567_890,
            nbf: None,
            iat: Some(1_234_567_800),
            scope: "xzepr:read xzepr:write".to_string(),
            email: None,
            preferred_username: None,
        };

        assert!(JwtValidator::check_scope(&claims, "xzepr:read").is_ok());
        assert!(JwtValidator::check_scope(&claims, "xzepr:write").is_ok());
    }

    #[test]
    fn test_check_scope_failure() {
        let claims = Claims {
            sub: "user123".to_string(),
            iss: "https://issuer.example.com".to_string(),
            aud: vec!["xzepr-mcp".to_string()],
            exp: 1_234_567_890,
            nbf: None,
            iat: Some(1_234_567_800),
            scope: "xzepr:read".to_string(),
            email: None,
            preferred_username: None,
        };

        assert!(JwtValidator::check_scope(&claims, "xzepr:write").is_err());
    }

    #[tokio::test]
    async fn test_validator_creation() {
        let validator = JwtValidator::new(
            "https://keycloak.example.com/realms/xzepr".to_string(),
            "https://keycloak.example.com/realms/xzepr".to_string(),
            "xzepr-mcp".to_string(),
            Duration::from_secs(3600),
            true,
        );

        assert_eq!(validator.expected_audience, "xzepr-mcp");
    }

    #[tokio::test]
    async fn test_validation_disabled_mode() {
        let validator = JwtValidator::new(
            "https://keycloak.example.com/realms/xzepr".to_string(),
            "https://keycloak.example.com/realms/xzepr".to_string(),
            "xzepr-mcp".to_string(),
            Duration::from_secs(3600),
            false, // Validation disabled
        );

        let claims = validator.validate("fake-token").await.unwrap();
        assert_eq!(claims.sub, "test-user");
    }
}
