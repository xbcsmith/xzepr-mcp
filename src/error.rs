//! Error types and handling for XZepr MCP
//!
//! This module defines all error types used throughout the application,
//! with proper error propagation and context preservation.

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
    #[error("Failed to read config file: {0}")]
    ReadFailed(String),

    #[error("Invalid YAML syntax: {0}")]
    ParseError(String),

    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    #[error("Invalid configuration value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("Environment variable error: {0}")]
    EnvError(String),
}

/// Authentication and authorization errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid JWT token: {0}")]
    InvalidToken(String),

    #[error("Token expired at {0}")]
    TokenExpired(String),

    #[error("Token not yet valid (nbf claim)")]
    TokenNotYetValid,

    #[error("Invalid token signature")]
    InvalidSignature,

    #[error("Missing required claim: {0}")]
    MissingClaim(String),

    #[error("Invalid issuer: expected {expected}, got {actual}")]
    InvalidIssuer { expected: String, actual: String },

    #[error("Invalid audience: token does not contain required audience 'xzepr-mcp'")]
    InvalidAudience,

    #[error("Missing or invalid authorization header")]
    MissingAuthHeader,

    #[error("Insufficient permissions: required scope '{required}' not found")]
    InsufficientPermissions { required: String },

    #[error("JWKS fetch failed: {0}")]
    JwksFetchFailed(String),

    #[error("Key ID (kid) not found in JWKS: {0}")]
    KeyIdNotFound(String),

    #[error("Failed to decode public key: {0}")]
    KeyDecodeFailed(String),

    #[error("Session not found or expired")]
    SessionNotFound,

    #[error("Session validation failed: {0}")]
    SessionValidationFailed(String),
}

/// Input validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid ULID format: {0}")]
    InvalidUlid(String),

    #[error("Invalid UUID format: {0}")]
    InvalidUuid(String),

    #[error("Invalid semver format: {0}")]
    InvalidSemver(String),

    #[error("Field '{field}' validation failed: {reason}")]
    FieldValidation { field: String, reason: String },

    #[error("Payload too large: {size} bytes exceeds maximum of {max} bytes")]
    PayloadTooLarge { size: usize, max: usize },

    #[error("JSON nesting too deep: {depth} exceeds maximum of {max}")]
    JsonTooDeep { depth: usize, max: usize },

    #[error("Potential injection detected in field '{field}': {reason}")]
    PotentialInjection { field: String, reason: String },

    #[error("Invalid query parameter '{param}': {reason}")]
    InvalidQueryParam { param: String, reason: String },

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("Required field missing: {0}")]
    RequiredFieldMissing(String),

    #[error("Invalid enum value for '{field}': {value}")]
    InvalidEnumValue { field: String, value: String },
}

/// HTTP client errors
#[derive(Debug, thiserror::Error)]
pub enum HttpClientError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Request timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Too many retries: {attempts} attempts failed")]
    TooManyRetries { attempts: u32 },

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Response parsing failed: {0}")]
    ResponseParseFailed(String),

    #[error("Circuit breaker open: service unavailable")]
    CircuitBreakerOpen,
}

/// XZepr API-specific errors
#[derive(Debug, thiserror::Error)]
pub enum XzeprApiError {
    #[error("API request failed with status {status}: {message}")]
    RequestFailed { status: u16, message: String },

    #[error("Resource not found: {resource_type} with ID {id}")]
    NotFound { resource_type: String, id: String },

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Result type alias for XZepr MCP operations
pub type Result<T> = std::result::Result<T, Error>;

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
            Error::HttpClient(HttpClientError::Timeout { .. })
                | Error::HttpClient(HttpClientError::ConnectionFailed(_))
                | Error::XzeprApi(XzeprApiError::ServiceUnavailable(_))
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
            size: 100000,
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
            Error::Auth(AuthError::InvalidToken("".to_string())).category(),
            "auth"
        );
        assert_eq!(
            Error::Validation(ValidationError::InvalidUlid("".to_string())).category(),
            "validation"
        );
        assert_eq!(Error::RateLimit("".to_string()).category(), "rate_limit");
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
            size: 100000,
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
