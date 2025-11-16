//! Configuration module for XZepr MCP
//!
//! This module provides configuration management with support for:
//! - YAML configuration files
//! - Environment variable overrides
//! - CLI argument integration
//! - Configuration validation
//!
//! # Example
//!
//! ```no_run
//! use xzepr_mcp::config::Settings;
//!
//! let settings = Settings::load(Some("config/production.yaml")).unwrap();
//! println!("Server listening on {}:{}", settings.server.host, settings.server.port);
//! ```

mod settings;

pub use settings::{
    AuthConfig, ObservabilityConfig, PerToolRateLimits, RateLimitConfig, SecurityConfig,
    ServerConfig, Settings, XzeprConfig,
};
