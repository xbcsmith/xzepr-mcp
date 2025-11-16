//! Request models for MCP tool calls
//!
//! This module provides data structures for MCP tool call requests
//! with validation and serialization support.
//!
//! Implementation is a stub for Phase 1 foundation.

use serde::{Deserialize, Serialize};

/// Tool request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    /// Tool name
    pub name: String,

    /// Tool parameters
    pub params: ToolCallParams,
}

/// Tool call parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolCallParams {
    /// Get event parameters
    GetEvent { event_id: String },

    /// Create event parameters
    CreateEvent { event_data: serde_json::Value },

    /// Search events parameters
    SearchEvents { query: serde_json::Value },

    /// Generic parameters
    Generic(serde_json::Value),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_request_serialization() {
        let request = ToolRequest {
            name: "get_event".to_string(),
            params: ToolCallParams::GetEvent {
                event_id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("get_event"));
    }

    #[test]
    fn test_tool_request_deserialization() {
        let json = r#"{"name":"get_event","params":{"event_id":"01ARZ3NDEKTSV4RRFFQ69G5FAV"}}"#;
        let request: ToolRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "get_event");
    }
}
