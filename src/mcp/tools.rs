//! MCP tool definitions
//!
//! This module provides tool definitions with security metadata for the MCP server.
//! Each tool includes schema validation, required scopes, and rate limit categories.
//!
//! Implementation is a stub for Phase 1 foundation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool definition with security metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Required scope (e.g., "xzepr:read" or "xzepr:write")
    pub required_scope: String,

    /// Rate limit category (read, write, search)
    pub rate_limit_category: RateLimitCategory,

    /// Input schema (JSON Schema)
    pub input_schema: serde_json::Value,
}

/// Rate limit category for tools
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RateLimitCategory {
    /// Read operations (higher limits)
    Read,

    /// Write operations (lower limits)
    Write,

    /// Search operations (medium limits)
    Search,
}

/// Tool registry
pub struct ToolRegistry {
    /// Registered tools
    tools: HashMap<String, ToolDefinition>,
}

impl ToolRegistry {
    /// Create a new tool registry
    ///
    /// # Returns
    ///
    /// Returns a new `ToolRegistry` with default tools
    ///
    /// # Examples
    ///
    /// ```
    /// use xzepr_mcp::mcp::ToolRegistry;
    ///
    /// let registry = ToolRegistry::new();
    /// ```
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Register default tools
        registry.register_default_tools();

        registry
    }

    /// Register a tool
    ///
    /// # Arguments
    ///
    /// * `tool` - Tool definition to register
    pub fn register(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Get tool by name
    ///
    /// # Arguments
    ///
    /// * `name` - Tool name
    ///
    /// # Returns
    ///
    /// Returns the tool definition if found
    pub fn get(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// List all registered tools
    ///
    /// # Returns
    ///
    /// Returns a vector of all tool definitions
    pub fn list(&self) -> Vec<&ToolDefinition> {
        self.tools.values().collect()
    }

    /// Register default XZepr tools
    fn register_default_tools(&mut self) {
        // Get event tool
        self.register(ToolDefinition {
            name: "get_event".to_string(),
            description: "Retrieve an event by its ULID".to_string(),
            required_scope: "xzepr:read".to_string(),
            rate_limit_category: RateLimitCategory::Read,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "event_id": {
                        "type": "string",
                        "description": "Event ULID"
                    }
                },
                "required": ["event_id"]
            }),
        });

        // Create event tool
        self.register(ToolDefinition {
            name: "create_event".to_string(),
            description: "Create a new event in XZepr".to_string(),
            required_scope: "xzepr:write".to_string(),
            rate_limit_category: RateLimitCategory::Write,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "event_type": {
                        "type": "string",
                        "description": "Event type"
                    },
                    "data": {
                        "type": "object",
                        "description": "Event data payload"
                    },
                    "source": {
                        "type": "string",
                        "description": "Event source"
                    },
                    "version": {
                        "type": "string",
                        "description": "Event version (semver)"
                    }
                },
                "required": ["event_type", "data"]
            }),
        });

        // Search events tool
        self.register(ToolDefinition {
            name: "search_events".to_string(),
            description: "Search for events matching criteria".to_string(),
            required_scope: "xzepr:read".to_string(),
            rate_limit_category: RateLimitCategory::Search,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "event_type": {
                        "type": "string",
                        "description": "Event type filter"
                    },
                    "source": {
                        "type": "string",
                        "description": "Event source filter"
                    },
                    "from_timestamp": {
                        "type": "string",
                        "description": "Start timestamp (ISO 8601)"
                    },
                    "to_timestamp": {
                        "type": "string",
                        "description": "End timestamp (ISO 8601)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum results to return",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 10
                    }
                }
            }),
        });

        // List event types tool
        self.register(ToolDefinition {
            name: "list_event_types".to_string(),
            description: "List all available event types".to_string(),
            required_scope: "xzepr:read".to_string(),
            rate_limit_category: RateLimitCategory::Read,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        });

        // Get event schema tool
        self.register(ToolDefinition {
            name: "get_event_schema".to_string(),
            description: "Retrieve the schema for an event type".to_string(),
            required_scope: "xzepr:read".to_string(),
            rate_limit_category: RateLimitCategory::Read,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "event_type": {
                        "type": "string",
                        "description": "Event type"
                    }
                },
                "required": ["event_type"]
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
        assert!(!registry.tools.is_empty());
    }

    #[test]
    fn test_tool_registry_get() {
        let registry = ToolRegistry::new();
        let tool = registry.get("get_event");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name, "get_event");
    }

    #[test]
    fn test_tool_registry_list() {
        let registry = ToolRegistry::new();
        let tools = registry.list();
        assert!(tools.len() >= 5);
    }

    #[test]
    fn test_tool_definition_scopes() {
        let registry = ToolRegistry::new();

        let get_tool = registry.get("get_event").unwrap();
        assert_eq!(get_tool.required_scope, "xzepr:read");

        let create_tool = registry.get("create_event").unwrap();
        assert_eq!(create_tool.required_scope, "xzepr:write");
    }

    #[test]
    fn test_tool_definition_rate_limits() {
        let registry = ToolRegistry::new();

        let get_tool = registry.get("get_event").unwrap();
        assert_eq!(get_tool.rate_limit_category, RateLimitCategory::Read);

        let create_tool = registry.get("create_event").unwrap();
        assert_eq!(create_tool.rate_limit_category, RateLimitCategory::Write);

        let search_tool = registry.get("search_events").unwrap();
        assert_eq!(search_tool.rate_limit_category, RateLimitCategory::Search);
    }

    #[test]
    fn test_register_custom_tool() {
        let mut registry = ToolRegistry::new();

        let custom_tool = ToolDefinition {
            name: "custom_tool".to_string(),
            description: "A custom tool".to_string(),
            required_scope: "xzepr:read".to_string(),
            rate_limit_category: RateLimitCategory::Read,
            input_schema: serde_json::json!({}),
        };

        registry.register(custom_tool);

        let tool = registry.get("custom_tool");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().description, "A custom tool");
    }
}
