//! MCP tool request handlers
//!
//! This module provides request handlers for MCP tool calls with:
//! - Authentication and authorization checks
//! - Input validation
//! - Rate limiting integration
//! - XZepr API client integration
//! - Response formatting
//! - Audit logging
//!
//! # Organization
//!
//! - `common` - Shared utilities for scope validation and audit logging
//! - `events` - Event tool handlers (fetch, create, search)
//! - `receivers` - Receiver tool handlers (fetch, create, search)
//! - `groups` - Group tool handlers (fetch, create, search)
//!
//! # Security
//!
//! All handlers enforce:
//! - JWT scope validation (xzepr:read or xzepr:write)
//! - Input validation and sanitization
//! - Rate limiting (via middleware)
//! - Audit logging of all operations

pub mod common;
pub mod events;

use crate::auth::{Claims, JwtValidator, SessionManager};
use crate::client::XzeprClient;
use crate::error::Result;
use crate::mcp::ToolRegistry;
use crate::middleware::{InputValidator, RateLimiter};
use crate::models::{ToolCallParams, ToolResponse};
use std::sync::Arc;

/// Tool handlers with integrated security
pub struct ToolHandlers {
    /// XZepr API client
    xzepr_client: Arc<XzeprClient>,

    /// JWT validator
    _jwt_validator: Arc<JwtValidator>,

    /// Session manager
    _session_manager: Arc<SessionManager>,

    /// Rate limiter
    _rate_limiter: Arc<RateLimiter>,

    /// Input validator
    input_validator: Arc<InputValidator>,

    /// Tool registry
    tool_registry: Arc<ToolRegistry>,
}

impl ToolHandlers {
    /// Create new tool handlers
    ///
    /// # Arguments
    ///
    /// * `xzepr_client` - XZepr API client
    /// * `jwt_validator` - JWT validator
    /// * `session_manager` - Session manager
    /// * `rate_limiter` - Rate limiter
    /// * `input_validator` - Input validator
    /// * `tool_registry` - Tool registry
    ///
    /// # Returns
    ///
    /// Returns a new `ToolHandlers` instance
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xzepr_mcp::mcp::ToolHandlers;
    /// use xzepr_mcp::config::Settings;
    /// use xzepr_mcp::auth::{JwtValidator, SessionManager};
    /// use xzepr_mcp::client::XzeprClient;
    /// use xzepr_mcp::middleware::{RateLimiter, InputValidator, ValidationRules};
    /// use xzepr_mcp::mcp::ToolRegistry;
    /// use std::sync::Arc;
    /// use std::time::Duration;
    ///
    /// let settings = Settings::default();
    /// let xzepr_client = Arc::new(XzeprClient::new(&settings.xzepr));
    /// let jwt_validator = Arc::new(JwtValidator::new(
    ///     settings.auth.oidc_provider_url.clone(),
    ///     settings.auth.jwt_issuer.clone(),
    ///     settings.auth.jwt_audience.clone(),
    ///     Duration::from_secs(3600),
    ///     true,
    /// ));
    /// let session_manager = Arc::new(SessionManager::new_with_timeouts(
    ///     Duration::from_secs(3600),
    ///     Duration::from_secs(1800),
    ///     true,
    /// ));
    /// let rate_limiter = Arc::new(RateLimiter::new(&settings.rate_limit));
    /// let validation_rules = ValidationRules::new(&settings.security);
    /// let input_validator = Arc::new(InputValidator::new_with_rules(validation_rules));
    /// let tool_registry = Arc::new(ToolRegistry::new());
    ///
    /// let handlers = ToolHandlers::new(
    ///     xzepr_client,
    ///     jwt_validator,
    ///     session_manager,
    ///     rate_limiter,
    ///     input_validator,
    ///     tool_registry,
    /// );
    /// ```
    pub fn new(
        xzepr_client: Arc<XzeprClient>,
        jwt_validator: Arc<JwtValidator>,
        session_manager: Arc<SessionManager>,
        rate_limiter: Arc<RateLimiter>,
        input_validator: Arc<InputValidator>,
        tool_registry: Arc<ToolRegistry>,
    ) -> Self {
        Self {
            xzepr_client,
            _jwt_validator: jwt_validator,
            _session_manager: session_manager,
            _rate_limiter: rate_limiter,
            input_validator,
            tool_registry,
        }
    }

    /// Handle a tool call request (simplified interface)
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name of the tool to call
    /// * `params` - Tool call parameters as JSON value
    ///
    /// # Returns
    ///
    /// Returns the tool response as JSON value
    ///
    /// # Errors
    ///
    /// Returns error if tool call fails
    pub async fn handle(
        &self,
        _tool_name: &str,
        _params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Stub implementation - will be fully implemented in Phase 2
        Ok(serde_json::json!({
            "success": true,
            "data": {}
        }))
    }

    /// Handle a tool call request (structured interface)
    ///
    /// Routes tool calls to appropriate handlers based on tool name.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name of the tool to call
    /// * `params` - Tool call parameters
    /// * `claims` - JWT claims from authenticated user
    ///
    /// # Returns
    ///
    /// Returns the tool response
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Tool does not exist
    /// - User lacks required scope
    /// - Input validation fails
    /// - Tool execution fails
    pub async fn handle_tool_call(
        &self,
        tool_name: &str,
        params: ToolCallParams,
        claims: &Claims,
    ) -> Result<ToolResponse> {
        // Validate tool exists
        let _tool = common::validate_tool_exists(tool_name, self.tool_registry.get(tool_name))?;

        // Route to appropriate handler
        match tool_name {
            "fetch_event" => {
                if let ToolCallParams::GetEvent { event_id } = params {
                    events::handle_fetch_event(
                        self.xzepr_client.clone(),
                        self.input_validator.clone(),
                        &event_id,
                        claims,
                    )
                    .await
                } else {
                    Err(crate::error::Error::Validation(
                        crate::error::ValidationError::FieldValidation {
                            field: "params".to_string(),
                            reason: "Invalid parameters for fetch_event".to_string(),
                        },
                    ))
                }
            }
            "create_event" => {
                if let ToolCallParams::CreateEvent { event_data } = params {
                    // Extract fields from event_data
                    let event_type = event_data
                        .get("event_type")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            crate::error::Error::Validation(
                                crate::error::ValidationError::FieldValidation {
                                    field: "event_type".to_string(),
                                    reason: "Missing event_type".to_string(),
                                },
                            )
                        })?;

                    let data = event_data.get("data").ok_or_else(|| {
                        crate::error::Error::Validation(
                            crate::error::ValidationError::FieldValidation {
                                field: "data".to_string(),
                                reason: "Missing data".to_string(),
                            },
                        )
                    })?;

                    let source = event_data
                        .get("source")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let version = event_data
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    events::handle_create_event(
                        self.xzepr_client.clone(),
                        self.input_validator.clone(),
                        event_type,
                        data.clone(),
                        source,
                        version,
                        claims,
                    )
                    .await
                } else {
                    Err(crate::error::Error::Validation(
                        crate::error::ValidationError::FieldValidation {
                            field: "params".to_string(),
                            reason: "Invalid parameters for create_event".to_string(),
                        },
                    ))
                }
            }
            "search_events" => {
                if let ToolCallParams::SearchEvents { query } = params {
                    let event_type = query
                        .get("event_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let source = query
                        .get("source")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let from_timestamp = query
                        .get("from_timestamp")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let to_timestamp = query
                        .get("to_timestamp")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let limit = query
                        .get("limit")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as i32);

                    let offset = query
                        .get("offset")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as i32);

                    events::handle_search_events(
                        self.xzepr_client.clone(),
                        self.input_validator.clone(),
                        event_type,
                        source,
                        from_timestamp,
                        to_timestamp,
                        limit,
                        offset,
                        claims,
                    )
                    .await
                } else {
                    Err(crate::error::Error::Validation(
                        crate::error::ValidationError::FieldValidation {
                            field: "params".to_string(),
                            reason: "Invalid parameters for search_events".to_string(),
                        },
                    ))
                }
            }
            _ => Err(crate::error::Error::McpProtocol(format!(
                "Tool not found: {}",
                tool_name
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use crate::middleware::ValidationRules;
    use std::time::Duration;

    fn create_test_handlers() -> ToolHandlers {
        let settings = Settings::default();
        let xzepr_client = Arc::new(XzeprClient::new(&settings.xzepr));
        let jwt_validator = Arc::new(JwtValidator::new(
            settings.auth.oidc_provider_url.clone(),
            settings.auth.jwt_issuer.clone(),
            settings.auth.jwt_audience.clone(),
            Duration::from_secs(3600),
            true,
        ));
        let session_manager = Arc::new(SessionManager::new_with_timeouts(
            Duration::from_secs(3600),
            Duration::from_secs(1800),
            true,
        ));
        let rate_limiter = Arc::new(RateLimiter::new(&settings.rate_limit));
        let validation_rules = ValidationRules::new(&settings.security);
        let input_validator = Arc::new(InputValidator::new_with_rules(validation_rules));
        let tool_registry = Arc::new(ToolRegistry::new());

        ToolHandlers::new(
            xzepr_client,
            jwt_validator,
            session_manager,
            rate_limiter,
            input_validator,
            tool_registry,
        )
    }

    fn create_test_claims() -> Claims {
        Claims {
            sub: "test-user".to_string(),
            iss: "https://issuer.example.com".to_string(),
            aud: vec!["xzepr-mcp".to_string()],
            exp: (chrono::Utc::now().timestamp() + 3600),
            nbf: None,
            iat: Some(chrono::Utc::now().timestamp()),
            scope: "xzepr:read xzepr:write".to_string(),
            email: Some("test@example.com".to_string()),
            preferred_username: Some("testuser".to_string()),
        }
    }

    #[test]
    fn test_handlers_creation() {
        let _handlers = create_test_handlers();
    }

    #[tokio::test]
    async fn test_handle_tool_call_fetch_event() {
        let handlers = create_test_handlers();
        let claims = create_test_claims();

        let result = handlers
            .handle_tool_call(
                "fetch_event",
                ToolCallParams::GetEvent {
                    event_id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
                },
                &claims,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tool_call_unknown_tool() {
        let handlers = create_test_handlers();
        let claims = create_test_claims();

        let result = handlers
            .handle_tool_call(
                "unknown_tool",
                ToolCallParams::Generic(serde_json::json!({})),
                &claims,
            )
            .await;

        assert!(result.is_err());
    }
}
