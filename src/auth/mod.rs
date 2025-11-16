//! Authentication and authorization module
//!
//! This module provides JWT validation, JWKS caching, and session management
//! for the XZepr MCP server.
//!
//! # Features
//!
//! - JWT token validation with signature verification
//! - JWKS fetching and caching with automatic refresh
//! - Scope-based authorization (xzepr:read, xzepr:write)
//! - Session management for StreamableHTTP transport
//!
//! # Example
//!
//! ```no_run
//! use xzepr_mcp::auth::JwtValidator;
//! use std::time::Duration;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let validator = JwtValidator::new(
//!     "https://keycloak.example.com/realms/xzepr".to_string(),
//!     "https://keycloak.example.com/realms/xzepr".to_string(),
//!     "xzepr-mcp".to_string(),
//!     Duration::from_secs(3600),
//!     true,
//! );
//!
//! let claims = validator.validate("eyJhbGc...").await?;
//! println!("Authenticated user: {}", claims.sub);
//! # Ok(())
//! # }
//! ```

mod jwt;
mod session;

pub use jwt::{Claims, JwtValidator};
pub use session::{Session, SessionManager};
