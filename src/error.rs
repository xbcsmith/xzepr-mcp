//! Error types and handling for XZepr MCP
//!
//! This module defines all error types used throughout the application,
//! with proper error propagation and context preservation.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

/// Main error type for XZepr MCP operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Authentication and authorization errors
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    /// Input validation errors
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// HTTP client errors
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] HttpClientError),

    /// Rate limiting errors
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// MCP protocol errors
    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    /// XZepr API errors
    #[error("XZepr API error: {0}")]
    XzeprApi(#[from] XzeprApiError),

    /// Internal server errors
    #[error("Internal error: {0}")]
    Internal(String),

    /// External dependency errors
    #[error("External service error: {0}")]
    External(String),
}

/// Configuration-related errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Failed to read configuration file from disk
    #[error("Failed to read config file: {0}")]
    ReadFailed(String),

    /// YAML parsing error in configuration file
    #[error("Invalid YAML syntax: {0}")]
    ParseError(String),

    /// Required configuration field is missing
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    /// Configuration value is invalid
    #[error("Invalid configuration value for {field}: {reason}")]
    InvalidValue {
        /// The field name that has an invalid value
        field: String,
        /// The reason why the value is invalid
        reason: String,
    },

    /// Environment variable error during configuration loading
    #[error("Environment variable error: {0}")]
    EnvError(String),
}

/// Authentication and authorization errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// JWT token is invalid or malformed
    #[error("Invalid JWT token: {0}")]
    InvalidToken(String),

    /// JWT token has expired
    #[error("Token expired at {0}")]
    TokenExpired(String),

    /// JWT token is not yet valid (nbf claim is in the future)
    #[error("Token not yet valid (nbf claim)")]
    TokenNotYetValid,

    /// JWT signature verification failed
    #[error("Invalid token signature")]
    InvalidSignature,

    /// Required claim is missing from JWT token
    #[error("Missing required claim: {0}")]
    MissingClaim(String),

    /// JWT issuer does not match expected value
    #[error("Invalid issuer: expected {expected}, got {actual}")]
    InvalidIssuer {
        /// The expected issuer value
        expected: String,
        /// The actual issuer value from the token
        actual: String,
    },

    /// JWT audience does not include required value
    #[error("Invalid audience: token does not contain required audience 'xzepr-mcp'")]
    InvalidAudience,

    /// Authorization header is missing or malformed
    #[error("Missing or invalid authorization header")]
    MissingAuthHeader,

    /// User lacks required scope/permission
    #[error("Insufficient permissions: required scope '{required}' not found")]
    InsufficientPermissions {
        /// The required scope that was not found
        required: String,
    },

    /// Failed to fetch JWKS from OIDC provider
    #[error("JWKS fetch failed: {0}")]
    JwksFetchFailed(String),

    /// Key ID from JWT header not found in JWKS
    #[error("Key ID (kid) not found in JWKS: {0}")]
    KeyIdNotFound(String),

    /// Failed to decode public key from JWKS
    #[error("Failed to decode public key: {0}")]
    KeyDecodeFailed(String),

    /// Session not found or has expired
    #[error("Session not found or expired")]
    SessionNotFound,

    /// Session validation failed
    #[error("Session validation failed: {0}")]
    SessionValidationFailed(String),
}

/// Input validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// ULID format is invalid
    #[error("Invalid ULID format: {0}")]
    InvalidUlid(String),

    /// UUID format is invalid
    #[error("Invalid UUID format: {0}")]
    InvalidUuid(String),

    /// Semantic version format is invalid
    #[error("Invalid semver format: {0}")]
    InvalidSemver(String),

    /// Field validation failed
    #[error("Field '{field}' validation failed: {reason}")]
    FieldValidation {
        /// The field name that failed validation
        field: String,
        /// The reason for validation failure
        reason: String,
    },

    /// Request payload exceeds size limit
    #[error("Payload too large: {size} bytes exceeds maximum of {max} bytes")]
    PayloadTooLarge {
        /// Actual size of the payload in bytes
        size: usize,
        /// Maximum allowed size in bytes
        max: usize,
    },

    /// JSON nesting depth exceeds limit
    #[error("JSON nesting too deep: {depth} exceeds maximum of {max}")]
    JsonTooDeep {
        /// Actual nesting depth
        depth: usize,
        /// Maximum allowed nesting depth
        max: usize,
    },

    /// Potential injection attack detected
    #[error("Potential injection detected in field '{field}': {reason}")]
    PotentialInjection {
        /// The field where injection was detected
        field: String,
        /// Description of the detected injection pattern
        reason: String,
    },

    /// Query parameter validation failed
    #[error("Invalid query parameter '{param}': {reason}")]
    InvalidQueryParam {
        /// The parameter name that failed validation
        param: String,
        /// The reason for validation failure
        reason: String,
    },

    /// JSON schema validation failed
    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    /// Required field is missing from input
    #[error("Required field missing: {0}")]
    RequiredFieldMissing(String),

    /// Enum field has invalid value
    #[error("Invalid enum value for '{field}': {value}")]
    InvalidEnumValue {
        /// The field name with invalid enum value
        field: String,
        /// The invalid value that was provided
        value: String,
    },
}

/// HTTP client errors
#[derive(Debug, thiserror::Error)]
pub enum HttpClientError {
    /// HTTP request failed
    #[error("Request failed: {0}")]
    RequestFailed(String),

    /// Failed to establish connection
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Request timed out
    #[error("Request timeout after {timeout_ms}ms")]
    Timeout {
        /// Timeout duration in milliseconds
        timeout_ms: u64,
    },

    /// Maximum retry attempts exceeded
    #[error("Too many retries: {attempts} attempts failed")]
    TooManyRetries {
        /// Number of failed attempts
        attempts: u32,
    },

    /// URL is invalid or malformed
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Failed to parse response body
    #[error("Response parsing failed: {0}")]
    ResponseParseFailed(String),

    /// Circuit breaker is open due to repeated failures
    #[error("Circuit breaker open: service unavailable")]
    CircuitBreakerOpen,
}

/// XZepr API-specific errors
#[derive(Debug, thiserror::Error)]
pub enum XzeprApiError {
    /// API request failed with HTTP error status
    #[error("API request failed with status {status}: {message}")]
    RequestFailed {
        /// HTTP status code
        status: u16,
        /// Error message from API
        message: String,
    },

    /// Requested resource was not found
    #[error("Resource not found: {resource_type} with ID {id}")]
    NotFound {
        /// Type of resource that was not found
        resource_type: String,
        /// ID of the resource that was not found
        id: String,
    },

    /// Resource conflict (e.g., duplicate)
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Bad request parameters
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Authentication required or failed
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Access denied to resource
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// Internal server error from XZepr API
    #[error("Internal server error: {0}")]
    InternalServerError(String),

    /// XZepr service is temporarily unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Result type alias for XZepr MCP operations
pub type Result<T> = std::result::Result<T, Error>;

/// Backward-compatible alias for older test suites and external code that
/// expect `McpError` in the `xzepr_mcp::error` module. This preserves the
/// previous import path without changing the main `Error` enum name.
pub type McpError = Error;

/// Extension trait for adding context to errors
pub trait ErrorContext<T> {
    /// Add context to an error
    fn context(self, context: impl fmt::Display) -> Result<T>;

    /// Add context to an error using a closure (lazy evaluation)
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<Error>,
{
    fn context(self, context: impl fmt::Display) -> Result<T> {
        self.map_err(|e| {
            let base_error = e.into();
            Error::Internal(format!("{}: {}", context, base_error))
        })
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            Error::Internal(format!("{}: {}", f(), base_error))
        })
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Internal(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Internal(format!("JSON error: {}", err))
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::Config(ConfigError::ParseError(err.to_string()))
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Error::HttpClient(HttpClientError::Timeout { timeout_ms: 30000 })
        } else if err.is_connect() {
            Error::HttpClient(HttpClientError::ConnectionFailed(err.to_string()))
        } else {
            Error::HttpClient(HttpClientError::RequestFailed(err.to_string()))
        }
    }
}

impl Error {
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            Error::Auth(_) => 401,
            Error::Validation(_) => 400,
            Error::RateLimit(_) => 429,
            Error::Config(_) => 500,
            Error::HttpClient(_) => 502,
            Error::McpProtocol(_) => 400,
            Error::XzeprApi(e) => match e {
                XzeprApiError::NotFound { .. } => 404,
                XzeprApiError::Conflict(_) => 409,
                XzeprApiError::BadRequest(_) => 400,
                XzeprApiError::Unauthorized(_) => 401,
                XzeprApiError::Forbidden(_) => 403,
                XzeprApiError::ServiceUnavailable(_) => 503,
                _ => 500,
            },
            Error::Internal(_) => 500,
            Error::External(_) => 502,
        }
    }

    /// Check if error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            Error::HttpClient(
                HttpClientError::Timeout { .. } | HttpClientError::ConnectionFailed(_)
            ) | Error::XzeprApi(XzeprApiError::ServiceUnavailable(_))
                | Error::External(_)
        )
    }

    /// Get error category for metrics
    pub fn category(&self) -> &'static str {
        match self {
            Error::Config(_) => "config",
            Error::Auth(_) => "auth",
            Error::Validation(_) => "validation",
            Error::HttpClient(_) => "http_client",
            Error::RateLimit(_) => "rate_limit",
            Error::McpProtocol(_) => "mcp_protocol",
            Error::XzeprApi(_) => "xzepr_api",
            Error::Internal(_) => "internal",
            Error::External(_) => "external",
        }
    }

    /// Check if error should be logged at ERROR level (vs WARN)
    pub fn is_severe(&self) -> bool {
        matches!(
            self,
            Error::Internal(_)
                | Error::Config(_)
                | Error::XzeprApi(XzeprApiError::InternalServerError(_))
        )
    }
}

/// Error response for API endpoints
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    category: String,
    status: u16,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status_code =
            StatusCode::from_u16(self.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let body = ErrorResponse {
            error: self.to_string(),
            category: self.category().to_string(),
            status: self.status_code(),
        };

        (status_code, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Validation(ValidationError::InvalidUlid("bad_ulid".to_string()));
        assert!(err.to_string().contains("Invalid ULID format"));
    }

    #[test]
    fn test_auth_error_status_code() {
        let err = Error::Auth(AuthError::InvalidToken("test".to_string()));
        assert_eq!(err.status_code(), 401);
    }

    #[test]
    fn test_validation_error_status_code() {
        let err = Error::Validation(ValidationError::PayloadTooLarge {
            size: 100_000,
            max: 65536,
        });
        assert_eq!(err.status_code(), 400);
    }

    #[test]
    fn test_rate_limit_error_status_code() {
        let err = Error::RateLimit("too many requests".to_string());
        assert_eq!(err.status_code(), 429);
    }

    #[test]
    fn test_error_is_retriable() {
        let retriable = Error::HttpClient(HttpClientError::Timeout { timeout_ms: 5000 });
        assert!(retriable.is_retriable());

        let not_retriable = Error::Validation(ValidationError::InvalidUlid("test".to_string()));
        assert!(!not_retriable.is_retriable());
    }

    #[test]
    fn test_error_category() {
        assert_eq!(
            Error::Auth(AuthError::InvalidToken(String::new())).category(),
            "auth"
        );
        assert_eq!(
            Error::Validation(ValidationError::InvalidUlid(String::new())).category(),
            "validation"
        );
        assert_eq!(Error::RateLimit(String::new()).category(), "rate_limit");
    }

    #[test]
    fn test_error_context() {
        let result: std::result::Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        let err = result.context("Failed to load configuration").unwrap_err();
        assert!(err.to_string().contains("Failed to load configuration"));
    }

    #[test]
    fn test_xzepr_api_error_status_codes() {
        let errors = vec![
            (
                Error::XzeprApi(XzeprApiError::NotFound {
                    resource_type: "event".to_string(),
                    id: "123".to_string(),
                }),
                404,
            ),
            (
                Error::XzeprApi(XzeprApiError::Conflict("duplicate".to_string())),
                409,
            ),
            (
                Error::XzeprApi(XzeprApiError::BadRequest("invalid".to_string())),
                400,
            ),
            (
                Error::XzeprApi(XzeprApiError::Unauthorized("no auth".to_string())),
                401,
            ),
            (
                Error::XzeprApi(XzeprApiError::Forbidden("no access".to_string())),
                403,
            ),
            (
                Error::XzeprApi(XzeprApiError::ServiceUnavailable("down".to_string())),
                503,
            ),
        ];

        for (error, expected_status) in errors {
            assert_eq!(error.status_code(), expected_status);
        }
    }

    #[test]
    fn test_error_is_severe() {
        assert!(Error::Internal("panic".to_string()).is_severe());
        assert!(Error::Config(ConfigError::ParseError("bad yaml".to_string())).is_severe());
        assert!(!Error::Validation(ValidationError::InvalidUlid("test".to_string())).is_severe());
    }

    #[test]
    fn test_validation_error_types() {
        let err = ValidationError::PayloadTooLarge {
            size: 100_000,
            max: 65536,
        };
        assert!(err.to_string().contains("100000"));
        assert!(err.to_string().contains("65536"));

        let err = ValidationError::JsonTooDeep { depth: 50, max: 32 };
        assert!(err.to_string().contains("50"));
        assert!(err.to_string().contains("32"));
    }

    #[test]
    fn test_http_client_error_types() {
        let err = HttpClientError::Timeout { timeout_ms: 5000 };
        assert!(err.to_string().contains("5000"));

        let err = HttpClientError::TooManyRetries { attempts: 3 };
        assert!(err.to_string().contains("3"));
    }
}
