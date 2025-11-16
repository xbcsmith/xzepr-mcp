//! MCP protocol implementation module
//!
//! This module provides the Model Context Protocol (MCP) server implementation
//! for XZepr, including:
//! - MCP server setup with StreamableHTTP transport
//! - Tool definitions and schemas
//! - Tool request handlers
//! - Response formatting
//! - Session management integration
//!
//! # Components
//!
//! - `server` - MCP server setup and lifecycle management
//! - `tools` - Tool definitions with security metadata
//! - `handlers` - Tool request handlers with authentication and validation
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
//! let server = McpServer::new(settings);
//! // server.start().await?;
//! # Ok(())
//! # }
//! ```
//!
//! Implementation is a stub for Phase 1 foundation.

mod handlers;
mod server;
mod tools;

pub use handlers::ToolHandlers;
pub use server::McpServer;
pub use tools::{ToolDefinition, ToolRegistry};
