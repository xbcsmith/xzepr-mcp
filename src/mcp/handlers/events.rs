//! Event tool handlers
//!
//! This module provides handlers for event-related MCP tools:
//! - fetch_event: Retrieve event by ULID
//! - create_event: Create new event
//! - search_events: Search events with filters
//!
//! All handlers integrate with:
//! - JWT scope validation (xzepr:read or xzepr:write)
//! - Input validation and sanitization
//! - XZepr API client
//! - Audit logging
//! - Error handling with user-friendly messages

use crate::auth::Claims;
use crate::client::XzeprClient;
use crate::error::{Error, Result};
use crate::middleware::InputValidator;
use crate::models::ToolResponse;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, instrument};

use super::common::{
    audit_log_tool_call, create_params_fingerprint, generate_correlation_id, validate_scope,
};

/// Handle fetch_event tool call
///
/// Retrieves a specific event by its ULID identifier.
///
/// # Arguments
///
/// * `xzepr_client` - XZepr API client
/// * `input_validator` - Input validator
/// * `event_id` - Event ULID to fetch
/// * `claims` - JWT claims from authenticated user
///
/// # Returns
///
/// Returns the event data as a tool response
///
/// # Errors
///
/// Returns error if:
/// - User lacks xzepr:read scope
/// - Event ID is invalid format
/// - Event not found in XZepr
/// - XZepr API is unavailable
///
/// # Security
///
/// - Requires xzepr:read scope
/// - Validates event ID format
/// - Audit logs all access attempts
#[instrument(skip(xzepr_client, input_validator, claims), fields(user = %claims.sub))]
pub async fn handle_fetch_event(
    xzepr_client: Arc<XzeprClient>,
    input_validator: Arc<InputValidator>,
    event_id: &str,
    claims: &Claims,
) -> Result<ToolResponse> {
    let start = Instant::now();
    let correlation_id = generate_correlation_id();

    debug!(
        correlation_id = %correlation_id,
        event_id = %event_id,
        "Handling fetch_event request"
    );

    // Validate scope
    validate_scope(claims, "xzepr:read")?;

    // Validate input
    let validation_result = input_validator.validate_input(&json!({
        "event_id": event_id
    }));

    if let Err(e) = validation_result {
        let duration = start.elapsed().as_millis() as u64;
        audit_log_tool_call(
            &claims.sub,
            "fetch_event",
            &format!("event_id={}", event_id),
            false,
            Some(&e.to_string()),
            duration,
        );
        return Err(e);
    }

    // Fetch event from XZepr
    match xzepr_client.get_event(event_id).await {
        Ok(event) => {
            let duration = start.elapsed().as_millis() as u64;
            audit_log_tool_call(
                &claims.sub,
                "fetch_event",
                &format!("event_id={}", event_id),
                true,
                None,
                duration,
            );

            info!(
                correlation_id = %correlation_id,
                event_id = %event_id,
                duration_ms = duration,
                "Successfully fetched event"
            );

            Ok(ToolResponse {
                success: true,
                data: Some(json!({
                    "event": event,
                    "correlation_id": correlation_id,
                })),
                error: None,
            })
        }
        Err(e) => {
            let duration = start.elapsed().as_millis() as u64;
            let error_msg = e.to_string();
            audit_log_tool_call(
                &claims.sub,
                "fetch_event",
                &format!("event_id={}", event_id),
                false,
                Some(&error_msg),
                duration,
            );
            Err(e)
        }
    }
}

/// Handle create_event tool call
///
/// Creates a new event in the XZepr system.
///
/// # Arguments
///
/// * `xzepr_client` - XZepr API client
/// * `input_validator` - Input validator
/// * `event_type` - Event type identifier
/// * `data` - Event payload data
/// * `source` - Optional event source
/// * `version` - Optional event schema version
/// * `claims` - JWT claims from authenticated user
///
/// # Returns
///
/// Returns the created event data with assigned ULID
///
/// # Errors
///
/// Returns error if:
/// - User lacks xzepr:write scope
/// - Event type is invalid
/// - Event data fails validation
/// - Version is not valid semver
/// - XZepr API is unavailable
///
/// # Security
///
/// - Requires xzepr:write scope
/// - Validates all input fields
/// - Sanitizes event data
/// - Audit logs all creation attempts
#[instrument(skip(xzepr_client, input_validator, data, claims), fields(user = %claims.sub))]
pub async fn handle_create_event(
    xzepr_client: Arc<XzeprClient>,
    input_validator: Arc<InputValidator>,
    event_type: &str,
    data: serde_json::Value,
    source: Option<String>,
    version: Option<String>,
    claims: &Claims,
) -> Result<ToolResponse> {
    let start = Instant::now();
    let correlation_id = generate_correlation_id();

    debug!(
        correlation_id = %correlation_id,
        event_type = %event_type,
        "Handling create_event request"
    );

    // Validate scope
    validate_scope(claims, "xzepr:write")?;

    // Build event payload for validation
    let mut event_payload = json!({
        "event_type": event_type,
        "data": data,
    });

    if let Some(ref src) = source {
        event_payload["source"] = json!(src);
    }

    if let Some(ref ver) = version {
        event_payload["version"] = json!(ver);
    }

    // Validate input
    let validation_result = input_validator.validate_input(&event_payload);

    if let Err(e) = validation_result {
        let duration = start.elapsed().as_millis() as u64;
        audit_log_tool_call(
            &claims.sub,
            "create_event",
            &create_params_fingerprint(&event_payload),
            false,
            Some(&e.to_string()),
            duration,
        );
        return Err(e);
    }

    // Create event in XZepr
    match xzepr_client
        .create_event(event_type, &data, source.as_deref(), version.as_deref())
        .await
    {
        Ok(event) => {
            let duration = start.elapsed().as_millis() as u64;
            audit_log_tool_call(
                &claims.sub,
                "create_event",
                &create_params_fingerprint(&event_payload),
                true,
                None,
                duration,
            );

            info!(
                correlation_id = %correlation_id,
                event_type = %event_type,
                event_id = %event.id,
                duration_ms = duration,
                "Successfully created event"
            );

            Ok(ToolResponse {
                success: true,
                data: Some(json!({
                    "event": event,
                    "correlation_id": correlation_id,
                })),
                error: None,
            })
        }
        Err(e) => {
            let duration = start.elapsed().as_millis() as u64;
            let error_msg = e.to_string();
            audit_log_tool_call(
                &claims.sub,
                "create_event",
                &create_params_fingerprint(&event_payload),
                false,
                Some(&error_msg),
                duration,
            );
            Err(e)
        }
    }
}

/// Handle search_events tool call
///
/// Searches for events matching specified criteria.
///
/// # Arguments
///
/// * `xzepr_client` - XZepr API client
/// * `input_validator` - Input validator
/// * `event_type` - Optional event type filter
/// * `source` - Optional event source filter
/// * `from_timestamp` - Optional start timestamp (ISO 8601)
/// * `to_timestamp` - Optional end timestamp (ISO 8601)
/// * `limit` - Maximum results (1-100, default 10)
/// * `offset` - Pagination offset (default 0)
/// * `claims` - JWT claims from authenticated user
///
/// # Returns
///
/// Returns paginated list of events matching criteria
///
/// # Errors
///
/// Returns error if:
/// - User lacks xzepr:read scope
/// - Timestamp format is invalid
/// - Limit is out of range
/// - XZepr API is unavailable
///
/// # Security
///
/// - Requires xzepr:read scope
/// - Validates all input parameters
/// - Enforces pagination limits
/// - Audit logs all search attempts
#[instrument(skip(xzepr_client, input_validator, claims), fields(user = %claims.sub))]
pub async fn handle_search_events(
    xzepr_client: Arc<XzeprClient>,
    input_validator: Arc<InputValidator>,
    event_type: Option<String>,
    source: Option<String>,
    from_timestamp: Option<String>,
    to_timestamp: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
    claims: &Claims,
) -> Result<ToolResponse> {
    let start = Instant::now();
    let correlation_id = generate_correlation_id();

    debug!(
        correlation_id = %correlation_id,
        "Handling search_events request"
    );

    // Validate scope
    validate_scope(claims, "xzepr:read")?;

    // Build search query for validation
    let mut query = json!({});

    if let Some(ref et) = event_type {
        query["event_type"] = json!(et);
    }
    if let Some(ref src) = source {
        query["source"] = json!(src);
    }
    if let Some(ref from) = from_timestamp {
        query["from_timestamp"] = json!(from);
    }
    if let Some(ref to) = to_timestamp {
        query["to_timestamp"] = json!(to);
    }
    if let Some(lim) = limit {
        query["limit"] = json!(lim);
    }
    if let Some(off) = offset {
        query["offset"] = json!(off);
    }

    // Validate input
    let validation_result = input_validator.validate_input(&query);

    if let Err(e) = validation_result {
        let duration = start.elapsed().as_millis() as u64;
        audit_log_tool_call(
            &claims.sub,
            "search_events",
            &create_params_fingerprint(&query),
            false,
            Some(&e.to_string()),
            duration,
        );
        return Err(e);
    }

    // Validate limit range
    let limit = limit.unwrap_or(10);
    if limit < 1 || limit > 100 {
        return Err(Error::Validation(
            crate::error::ValidationError::InvalidQueryParam {
                param: "limit".to_string(),
                reason: "Limit must be between 1 and 100".to_string(),
            },
        ));
    }

    let offset = offset.unwrap_or(0);
    if offset < 0 {
        return Err(Error::Validation(
            crate::error::ValidationError::InvalidQueryParam {
                param: "offset".to_string(),
                reason: "Offset must be non-negative".to_string(),
            },
        ));
    }

    // Search events in XZepr
    match xzepr_client
        .search_events(
            event_type.as_deref(),
            source.as_deref(),
            from_timestamp.as_deref(),
            to_timestamp.as_deref(),
            limit,
            offset,
        )
        .await
    {
        Ok(events) => {
            let duration = start.elapsed().as_millis() as u64;
            audit_log_tool_call(
                &claims.sub,
                "search_events",
                &create_params_fingerprint(&query),
                true,
                None,
                duration,
            );

            info!(
                correlation_id = %correlation_id,
                result_count = events.len(),
                duration_ms = duration,
                "Successfully searched events"
            );

            Ok(ToolResponse {
                success: true,
                data: Some(json!({
                    "events": events,
                    "count": events.len(),
                    "limit": limit,
                    "offset": offset,
                    "correlation_id": correlation_id,
                })),
                error: None,
            })
        }
        Err(e) => {
            let duration = start.elapsed().as_millis() as u64;
            let error_msg = e.to_string();
            audit_log_tool_call(
                &claims.sub,
                "search_events",
                &create_params_fingerprint(&query),
                false,
                Some(&error_msg),
                duration,
            );
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use crate::middleware::ValidationRules;

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

    fn create_test_xzepr_client() -> Arc<XzeprClient> {
        let settings = Settings::default();
        Arc::new(XzeprClient::new(&settings.xzepr))
    }

    fn create_test_validator() -> Arc<InputValidator> {
        let settings = Settings::default();
        let rules = ValidationRules::new(&settings.security);
        Arc::new(InputValidator::new(rules))
    }

    #[tokio::test]
    async fn test_handle_fetch_event_missing_scope() {
        let client = create_test_xzepr_client();
        let validator = create_test_validator();
        let claims = create_test_claims("xzepr:write");

        let result =
            handle_fetch_event(client, validator, "01ARZ3NDEKTSV4RRFFQ69G5FAV", &claims).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Auth(_)));
    }

    #[tokio::test]
    async fn test_handle_create_event_missing_scope() {
        let client = create_test_xzepr_client();
        let validator = create_test_validator();
        let claims = create_test_claims("xzepr:read");

        let result = handle_create_event(
            client,
            validator,
            "test.event",
            json!({"key": "value"}),
            None,
            None,
            &claims,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Auth(_)));
    }

    #[tokio::test]
    async fn test_handle_search_events_missing_scope() {
        let client = create_test_xzepr_client();
        let validator = create_test_validator();
        let claims = create_test_claims("xzepr:write");

        let result = handle_search_events(
            client,
            validator,
            None,
            None,
            None,
            None,
            Some(10),
            Some(0),
            &claims,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Auth(_)));
    }

    #[tokio::test]
    async fn test_handle_search_events_invalid_limit() {
        let client = create_test_xzepr_client();
        let validator = create_test_validator();
        let claims = create_test_claims("xzepr:read");

        let result = handle_search_events(
            client,
            validator,
            None,
            None,
            None,
            None,
            Some(200),
            Some(0),
            &claims,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }

    #[tokio::test]
    async fn test_handle_search_events_negative_offset() {
        let client = create_test_xzepr_client();
        let validator = create_test_validator();
        let claims = create_test_claims("xzepr:read");

        let result = handle_search_events(
            client,
            validator,
            None,
            None,
            None,
            None,
            Some(10),
            Some(-1),
            &claims,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Validation(_)));
    }
}
