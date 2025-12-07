//! Configuration settings for XZepr MCP
//!
//! This module handles loading configuration from multiple sources:
//! - Configuration files (YAML)
//! - Environment variables
//! - CLI arguments
//!
//! # Configuration Precedence
//!
//! 1. CLI arguments (highest priority)
//! 2. Environment variables
//! 3. Configuration file
//! 4. Default values (lowest priority)

use crate::error::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

/// Main application settings
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Server configuration
    pub server: ServerConfig,

    /// XZepr API configuration
    pub xzepr: XzeprConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,

    /// Observability configuration
    pub observability: ObservabilityConfig,

    /// Security settings
    pub security: SecurityConfig,
}

/// Server configuration
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// Server port to bind to
    #[serde(default = "default_port")]
    pub port: u16,

    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,

    /// Graceful shutdown timeout in seconds
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout_secs: u64,

    /// Maximum payload size in bytes (64KB default)
    #[serde(default = "default_max_payload_size")]
    pub max_payload_size: usize,

    /// Enable CORS
    #[serde(default = "default_true")]
    pub enable_cors: bool,

    /// Allowed CORS origins (empty = allow all)
    #[serde(default)]
    pub cors_origins: Vec<String>,
}

/// XZepr API configuration
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XzeprConfig {
    /// Base URL for XZepr API
    pub base_url: String,

    /// API timeout in seconds
    #[serde(default = "default_api_timeout")]
    pub timeout_secs: u64,

    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Retry backoff base in milliseconds
    #[serde(default = "default_retry_backoff_ms")]
    pub retry_backoff_ms: u64,

    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,

    /// Enable circuit breaker
    #[serde(default = "default_true")]
    pub enable_circuit_breaker: bool,

    /// Circuit breaker failure threshold
    #[serde(default = "default_circuit_breaker_threshold")]
    pub circuit_breaker_threshold: u32,

    /// Circuit breaker timeout in seconds
    #[serde(default = "default_circuit_breaker_timeout")]
    pub circuit_breaker_timeout_secs: u64,
}

/// Authentication configuration
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// OIDC provider URL (e.g., https://keycloak.example.com/realms/xzepr)
    pub oidc_provider_url: String,

    /// Expected JWT issuer
    pub jwt_issuer: String,

    /// Required audience value (token must contain "xzepr-mcp")
    #[serde(default = "default_audience")]
    pub jwt_audience: String,

    /// JWKS cache TTL in seconds
    #[serde(default = "default_jwks_cache_ttl")]
    pub jwks_cache_ttl_secs: u64,

    /// JWKS refresh interval in seconds
    #[serde(default = "default_jwks_refresh_interval")]
    pub jwks_refresh_interval_secs: u64,

    /// Enable JWT validation (should always be true in production)
    #[serde(default = "default_true")]
    pub enable_jwt_validation: bool,

    /// Session timeout in seconds
    #[serde(default = "default_session_timeout")]
    pub session_timeout_secs: u64,

    /// Session idle timeout in seconds
    #[serde(default = "default_session_idle_timeout")]
    pub session_idle_timeout_secs: u64,

    /// Enable session binding to JWT sub claim
    #[serde(default = "default_true")]
    pub enable_session_binding: bool,
}

/// Rate limiting configuration
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Global rate limit (requests per minute)
    #[serde(default = "default_global_rate_limit")]
    pub global_requests_per_minute: u32,

    /// Per-user rate limit (requests per minute)
    #[serde(default = "default_per_user_rate_limit")]
    pub per_user_requests_per_minute: u32,

    /// Per-tool rate limits
    #[serde(default)]
    pub per_tool_limits: PerToolRateLimits,
}

/// Per-tool rate limit configuration
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerToolRateLimits {
    /// Rate limit for read operations (per minute)
    #[serde(default = "default_read_rate_limit")]
    pub read_operations: u32,

    /// Rate limit for write operations (per minute)
    #[serde(default = "default_write_rate_limit")]
    pub write_operations: u32,

    /// Rate limit for search operations (per minute)
    #[serde(default = "default_search_rate_limit")]
    pub search_operations: u32,
}

/// Observability configuration
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Enable tracing
    #[serde(default = "default_true")]
    pub enable_tracing: bool,

    /// Tracing level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// OTLP exporter endpoint
    pub otlp_endpoint: Option<String>,

    /// Enable Prometheus metrics
    #[serde(default = "default_true")]
    pub enable_metrics: bool,

    /// Prometheus metrics port
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,

    /// Enable audit logging
    #[serde(default = "default_true")]
    pub enable_audit_logging: bool,

    /// Audit log file path
    pub audit_log_path: Option<PathBuf>,

    /// Service name for tracing
    #[serde(default = "default_service_name")]
    pub service_name: String,

    /// Service version for tracing
    #[serde(default = "default_service_version")]
    pub service_version: String,
}

/// Enum representing the available security checks that can be enabled
/// or disabled through configuration (for example via `DetectionConfig`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum SecurityCheck {
    /// SQL injection detection
    SqlInjection,

    /// XSS detection
    Xss,

    /// Path traversal detection
    PathTraversal,

    /// SSRF detection
    Ssrf,
}

impl std::fmt::Display for SecurityCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SecurityCheck::{PathTraversal, SqlInjection, Ssrf, Xss};
        let s = match self {
            SqlInjection => "sql_injection",
            Xss => "xss",
            PathTraversal => "path_traversal",
            Ssrf => "ssrf",
        };
        write!(f, "{}", s)
    }
}

impl SecurityCheck {
    /// All supported security checks
    pub fn all_checks() -> HashSet<SecurityCheck> {
        use SecurityCheck::{PathTraversal, SqlInjection, Ssrf, Xss};
        let mut s = HashSet::new();
        s.insert(SqlInjection);
        s.insert(Xss);
        s.insert(PathTraversal);
        s.insert(Ssrf);
        s
    }
}

/// Detection configuration represented as a set of enabled checks.
/// This is preferable to many individual bool fields because it scales better
/// and avoids struct-excessive-bools lint warnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    /// Set of enabled security checks
    #[serde(default = "default_enabled_checks")]
    pub enabled_checks: HashSet<SecurityCheck>,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            enabled_checks: default_enabled_checks(),
        }
    }
}

impl DetectionConfig {
    /// Return whether the provided `SecurityCheck` is enabled by the current configuration.
    ///
    /// This is a convenience wrapper that abstracts the underlying representation
    /// (a set of enabled checks), and provides a clear entrypoint for other modules.
    ///
    /// # Arguments
    ///
    /// * `check` - The `SecurityCheck` to query
    ///
    /// # Returns
    ///
    /// `true` if the check is enabled; otherwise `false`.
    pub fn is_enabled(&self, check: SecurityCheck) -> bool {
        self.enabled_checks.contains(&check)
    }
}

/// Default detection set (enable the common checks by default)
pub fn default_enabled_checks() -> HashSet<SecurityCheck> {
    SecurityCheck::all_checks()
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Maximum JSON nesting depth
    #[serde(default = "default_max_json_depth")]
    pub max_json_depth: usize,

    /// Maximum JSON object key count allowed during validation
    #[serde(default = "default_max_json_keys")]
    pub max_json_keys: usize,

    /// Maximum number of elements allowed inside a JSON array
    #[serde(default = "default_max_array_len")]
    pub max_array_len: usize,

    /// Maximum length in bytes of string values within JSON payloads
    #[serde(default = "default_max_string_len")]
    pub max_string_len: usize,

    /// Detection toggles grouped as a nested configuration.
    #[serde(default)]
    pub detection: DetectionConfig,

    /// Allowed query parameter characters regex
    #[serde(default = "default_query_param_regex")]
    pub query_param_allow_regex: String,
}

// Default value functions
fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_request_timeout() -> u64 {
    30
}

fn default_shutdown_timeout() -> u64 {
    10
}

fn default_max_payload_size() -> usize {
    65536 // 64KB
}

fn default_api_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_backoff_ms() -> u64 {
    100
}

fn default_pool_size() -> usize {
    10
}

fn default_circuit_breaker_threshold() -> u32 {
    5
}

fn default_circuit_breaker_timeout() -> u64 {
    60
}

fn default_audience() -> String {
    "xzepr-mcp".to_string()
}

fn default_jwks_cache_ttl() -> u64 {
    3600 // 1 hour
}

fn default_jwks_refresh_interval() -> u64 {
    300 // 5 minutes
}

fn default_session_timeout() -> u64 {
    3600 // 1 hour
}

fn default_session_idle_timeout() -> u64 {
    1800 // 30 minutes
}

fn default_global_rate_limit() -> u32 {
    1000
}

fn default_per_user_rate_limit() -> u32 {
    100
}

fn default_read_rate_limit() -> u32 {
    100
}

fn default_write_rate_limit() -> u32 {
    20
}

fn default_search_rate_limit() -> u32 {
    30
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_metrics_port() -> u16 {
    9090
}

fn default_service_name() -> String {
    "xzepr-mcp".to_string()
}

fn default_service_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_max_json_depth() -> usize {
    32
}

fn default_max_json_keys() -> usize {
    1024
}

fn default_max_array_len() -> usize {
    10000
}

fn default_max_string_len() -> usize {
    1024 * 1024
}

fn default_query_param_regex() -> String {
    r"^[a-zA-Z0-9_\-\.]+$".to_string()
}

fn default_true() -> bool {
    true
}

impl Settings {
    /// Load settings from file and environment
    ///
    /// # Arguments
    ///
    /// * `config_path` - Optional path to configuration file
    ///
    /// # Returns
    ///
    /// Returns the loaded settings
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if configuration loading fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xzepr_mcp::config::Settings;
    ///
    /// let settings = Settings::load(Some("config/development.yaml")).unwrap();
    /// assert_eq!(settings.server.host, "127.0.0.1");
    /// ```
    pub fn load(config_path: Option<&str>) -> Result<Self> {
        let mut builder = config::Config::builder();

        // Load from file if provided
        if let Some(path) = config_path {
            builder = builder.add_source(config::File::with_name(path));
        }

        // Override with environment variables (prefix: XZEPR_MCP_)
        builder = builder.add_source(
            config::Environment::with_prefix("XZEPR_MCP")
                .separator("__")
                .try_parsing(true),
        );

        let config = builder
            .build()
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        let settings: Settings = config
            .try_deserialize()
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        settings.validate()?;
        Ok(settings)
    }

    /// Validate configuration settings
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::InvalidValue` if validation fails
    pub fn validate(&self) -> Result<()> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "server.port".to_string(),
                reason: "port must be greater than 0".to_string(),
            }
            .into());
        }

        if self.server.max_payload_size == 0 || self.server.max_payload_size > 10_485_760 {
            // 10MB max
            return Err(ConfigError::InvalidValue {
                field: "server.max_payload_size".to_string(),
                reason: "must be between 1 and 10485760 bytes".to_string(),
            }
            .into());
        }

        // Validate XZepr configuration
        if self.xzepr.base_url.is_empty() {
            return Err(ConfigError::MissingRequired("xzepr.base_url".to_string()).into());
        }

        if !self.xzepr.base_url.starts_with("http://")
            && !self.xzepr.base_url.starts_with("https://")
        {
            return Err(ConfigError::InvalidValue {
                field: "xzepr.base_url".to_string(),
                reason: "must start with http:// or https://".to_string(),
            }
            .into());
        }

        // Validate auth configuration
        if self.auth.oidc_provider_url.is_empty() {
            return Err(ConfigError::MissingRequired("auth.oidc_provider_url".to_string()).into());
        }

        if self.auth.jwt_issuer.is_empty() {
            return Err(ConfigError::MissingRequired("auth.jwt_issuer".to_string()).into());
        }

        if self.auth.jwt_audience.is_empty() {
            return Err(ConfigError::MissingRequired("auth.jwt_audience".to_string()).into());
        }

        // Validate observability configuration
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.observability.log_level.as_str()) {
            return Err(ConfigError::InvalidValue {
                field: "observability.log_level".to_string(),
                reason: format!("must be one of: {}", valid_log_levels.join(", ")),
            }
            .into());
        }

        Ok(())
    }

    /// Get request timeout as Duration
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.server.request_timeout_secs)
    }

    /// Get shutdown timeout as Duration
    pub fn shutdown_timeout(&self) -> Duration {
        Duration::from_secs(self.server.shutdown_timeout_secs)
    }

    /// Get API timeout as Duration
    pub fn api_timeout(&self) -> Duration {
        Duration::from_secs(self.xzepr.timeout_secs)
    }

    /// Get session timeout as Duration
    pub fn session_timeout(&self) -> Duration {
        Duration::from_secs(self.auth.session_timeout_secs)
    }

    /// Get session idle timeout as Duration
    pub fn session_idle_timeout(&self) -> Duration {
        Duration::from_secs(self.auth.session_idle_timeout_secs)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: default_host(),
                port: default_port(),
                request_timeout_secs: default_request_timeout(),
                shutdown_timeout_secs: default_shutdown_timeout(),
                max_payload_size: default_max_payload_size(),
                enable_cors: true,
                cors_origins: vec![],
            },
            xzepr: XzeprConfig {
                base_url: "http://localhost:8000".to_string(),
                timeout_secs: default_api_timeout(),
                max_retries: default_max_retries(),
                retry_backoff_ms: default_retry_backoff_ms(),
                pool_size: default_pool_size(),
                enable_circuit_breaker: true,
                circuit_breaker_threshold: default_circuit_breaker_threshold(),
                circuit_breaker_timeout_secs: default_circuit_breaker_timeout(),
            },
            auth: AuthConfig {
                oidc_provider_url: "http://localhost:8080/realms/xzepr".to_string(),
                jwt_issuer: "http://localhost:8080/realms/xzepr".to_string(),
                jwt_audience: default_audience(),
                jwks_cache_ttl_secs: default_jwks_cache_ttl(),
                jwks_refresh_interval_secs: default_jwks_refresh_interval(),
                enable_jwt_validation: true,
                session_timeout_secs: default_session_timeout(),
                session_idle_timeout_secs: default_session_idle_timeout(),
                enable_session_binding: true,
            },
            rate_limit: RateLimitConfig {
                enabled: true,
                global_requests_per_minute: default_global_rate_limit(),
                per_user_requests_per_minute: default_per_user_rate_limit(),
                per_tool_limits: PerToolRateLimits::default(),
            },
            observability: ObservabilityConfig {
                enable_tracing: true,
                log_level: default_log_level(),
                otlp_endpoint: None,
                enable_metrics: true,
                metrics_port: default_metrics_port(),
                enable_audit_logging: true,
                audit_log_path: None,
                service_name: default_service_name(),
                service_version: default_service_version(),
            },
            security: SecurityConfig {
                max_json_depth: default_max_json_depth(),
                max_json_keys: default_max_json_keys(),
                max_array_len: default_max_array_len(),
                max_string_len: default_max_string_len(),
                detection: DetectionConfig::default(),
                query_param_allow_regex: default_query_param_regex(),
            },
        }
    }
}

impl Default for PerToolRateLimits {
    fn default() -> Self {
        Self {
            read_operations: default_read_rate_limit(),
            write_operations: default_write_rate_limit(),
            search_operations: default_search_rate_limit(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.server.host, "127.0.0.1");
        assert_eq!(settings.server.port, 8080);
        assert_eq!(settings.server.max_payload_size, 65536);
    }

    #[test]
    fn test_settings_validation_success() {
        let settings = Settings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_settings_validation_invalid_port() {
        let mut settings = Settings::default();
        settings.server.port = 0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_settings_validation_invalid_payload_size() {
        let mut settings = Settings::default();
        settings.server.max_payload_size = 0;
        assert!(settings.validate().is_err());

        settings.server.max_payload_size = 20_000_000;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_settings_validation_missing_base_url() {
        let mut settings = Settings::default();
        settings.xzepr.base_url = String::new();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_settings_validation_invalid_base_url() {
        let mut settings = Settings::default();
        settings.xzepr.base_url = "ftp://example.com".to_string();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_settings_validation_invalid_log_level() {
        let mut settings = Settings::default();
        settings.observability.log_level = "invalid".to_string();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_duration_conversions() {
        let settings = Settings::default();
        assert_eq!(settings.request_timeout(), Duration::from_secs(30));
        assert_eq!(settings.api_timeout(), Duration::from_secs(30));
        assert_eq!(settings.session_timeout(), Duration::from_secs(3600));
    }

    #[test]
    fn test_per_tool_rate_limits_default() {
        let limits = PerToolRateLimits::default();
        assert_eq!(limits.read_operations, 100);
        assert_eq!(limits.write_operations, 20);
        assert_eq!(limits.search_operations, 30);
    }
}
