//! MCP tool definitions with security metadata
//!
//! This module provides comprehensive tool definitions for the XZepr MCP server.
//! Each tool includes:
//! - JSON schema validation for inputs
//! - Required OAuth scopes (xzepr:read or xzepr:write)
//! - Rate limit categories
//! - Tool metadata for MCP protocol
//!
//! # Tool Categories
//!
//! - **Events**: fetch_event, create_event, search_events
//! - **Receivers**: fetch_receiver, create_receiver, search_receivers
//! - **Groups**: fetch_group, create_group, search_groups
//!
//! # Security Model
//!
//! - Read operations require `xzepr:read` scope
//! - Write operations require `xzepr:write` scope
//! - All tools enforce input validation and rate limiting

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool definition with security metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name (must be unique)
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Required OAuth scope
    pub required_scope: String,

    /// Rate limit category
    pub rate_limit_category: RateLimitCategory,

    /// Input schema (JSON Schema)
    pub input_schema: serde_json::Value,

    /// Estimated latency category
    pub latency_category: LatencyCategory,
}

/// Rate limit category for tools
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RateLimitCategory {
    /// Read operations (higher limits: 100/min)
    Read,

    /// Write operations (lower limits: 20/min)
    Write,

    /// Search operations (medium limits: 50/min)
    Search,
}

/// Estimated latency category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LatencyCategory {
    /// Fast operations (< 100ms)
    Fast,

    /// Medium operations (100ms - 1s)
    Medium,

    /// Slow operations (> 1s)
    Slow,
}

/// Tool registry for MCP server
pub struct ToolRegistry {
    /// Registered tools by name
    tools: HashMap<String, ToolDefinition>,
}

impl ToolRegistry {
    /// Create a new tool registry with default XZepr tools
    ///
    /// Registers all 9 XZepr tools:
    /// - 3 event tools (fetch, create, search)
    /// - 3 receiver tools (fetch, create, search)
    /// - 3 group tools (fetch, create, search)
    ///
    /// # Returns
    ///
    /// Returns a new `ToolRegistry` with all default tools registered
    ///
    /// # Examples
    ///
    /// ```
    /// use xzepr_mcp::mcp::ToolRegistry;
    ///
    /// let registry = ToolRegistry::new();
    /// assert_eq!(registry.list().len(), 9);
    /// ```
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Register all default tools
        registry.register_event_tools();
        registry.register_receiver_tools();
        registry.register_group_tools();

        registry
    }

    /// Register a custom tool
    ///
    /// # Arguments
    ///
    /// * `tool` - Tool definition to register
    ///
    /// # Examples
    ///
    /// ```
    /// use xzepr_mcp::mcp::{ToolRegistry, ToolDefinition, RateLimitCategory, LatencyCategory};
    ///
    /// let mut registry = ToolRegistry::new();
    /// let tool = ToolDefinition {
    ///     name: "custom_tool".to_string(),
    ///     description: "A custom tool".to_string(),
    ///     required_scope: "xzepr:read".to_string(),
    ///     rate_limit_category: RateLimitCategory::Read,
    ///     input_schema: serde_json::json!({}),
    ///     latency_category: LatencyCategory::Fast,
    /// };
    /// registry.register(tool);
    /// ```
    pub fn register(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Get tool definition by name
    ///
    /// # Arguments
    ///
    /// * `name` - Tool name
    ///
    /// # Returns
    ///
    /// Returns the tool definition if found
    ///
    /// # Examples
    ///
    /// ```
    /// use xzepr_mcp::mcp::ToolRegistry;
    ///
    /// let registry = ToolRegistry::new();
    /// let tool = registry.get("fetch_event");
    /// assert!(tool.is_some());
    /// ```
    pub fn get(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// List all registered tools
    ///
    /// # Returns
    ///
    /// Returns a vector of all tool definitions
    ///
    /// # Examples
    ///
    /// ```
    /// use xzepr_mcp::mcp::ToolRegistry;
    ///
    /// let registry = ToolRegistry::new();
    /// let tools = registry.list();
    /// assert_eq!(tools.len(), 9);
    /// ```
    pub fn list(&self) -> Vec<&ToolDefinition> {
        self.tools.values().collect()
    }

    /// Register event-related tools
    fn register_event_tools(&mut self) {
        // fetch_event: Retrieve event by ULID
        self.register(ToolDefinition {
            name: "fetch_event".to_string(),
            description: "Retrieve a specific event by its ULID identifier".to_string(),
            required_scope: "xzepr:read".to_string(),
            rate_limit_category: RateLimitCategory::Read,
            latency_category: LatencyCategory::Fast,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "event_id": {
                        "type": "string",
                        "description": "Event ULID (26 characters)",
                        "pattern": "^[0-9A-HJKMNP-TV-Z]{26}$"
                    }
                },
                "required": ["event_id"],
                "additionalProperties": false
            }),
        });

        // create_event: Create new event
        self.register(ToolDefinition {
            name: "create_event".to_string(),
            description: "Create a new event in the XZepr system".to_string(),
            required_scope: "xzepr:write".to_string(),
            rate_limit_category: RateLimitCategory::Write,
            latency_category: LatencyCategory::Medium,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "event_type": {
                        "type": "string",
                        "description": "Event type identifier (e.g., 'user.created')",
                        "minLength": 1,
                        "maxLength": 255
                    },
                    "data": {
                        "type": "object",
                        "description": "Event payload data"
                    },
                    "source": {
                        "type": "string",
                        "description": "Event source identifier",
                        "maxLength": 255
                    },
                    "version": {
                        "type": "string",
                        "description": "Event schema version (semver format)",
                        "pattern": "^\\d+\\.\\d+\\.\\d+$"
                    }
                },
                "required": ["event_type", "data"],
                "additionalProperties": false
            }),
        });

        // search_events: Search events with filters
        self.register(ToolDefinition {
            name: "search_events".to_string(),
            description: "Search for events matching specified criteria".to_string(),
            required_scope: "xzepr:read".to_string(),
            rate_limit_category: RateLimitCategory::Search,
            latency_category: LatencyCategory::Medium,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "event_type": {
                        "type": "string",
                        "description": "Filter by event type"
                    },
                    "source": {
                        "type": "string",
                        "description": "Filter by event source"
                    },
                    "from_timestamp": {
                        "type": "string",
                        "description": "Start timestamp (ISO 8601 format)",
                        "format": "date-time"
                    },
                    "to_timestamp": {
                        "type": "string",
                        "description": "End timestamp (ISO 8601 format)",
                        "format": "date-time"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 10
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Pagination offset",
                        "minimum": 0,
                        "default": 0
                    }
                },
                "additionalProperties": false
            }),
        });
    }

    /// Register receiver-related tools
    fn register_receiver_tools(&mut self) {
        // fetch_receiver: Retrieve receiver by ULID
        self.register(ToolDefinition {
            name: "fetch_receiver".to_string(),
            description: "Retrieve a specific event receiver by its ULID identifier".to_string(),
            required_scope: "xzepr:read".to_string(),
            rate_limit_category: RateLimitCategory::Read,
            latency_category: LatencyCategory::Fast,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "receiver_id": {
                        "type": "string",
                        "description": "Receiver ULID (26 characters)",
                        "pattern": "^[0-9A-HJKMNP-TV-Z]{26}$"
                    }
                },
                "required": ["receiver_id"],
                "additionalProperties": false
            }),
        });

        // create_receiver: Create new receiver
        self.register(ToolDefinition {
            name: "create_receiver".to_string(),
            description: "Create a new event receiver in the XZepr system".to_string(),
            required_scope: "xzepr:write".to_string(),
            rate_limit_category: RateLimitCategory::Write,
            latency_category: LatencyCategory::Medium,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Receiver name",
                        "minLength": 1,
                        "maxLength": 255
                    },
                    "type": {
                        "type": "string",
                        "description": "Receiver type (webhook, kafka, etc.)",
                        "enum": ["webhook", "kafka", "grpc", "sqs", "pubsub"]
                    },
                    "config": {
                        "type": "object",
                        "description": "Receiver-specific configuration"
                    },
                    "event_types": {
                        "type": "array",
                        "description": "Event types to receive",
                        "items": {
                            "type": "string"
                        }
                    }
                },
                "required": ["name", "type", "config"],
                "additionalProperties": false
            }),
        });

        // search_receivers: Search receivers with filters
        self.register(ToolDefinition {
            name: "search_receivers".to_string(),
            description: "Search for event receivers matching specified criteria".to_string(),
            required_scope: "xzepr:read".to_string(),
            rate_limit_category: RateLimitCategory::Search,
            latency_category: LatencyCategory::Medium,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Filter by receiver name (partial match)"
                    },
                    "type": {
                        "type": "string",
                        "description": "Filter by receiver type"
                    },
                    "active": {
                        "type": "boolean",
                        "description": "Filter by active status"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 10
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Pagination offset",
                        "minimum": 0,
                        "default": 0
                    }
                },
                "additionalProperties": false
            }),
        });
    }

    /// Register group-related tools
    fn register_group_tools(&mut self) {
        // fetch_group: Retrieve group by ULID
        self.register(ToolDefinition {
            name: "fetch_group".to_string(),
            description: "Retrieve a specific event group by its ULID identifier".to_string(),
            required_scope: "xzepr:read".to_string(),
            rate_limit_category: RateLimitCategory::Read,
            latency_category: LatencyCategory::Fast,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "group_id": {
                        "type": "string",
                        "description": "Group ULID (26 characters)",
                        "pattern": "^[0-9A-HJKMNP-TV-Z]{26}$"
                    }
                },
                "required": ["group_id"],
                "additionalProperties": false
            }),
        });

        // create_group: Create new group
        self.register(ToolDefinition {
            name: "create_group".to_string(),
            description: "Create a new event group in the XZepr system".to_string(),
            required_scope: "xzepr:write".to_string(),
            rate_limit_category: RateLimitCategory::Write,
            latency_category: LatencyCategory::Medium,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Group name",
                        "minLength": 1,
                        "maxLength": 255
                    },
                    "description": {
                        "type": "string",
                        "description": "Group description",
                        "maxLength": 1000
                    },
                    "event_types": {
                        "type": "array",
                        "description": "Event types in this group",
                        "items": {
                            "type": "string"
                        },
                        "minItems": 1
                    },
                    "receivers": {
                        "type": "array",
                        "description": "Receiver IDs to associate with this group",
                        "items": {
                            "type": "string",
                            "pattern": "^[0-9A-HJKMNP-TV-Z]{26}$"
                        }
                    }
                },
                "required": ["name", "event_types"],
                "additionalProperties": false
            }),
        });

        // search_groups: Search groups with filters
        self.register(ToolDefinition {
            name: "search_groups".to_string(),
            description: "Search for event groups matching specified criteria".to_string(),
            required_scope: "xzepr:read".to_string(),
            rate_limit_category: RateLimitCategory::Search,
            latency_category: LatencyCategory::Medium,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Filter by group name (partial match)"
                    },
                    "event_type": {
                        "type": "string",
                        "description": "Filter by event type membership"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 10
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Pagination offset",
                        "minimum": 0,
                        "default": 0
                    }
                },
                "additionalProperties": false
            }),
        });
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_creation() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.tools.len(), 9);
    }

    #[test]
    fn test_tool_registry_default() {
        let registry = ToolRegistry::default();
        assert_eq!(registry.tools.len(), 9);
    }

    #[test]
    fn test_tool_registry_list() {
        let registry = ToolRegistry::new();
        let tools = registry.list();
        assert_eq!(tools.len(), 9);
    }

    #[test]
    fn test_event_tools_registered() {
        let registry = ToolRegistry::new();

        assert!(registry.get("fetch_event").is_some());
        assert!(registry.get("create_event").is_some());
        assert!(registry.get("search_events").is_some());
    }

    #[test]
    fn test_receiver_tools_registered() {
        let registry = ToolRegistry::new();

        assert!(registry.get("fetch_receiver").is_some());
        assert!(registry.get("create_receiver").is_some());
        assert!(registry.get("search_receivers").is_some());
    }

    #[test]
    fn test_group_tools_registered() {
        let registry = ToolRegistry::new();

        assert!(registry.get("fetch_group").is_some());
        assert!(registry.get("create_group").is_some());
        assert!(registry.get("search_groups").is_some());
    }

    #[test]
    fn test_tool_scopes() {
        let registry = ToolRegistry::new();

        // Read tools
        let fetch_event = registry.get("fetch_event").unwrap();
        assert_eq!(fetch_event.required_scope, "xzepr:read");

        let fetch_receiver = registry.get("fetch_receiver").unwrap();
        assert_eq!(fetch_receiver.required_scope, "xzepr:read");

        let fetch_group = registry.get("fetch_group").unwrap();
        assert_eq!(fetch_group.required_scope, "xzepr:read");

        // Write tools
        let create_event = registry.get("create_event").unwrap();
        assert_eq!(create_event.required_scope, "xzepr:write");

        let create_receiver = registry.get("create_receiver").unwrap();
        assert_eq!(create_receiver.required_scope, "xzepr:write");

        let create_group = registry.get("create_group").unwrap();
        assert_eq!(create_group.required_scope, "xzepr:write");
    }

    #[test]
    fn test_tool_rate_limit_categories() {
        let registry = ToolRegistry::new();

        let fetch_tool = registry.get("fetch_event").unwrap();
        assert_eq!(fetch_tool.rate_limit_category, RateLimitCategory::Read);

        let create_tool = registry.get("create_event").unwrap();
        assert_eq!(create_tool.rate_limit_category, RateLimitCategory::Write);

        let search_tool = registry.get("search_events").unwrap();
        assert_eq!(search_tool.rate_limit_category, RateLimitCategory::Search);
    }

    #[test]
    fn test_tool_latency_categories() {
        let registry = ToolRegistry::new();

        let fetch_tool = registry.get("fetch_event").unwrap();
        assert_eq!(fetch_tool.latency_category, LatencyCategory::Fast);

        let create_tool = registry.get("create_event").unwrap();
        assert_eq!(create_tool.latency_category, LatencyCategory::Medium);

        let search_tool = registry.get("search_events").unwrap();
        assert_eq!(search_tool.latency_category, LatencyCategory::Medium);
    }

    #[test]
    fn test_register_custom_tool() {
        let mut registry = ToolRegistry::new();

        let custom_tool = ToolDefinition {
            name: "custom_tool".to_string(),
            description: "A custom tool".to_string(),
            required_scope: "xzepr:read".to_string(),
            rate_limit_category: RateLimitCategory::Read,
            latency_category: LatencyCategory::Fast,
            input_schema: serde_json::json!({}),
        };

        registry.register(custom_tool);

        let tool = registry.get("custom_tool");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().description, "A custom tool");
    }

    #[test]
    fn test_tool_input_schemas_valid() {
        let registry = ToolRegistry::new();

        // Verify all tools have valid JSON schemas
        for tool in registry.list() {
            assert!(tool.input_schema.is_object());
            let schema = tool.input_schema.as_object().unwrap();
            assert_eq!(schema.get("type").unwrap().as_str().unwrap(), "object");
        }
    }

    #[test]
    fn test_fetch_event_schema() {
        let registry = ToolRegistry::new();
        let tool = registry.get("fetch_event").unwrap();

        let schema = tool.input_schema.as_object().unwrap();
        let properties = schema.get("properties").unwrap().as_object().unwrap();

        assert!(properties.contains_key("event_id"));
        assert_eq!(
            properties
                .get("event_id")
                .unwrap()
                .get("type")
                .unwrap()
                .as_str()
                .unwrap(),
            "string"
        );
    }

    #[test]
    fn test_create_event_schema() {
        let registry = ToolRegistry::new();
        let tool = registry.get("create_event").unwrap();

        let schema = tool.input_schema.as_object().unwrap();
        let required = schema.get("required").unwrap().as_array().unwrap();

        assert!(required.contains(&serde_json::json!("event_type")));
        assert!(required.contains(&serde_json::json!("data")));
    }

    #[test]
    fn test_search_events_schema() {
        let registry = ToolRegistry::new();
        let tool = registry.get("search_events").unwrap();

        let schema = tool.input_schema.as_object().unwrap();
        let properties = schema.get("properties").unwrap().as_object().unwrap();

        assert!(properties.contains_key("limit"));
        assert!(properties.contains_key("offset"));
    }
}
