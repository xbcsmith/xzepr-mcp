//! Response models for XZepr API
//!
//! This module provides data structures for XZepr API responses
//! with serialization and deserialization support.
//!
//! Implementation is a stub for Phase 1 foundation.

use serde::{Deserialize, Serialize};

/// Tool response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    /// Success flag
    pub success: bool,

    /// Response data
    pub data: Option<serde_json::Value>,

    /// Error message if failed
    pub error: Option<String>,
}

impl ToolResponse {
    /// Create a success response
    ///
    /// # Arguments
    ///
    /// * `data` - Response data
    ///
    /// # Returns
    ///
    /// Returns a successful `ToolResponse`
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response
    ///
    /// # Arguments
    ///
    /// * `message` - Error message
    ///
    /// # Returns
    ///
    /// Returns an error `ToolResponse`
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Event data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    /// Event ID (ULID)
    pub id: String,

    /// Event type
    #[serde(rename = "type")]
    pub event_type: String,

    /// Event data payload
    pub data: serde_json::Value,

    /// Event timestamp
    pub timestamp: String,

    /// Event source
    pub source: Option<String>,

    /// Event version
    pub version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_response_success() {
        let response = ToolResponse::success(serde_json::json!({"key": "value"}));
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_tool_response_error() {
        let response = ToolResponse::error("Something went wrong");
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Something went wrong");
    }

    #[test]
    fn test_event_data_serialization() {
        let event = EventData {
            id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
            event_type: "test.event".to_string(),
            data: serde_json::json!({"key": "value"}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            source: Some("test-source".to_string()),
            version: Some("1.0.0".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("test.event"));
        assert!(json.contains("01ARZ3NDEKTSV4RRFFQ69G5FAV"));
    }

    #[test]
    fn test_event_data_deserialization() {
        let json = r#"{
            "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            "type": "test.event",
            "data": {"key": "value"},
            "timestamp": "2024-01-01T00:00:00Z",
            "source": "test-source",
            "version": "1.0.0"
        }"#;

        let event: EventData = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, "01ARZ3NDEKTSV4RRFFQ69G5FAV");
        assert_eq!(event.event_type, "test.event");
    }
}
