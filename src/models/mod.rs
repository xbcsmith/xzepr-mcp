//! Shared data models for XZepr MCP
//!
//! This module provides common data structures used across the application:
//! - Request models (MCP tool call requests)
//! - Response models (XZepr API responses)
//! - Event models (XZepr event structures)
//! - Error response models
//!
//! Implementation is a stub for Phase 1 foundation.

mod requests;
mod responses;

pub use requests::{ToolCallParams, ToolRequest};
pub use responses::{EventData, ToolResponse};
