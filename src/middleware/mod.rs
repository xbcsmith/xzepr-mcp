//! Middleware module for XZepr MCP
//!
//! This module provides middleware components for:
//! - Rate limiting (per-user and per-tool)
//! - Input validation and sanitization
//! - Request/response logging
//! - Security headers
//!
//! # Components
//!
//! - `rate_limit` - Rate limiting middleware with per-user and per-tool limits
//! - `validation` - Input validation and injection detection
//!
//! # Example
//!
//! ```no_run
//! use xzepr_mcp::middleware::RateLimiter;
//! use xzepr_mcp::config::Settings;
//!
//! let settings = Settings::default();
//! let rate_limiter = RateLimiter::new(&settings.rate_limit);
//! ```

mod rate_limit;
mod validation;

pub use rate_limit::RateLimiter;
pub use validation::{InputValidator, ValidationRules};
