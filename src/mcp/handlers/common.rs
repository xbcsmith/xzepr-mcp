//! Common handler utilities
//!
//! This module provides shared utilities for MCP tool handlers:
//! - Scope validation against JWT claims
//! - Audit logging for all tool invocations
//! - Response correlation ID generation
//! - Error mapping from XZepr errors to MCP errors
//!
//! # Security
//!
//! All handlers must validate scopes before executing operations.
//! All tool invocations are audit logged with user, tool, and outcome.

use crate::auth::Claims;
use crate::error::{Error, Result};
use crate::mcp::ToolDefinition;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Validate that JWT claims contain required scope
///
/// # Arguments
///
/// * `claims` - JWT claims from authenticated user
/// * `required_scope` - Required scope for operation
///
/// # Returns
///
/// Returns Ok if scope is present, error otherwise
///
/// # Errors
///
/// Returns `Error::Forbidden` if required scope is missing
///
/// # Examples
///
/// ```
/// use xzepr_mcp::auth::Claims;
/// use xzepr_mcp::mcp::handlers::common::validate_scope;
///
/// let claims = Claims {
///     sub: "user123".to_string(),
///     iss: "https://issuer.example.com".to_string(),
///     aud: vec!["xzepr-mcp".to_string()],
///     exp: 1234567890,
///     nbf: None,
///     iat: Some(1234567800),
///     scope: "xzepr:read xzepr:write".to_string(),
///     email: Some("user@example.com".to_string()),
///     preferred_username: Some("user123".to_string()),
/// };
///
/// let result = validate_scope(&claims, "xzepr:read");
/// assert!(result.is_ok());
///
/// let result = validate_scope(&claims, "xzepr:admin");
/// assert!(result.is_err());
/// ```
pub fn validate_scope(claims: &Claims, required_scope: &str) -> Result<()> {
    let scopes: Vec<&str> = claims.scope.split_whitespace().collect();

    if scopes.contains(&required_scope) {
        info!(
            user = %claims.sub,
            required_scope = %required_scope,
            "Scope validation passed"
        );
        Ok(())
    } else {
        warn!(
            user = %claims.sub,
            required_scope = %required_scope,
            available_scopes = %claims.scope,
            "Scope validation failed: missing required scope"
        );
        Err(Error::Auth(
            crate::error::AuthError::InsufficientPermissions {
                required: required_scope.to_string(),
            },
        ))
    }
}

/// Audit log a tool invocation
///
/// # Arguments
///
/// * `user_id` - User identifier from JWT sub claim
/// * `tool_name` - Name of tool being invoked
/// * `params_fingerprint` - Hash or summary of input parameters
/// * `success` - Whether the operation succeeded
/// * `error_msg` - Error message if operation failed
/// * `duration_ms` - Operation duration in milliseconds
///
/// # Examples
///
/// ```
/// use xzepr_mcp::mcp::handlers::common::audit_log_tool_call;
///
/// audit_log_tool_call(
///     "user123",
///     "fetch_event",
///     "event_id=01ARZ3NDEKTSV4RRFFQ69G5FAV",
///     true,
///     None,
///     45,
/// );
/// ```
pub fn audit_log_tool_call(
    user_id: &str,
    tool_name: &str,
    params_fingerprint: &str,
    success: bool,
    error_msg: Option<&str>,
    duration_ms: u64,
) {
    if success {
        info!(
            user_id = %user_id,
            tool_name = %tool_name,
            params_fingerprint = %params_fingerprint,
            duration_ms = duration_ms,
            success = true,
            "Tool invocation completed successfully"
        );
    } else {
        error!(
            user_id = %user_id,
            tool_name = %tool_name,
            params_fingerprint = %params_fingerprint,
            duration_ms = duration_ms,
            success = false,
            error = ?error_msg,
            "Tool invocation failed"
        );
    }
}

/// Generate a correlation ID for request tracking
///
/// # Returns
///
/// Returns a new UUID v4 as a string
///
/// # Examples
///
/// ```
/// use xzepr_mcp::mcp::handlers::common::generate_correlation_id;
///
/// let correlation_id = generate_correlation_id();
/// assert_eq!(correlation_id.len(), 36);
/// ```
pub fn generate_correlation_id() -> String {
    Uuid::new_v4().to_string()
}

/// Create a fingerprint of tool parameters for audit logging
///
/// This creates a summary of parameters that is safe to log without
/// exposing sensitive data.
///
/// # Arguments
///
/// * `params` - Tool parameters as JSON value
///
/// # Returns
///
/// Returns a string fingerprint of the parameters
///
/// # Examples
///
/// ```
/// use xzepr_mcp::mcp::handlers::common::create_params_fingerprint;
/// use serde_json::json;
///
/// let params = json!({"event_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV"});
/// let fingerprint = create_params_fingerprint(&params);
/// assert!(fingerprint.contains("event_id"));
/// ```
pub fn create_params_fingerprint(params: &serde_json::Value) -> String {
    match params {
        serde_json::Value::Object(map) => {
            let keys: Vec<String> = map.keys().map(|k| k.to_string()).collect();
            format!("params[{}]", keys.join(","))
        }
        _ => "params[unknown]".to_string(),
    }
}

/// Validate tool exists and return its definition
///
/// # Arguments
///
/// * `tool_name` - Name of the tool
/// * `tool` - Optional tool definition
///
/// # Returns
///
/// Returns the tool definition
///
/// # Errors
///
/// Returns `Error::NotFound` if tool does not exist
pub fn validate_tool_exists<'a>(
    tool_name: &str,
    tool: Option<&'a ToolDefinition>,
) -> Result<&'a ToolDefinition> {
    tool.ok_or_else(|| Error::McpProtocol(format!("Tool not found: {}", tool_name)))
}

/// Map XZepr client errors to user-friendly messages
///
/// # Arguments
///
/// * `error` - The error to map
///
/// # Returns
///
/// Returns a user-friendly error message
#[allow(dead_code)]
pub fn map_error_to_message(error: &Error) -> String {
    match error {
        Error::Auth(auth_err) => format!("Authentication error: {}", auth_err),
        Error::Validation(val_err) => format!("Invalid input: {}", val_err),
        Error::RateLimit(msg) => format!("Rate limit exceeded: {}", msg),
        Error::XzeprApi(api_err) => format!("XZepr API error: {}", api_err),
        Error::McpProtocol(msg) => format!("Protocol error: {}", msg),
        Error::HttpClient(http_err) => format!("Network error: {}", http_err),
        Error::Config(cfg_err) => format!("Configuration error: {}", cfg_err),
        Error::Internal(msg) => format!("Internal error: {}", msg),
        Error::External(msg) => format!("External service error: {}", msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::RateLimitCategory;

    fn create_test_claims(scopes: &str) -> Claims {
        Claims {
            sub: "test-user".to_string(),
            iss: "https://issuer.example.com".to_string(),
            aud: vec!["xzepr-mcp".to_string()],
            exp: (chrono::Utc::now().timestamp() + 3600),
            nbf: None,
            iat: Some(chrono::Utc::now().timestamp()),
            scope: scopes.to_string(),
            email: Some("test@example.com".to_string()),
            preferred_username: Some("testuser".to_string()),
        }
    }

    #[test]
    fn test_validate_scope_success() {
        let claims = create_test_claims("xzepr:read xzepr:write");
        let result = validate_scope(&claims, "xzepr:read");
        assert!(result.is_ok());

        let result = validate_scope(&claims, "xzepr:write");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_scope_failure() {
        let claims = create_test_claims("xzepr:read");
        let result = validate_scope(&claims, "xzepr:write");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Auth(_)));
    }

    #[test]
    fn test_validate_scope_no_scopes() {
        let claims = create_test_claims("");
        let result = validate_scope(&claims, "xzepr:read");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_correlation_id() {
        let id1 = generate_correlation_id();
        let id2 = generate_correlation_id();

        assert_eq!(id1.len(), 36);
        assert_eq!(id2.len(), 36);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_create_params_fingerprint() {
        let params = serde_json::json!({
            "event_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            "type": "test"
        });

        let fingerprint = create_params_fingerprint(&params);
        assert!(fingerprint.contains("event_id"));
        assert!(fingerprint.contains("type"));
    }

    #[test]
    fn test_create_params_fingerprint_empty() {
        let params = serde_json::json!({});
        let fingerprint = create_params_fingerprint(&params);
        assert_eq!(fingerprint, "params[]");
    }

    #[test]
    fn test_create_params_fingerprint_non_object() {
        let params = serde_json::json!("string");
        let fingerprint = create_params_fingerprint(&params);
        assert_eq!(fingerprint, "params[unknown]");
    }

    #[test]
    fn test_validate_tool_exists_success() {
        let tool = ToolDefinition {
            name: "test_tool".to_string(),
            description: "Test tool".to_string(),
            required_scope: "xzepr:read".to_string(),
            rate_limit_category: RateLimitCategory::Read,
            latency_category: crate::mcp::LatencyCategory::Fast,
            input_schema: serde_json::json!({}),
        };

        let result = validate_tool_exists("test_tool", Some(&tool));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tool_exists_failure() {
        let result = validate_tool_exists("missing_tool", None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::McpProtocol(_)));
    }

    #[test]
    fn test_map_error_to_message() {
        let error = Error::McpProtocol("tool not found".to_string());
        let msg = map_error_to_message(&error);
        assert!(msg.contains("Protocol error"));

        let error = Error::Auth(crate::error::AuthError::InsufficientPermissions {
            required: "xzepr:write".to_string(),
        });
        let msg = map_error_to_message(&error);
        assert!(msg.contains("Authentication error"));

        let error = Error::RateLimit("exceeded".to_string());
        let msg = map_error_to_message(&error);
        assert!(msg.contains("Rate limit"));
    }

    #[test]
    fn test_audit_log_tool_call_success() {
        audit_log_tool_call("user123", "fetch_event", "event_id=123", true, None, 42);
    }

    #[test]
    fn test_audit_log_tool_call_failure() {
        audit_log_tool_call(
            "user123",
            "fetch_event",
            "event_id=123",
            false,
            Some("Not found"),
            42,
        );
    }
}
