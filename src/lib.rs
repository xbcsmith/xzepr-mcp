//! `XZepr` MCP Server
//!
//! A Model Context Protocol (MCP) server implementation for the `XZepr` Event Tracking System.
//! This server acts as a protocol adapter, translating MCP tool calls into `XZepr` API operations.
//!
//! # Architecture
//!
//! `XZepr`-MCP uses a simple modular architecture:
//!
//! - `config/` - Configuration management (YAML files, environment variables)
//! - `client/` - `XZepr` HTTP client with resilience patterns
//! - `auth/` - OIDC/JWT authentication and session management
//! - `middleware/` - Rate limiting, validation, and security middleware
//! - `mcp/` - MCP protocol implementation (server, tools, handlers)
//! - `models/` - Shared data structures
//! - `observability/` - Metrics, tracing, and audit logging
//! - `error` - Error types and handling
//!
//! # Security Features
//!
//! - **Authentication**: OIDC/JWT validation with JWKS caching
//! - **Authorization**: Scope-based access control (xzepr:read, xzepr:write)
//! - **Input Validation**: Schema validation, injection detection, ULID/semver validation
//! - **Rate Limiting**: Per-user and per-tool rate limits
//! - **Audit Logging**: Comprehensive audit trail with correlation IDs
//! - **Session Security**: Secure session management for `StreamableHTTP` transport
//!
//! # Example
//!
//! ```no_run
//! use xzepr_mcp::config::Settings;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration
//!     let settings = Settings::load(Some("config/production.yaml"))?;
//!
//!     // Initialize observability
//!     // let _guard = xzepr_mcp::observability::init(&settings)?;
//!
//!     // Start MCP server
//!     // xzepr_mcp::mcp::start_server(settings).await?;
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::unused_async)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::single_char_pattern)]
#![allow(clippy::items_after_test_module)]
#![allow(clippy::manual_let_else)]

pub mod config;
pub mod error;

// Implemented modules
pub mod auth;
pub mod client;
pub mod mcp;
pub mod middleware;
pub mod models;
pub mod observability;

// Re-export commonly used types
pub use error::{Error, Result};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_version_is_valid() {
        // VERSION is a compile-time constant from Cargo.toml, always non-empty
        assert!(!VERSION.is_empty());
        assert!(VERSION.contains('.'));
    }

    #[test]
    fn test_name_is_correct() {
        assert_eq!(NAME, "xzepr-mcp");
    }
}
