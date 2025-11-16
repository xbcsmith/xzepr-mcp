//! XZepr HTTP client module
//!
//! This module provides an HTTP client for interacting with the XZepr API
//! with resilience patterns including:
//! - Automatic retries with exponential backoff
//! - Circuit breaker
//! - Connection pooling
//! - Request timeout
//!
//! Implementation is a stub for Phase 1 foundation.

mod xzepr;

pub use xzepr::XzeprClient;
