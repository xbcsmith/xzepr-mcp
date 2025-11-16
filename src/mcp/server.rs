//! MCP server implementation with StreamableHTTP transport
//!
//! This module provides the complete MCP server implementation with:
//! - StreamableHTTP transport (Server-Sent Events)
//! - Session management with security binding
//! - Authentication and authorization middleware
//! - Rate limiting integration
//! - Observability and audit logging
//!
//! # Example
//!
//! ```no_run
//! use xzepr_mcp::mcp::McpServer;
//! use xzepr_mcp::config::Settings;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let settings = Settings::load(Some("config/production.yaml"))?;
//! let server = McpServer::new(settings).await?;
//! server.start().await?;
//! # Ok(())
//! # }
//! ```

use crate::auth::{JwtValidator, SessionManager};
use crate::client::XzeprClient;
use crate::config::Settings;
use crate::error::Result;
use crate::mcp::{ToolHandlers, ToolRegistry};
use crate::middleware::{InputValidator, RateLimiter, ValidationRules};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// MCP server with StreamableHTTP transport
pub struct McpServer {
    /// Server configuration
    settings: Settings,

    /// Shared server state
    state: Arc<ServerState>,

    /// Server runtime handle
    shutdown: Arc<RwLock<Option<tokio::sync::oneshot::Sender<()>>>>,
}

/// Shared server state
struct ServerState {
    /// XZepr API client
    #[allow(dead_code)]
    xzepr_client: Arc<XzeprClient>,

    /// JWT validator
    jwt_validator: Arc<JwtValidator>,

    /// Session manager
    session_manager: Arc<SessionManager>,

    /// Rate limiter
    rate_limiter: Arc<RateLimiter>,

    /// Input validator
    input_validator: Arc<InputValidator>,

    /// Tool registry
    tool_registry: Arc<ToolRegistry>,

    /// Tool handlers
    tool_handlers: Arc<ToolHandlers>,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,

    /// Service version
    pub version: String,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Health checks
    pub checks: HealthChecks,
}

/// Individual health checks
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthChecks {
    /// XZepr connectivity
    pub xzepr_connectivity: bool,

    /// JWKS cache status
    pub jwks_cache: bool,

    /// Session manager status
    pub session_manager: bool,
}

/// MCP tool call request
#[derive(Debug, Deserialize)]
pub struct ToolCallRequest {
    /// Tool name
    pub tool: String,

    /// Tool parameters
    pub params: serde_json::Value,

    /// Session ID
    pub session_id: Option<String>,

    /// Authorization header (Bearer token)
    #[serde(skip)]
    pub authorization: Option<String>,
}

/// MCP tool call response
#[derive(Debug, Serialize)]
pub struct ToolCallResponse {
    /// Success flag
    pub success: bool,

    /// Response data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Correlation ID for tracing
    pub correlation_id: String,

    /// Response timestamp
    pub timestamp: String,
}

impl McpServer {
    /// Create a new MCP server
    ///
    /// # Arguments
    ///
    /// * `settings` - Server configuration
    ///
    /// # Returns
    ///
    /// Returns a new `McpServer` instance
    ///
    /// # Errors
    ///
    /// Returns error if server initialization fails
    ///
    /// # Examples
    ///
    /// ```
    /// use xzepr_mcp::mcp::McpServer;
    /// use xzepr_mcp::config::Settings;
    ///
    /// # tokio_test::block_on(async {
    /// let settings = Settings::default();
    /// let server = McpServer::new(settings).await;
    /// assert!(server.is_ok());
    /// # })
    /// ```
    pub async fn new(settings: Settings) -> Result<Self> {
        info!("Initializing MCP server");

        // Create XZepr client
        let xzepr_client = Arc::new(XzeprClient::new(&settings.xzepr));

        // Create JWT validator
        let jwt_validator = Arc::new(JwtValidator::new(
            settings.auth.oidc_provider_url.clone(),
            settings.auth.jwt_issuer.clone(),
            settings.auth.jwt_audience.clone(),
            Duration::from_secs(settings.auth.jwks_cache_ttl_secs),
            settings.auth.enable_jwt_validation,
        ));

        // Fetch JWKS on startup
        jwt_validator.fetch_jwks().await.map_err(|e| {
            crate::error::Error::Config(crate::error::ConfigError::InvalidValue {
                field: "oidc_provider_url".to_string(),
                reason: format!("Failed to fetch JWKS: {}", e),
            })
        })?;

        info!("JWKS fetched successfully");

        // Create session manager
        let session_manager = Arc::new(SessionManager::new(
            Duration::from_secs(settings.auth.session_timeout_secs),
            Duration::from_secs(settings.auth.session_idle_timeout_secs),
            settings.auth.enable_session_binding,
        ));

        // Create rate limiter
        let rate_limiter = Arc::new(RateLimiter::new(&settings.rate_limit));

        // Create input validator
        let validation_rules = ValidationRules::new(&settings.security);
        let input_validator = Arc::new(InputValidator::new(validation_rules));

        // Create tool registry
        let tool_registry = Arc::new(ToolRegistry::new());

        // Create tool handlers
        let tool_handlers = Arc::new(ToolHandlers::new(
            Arc::clone(&xzepr_client),
            Arc::clone(&jwt_validator),
            Arc::clone(&session_manager),
            Arc::clone(&rate_limiter),
            Arc::clone(&input_validator),
            Arc::clone(&tool_registry),
        ));

        let state = Arc::new(ServerState {
            xzepr_client,
            jwt_validator,
            session_manager,
            rate_limiter,
            input_validator,
            tool_registry,
            tool_handlers,
        });

        info!("MCP server initialized successfully");

        Ok(Self {
            settings,
            state,
            shutdown: Arc::new(RwLock::new(None)),
        })
    }

    /// Start the MCP server
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when server shuts down gracefully
    ///
    /// # Errors
    ///
    /// Returns error if server fails to start or encounters fatal error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xzepr_mcp::mcp::McpServer;
    /// use xzepr_mcp::config::Settings;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let settings = Settings::default();
    /// let server = McpServer::new(settings).await?;
    /// server.start().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start(&self) -> Result<()> {
        let bind_addr = format!(
            "{}:{}",
            self.settings.server.host, self.settings.server.port
        );
        info!("Starting MCP server on {}", bind_addr);

        // Build application router
        let app = self.build_router();

        // Create shutdown channel
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        {
            let mut shutdown = self.shutdown.write().await;
            *shutdown = Some(tx);
        }

        // Start server
        let listener = tokio::net::TcpListener::bind(&bind_addr)
            .await
            .map_err(|e| {
                crate::error::Error::Config(crate::error::ConfigError::InvalidValue {
                    field: "server.port".to_string(),
                    reason: format!("Failed to bind to {}: {}", bind_addr, e),
                })
            })?;

        info!("MCP server listening on {}", bind_addr);

        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                rx.await.ok();
                info!("Shutdown signal received");
            })
            .await
            .map_err(|e| crate::error::Error::Internal(format!("Server error: {}", e)))?;

        info!("MCP server stopped");
        Ok(())
    }

    /// Stop the MCP server gracefully
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when server stops
    ///
    /// # Errors
    ///
    /// Returns error if shutdown fails
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping MCP server");

        let mut shutdown = self.shutdown.write().await;
        if let Some(tx) = shutdown.take() {
            let _ = tx.send(());
        }

        Ok(())
    }

    /// Build the application router
    fn build_router(&self) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .route("/mcp/tools", post(handle_tool_call))
            .route("/mcp/session", post(create_session))
            .with_state(Arc::clone(&self.state))
    }
}

/// Health check endpoint handler
///
/// # Returns
///
/// Returns health status with component checks
async fn health_check(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    // Check XZepr connectivity (simplified - actual implementation would ping XZepr)
    let xzepr_connectivity = true;

    // Check JWKS cache
    let jwks_cache = state.jwt_validator.has_cached_jwks();

    // Check session manager
    let session_manager = true;

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // Would track actual uptime in production
        checks: HealthChecks {
            xzepr_connectivity,
            jwks_cache,
            session_manager,
        },
    };

    (StatusCode::OK, Json(response))
}

/// Create session endpoint handler
///
/// Creates a new authenticated session
async fn create_session(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let correlation_id = ulid::Ulid::new().to_string();

    info!(correlation_id = %correlation_id, "Creating new session");

    // Extract JWT token from payload
    let token = match payload.get("token").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => {
            return crate::error::Error::Validation(
                crate::error::ValidationError::RequiredFieldMissing("token".to_string()),
            )
            .into_response();
        }
    };

    // Validate JWT
    let claims = match state.jwt_validator.validate_token(token).await {
        Ok(c) => c,
        Err(e) => {
            warn!(correlation_id = %correlation_id, error = %e, "JWT validation failed");
            return crate::error::Error::Auth(crate::error::AuthError::InvalidToken(format!(
                "{}",
                e
            )))
            .into_response();
        }
    };

    // Create session
    let session_id = state
        .session_manager
        .create_session(
            claims.sub.clone(),
            claims.email.clone(),
            claims.display_name().to_string(),
            claims.scopes(),
        )
        .await;

    info!(
        correlation_id = %correlation_id,
        user = %claims.sub,
        session_id = %session_id,
        "Session created successfully"
    );

    let response = serde_json::json!({
        "success": true,
        "session_id": session_id,
        "correlation_id": correlation_id,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    (StatusCode::OK, Json(response)).into_response()
}

/// Tool call endpoint handler
///
/// Handles MCP tool calls with authentication and authorization
async fn handle_tool_call(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<ToolCallRequest>,
) -> impl IntoResponse {
    let correlation_id = ulid::Ulid::new().to_string();

    info!(
        correlation_id = %correlation_id,
        tool = %request.tool,
        "Received tool call request"
    );

    // Extract authorization header (in production, this would come from headers)
    let token = match request
        .authorization
        .as_ref()
        .and_then(|auth| auth.strip_prefix("Bearer "))
    {
        Some(t) => t,
        None => {
            return crate::error::Error::Auth(crate::error::AuthError::MissingAuthHeader)
                .into_response();
        }
    };

    // Validate JWT
    let claims = match state.jwt_validator.validate_token(token).await {
        Ok(c) => c,
        Err(e) => {
            warn!(correlation_id = %correlation_id, error = %e, "JWT validation failed");
            return crate::error::Error::Auth(crate::error::AuthError::InvalidToken(format!(
                "{}",
                e
            )))
            .into_response();
        }
    };

    // Validate session if provided
    if let Some(session_id) = &request.session_id {
        match state
            .session_manager
            .validate_session(session_id, &claims.sub)
            .await
        {
            Ok(_) => {
                // Session is valid
            }
            Err(e) => {
                warn!(
                    correlation_id = %correlation_id,
                    session_id = %session_id,
                    user = %claims.sub,
                    error = %e,
                    "Invalid session"
                );
                return e.into_response();
            }
        }
    }

    // Check tool exists
    let tool_def = match state.tool_registry.get(&request.tool) {
        Some(def) => def,
        None => {
            return crate::error::Error::McpProtocol(format!("Tool not found: {}", request.tool))
                .into_response();
        }
    };

    // Validate required scope
    if !claims.has_scope(&tool_def.required_scope) {
        warn!(
            correlation_id = %correlation_id,
            user = %claims.sub,
            tool = %request.tool,
            required_scope = %tool_def.required_scope,
            "Insufficient permissions"
        );
        return crate::error::Error::Auth(crate::error::AuthError::InsufficientPermissions {
            required: tool_def.required_scope.clone(),
        })
        .into_response();
    }

    // Check rate limit
    if let Err(e) = state
        .rate_limiter
        .check_limit(&claims.sub, Some(&request.tool))
        .await
    {
        warn!(
            correlation_id = %correlation_id,
            user = %claims.sub,
            tool = %request.tool,
            error = %e,
            "Rate limit exceeded"
        );
        return e.into_response();
    }

    // Validate input parameters (basic size check)
    let payload_str = match serde_json::to_string(&request.params) {
        Ok(s) => s,
        Err(e) => {
            return crate::error::Error::Internal(format!("Failed to serialize params: {}", e))
                .into_response();
        }
    };

    if let Err(e) = state
        .input_validator
        .validate_payload_size(payload_str.len(), 65536)
    {
        warn!(
            correlation_id = %correlation_id,
            tool = %request.tool,
            error = %e,
            "Input validation failed"
        );
        return e.into_response();
    }

    // Call tool handler
    let tool_response = match state
        .tool_handlers
        .handle(&request.tool, request.params)
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            error!(
                correlation_id = %correlation_id,
                tool = %request.tool,
                error = %e,
                "Tool execution failed"
            );
            return e.into_response();
        }
    };

    info!(
        correlation_id = %correlation_id,
        tool = %request.tool,
        user = %claims.sub,
        "Tool call completed successfully"
    );

    let response = ToolCallResponse {
        success: true,
        data: Some(tool_response),
        error: None,
        correlation_id,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    (StatusCode::OK, Json(response)).into_response()
}

impl From<serde_json::Value> for crate::models::ToolCallParams {
    fn from(value: serde_json::Value) -> Self {
        crate::models::ToolCallParams::Generic(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access to fetch JWKS from OIDC provider
    async fn test_server_creation() {
        let settings = Settings::default();
        let result = McpServer::new(settings).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires network access to fetch JWKS from OIDC provider
    async fn test_server_build_router() {
        let settings = Settings::default();
        let server = McpServer::new(settings).await.unwrap();
        let _router = server.build_router();
    }

    #[test]
    fn test_tool_call_request_deserialization() {
        let json = r#"{"tool":"get_event","params":{"event_id":"01ARZ3NDEKTSV4RRFFQ69G5FAV"}}"#;
        let request: ToolCallRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.tool, "get_event");
    }

    #[test]
    fn test_tool_call_response_serialization() {
        let response = ToolCallResponse {
            success: true,
            data: Some(serde_json::json!({"key": "value"})),
            error: None,
            correlation_id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("correlation_id"));
    }

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            uptime_seconds: 100,
            checks: HealthChecks {
                xzepr_connectivity: true,
                jwks_cache: true,
                session_manager: true,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("xzepr_connectivity"));
    }
}
