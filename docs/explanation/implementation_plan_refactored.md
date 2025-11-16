# XZepr MCP Production Implementation Plan (Refactored)

## Overview

This document provides a comprehensive phased implementation plan to bring the XZepr MCP Rust server from initial architecture to production-ready deployment. The plan emphasizes security-first development, incorporating mandatory authentication, input validation, rate limiting, and observability from the earliest phases.

The implementation follows a four-phase approach with integrated security controls, comprehensive testing, and production hardening. Each phase builds incrementally on the previous, with clear success criteria and validation gates.

**Revision History**:
- Version 1.0: Initial implementation plan
- Version 2.0: Refactored based on validation analysis (2025-01-07)
  - Added OpenAPI specification generation (Task 4.1a)
  - Enhanced input validation requirements (Task 1.3)
  - Clarified rate limiting specifications (Task 1.6)
  - Enhanced audit logging requirements (Task 1.7)
  - Updated module structure setup (Task 1.1)
  - Revised timeline estimates (+7-9 days)

## Current State Analysis

### Existing Infrastructure

**XZepr Core Server (Rust)**
- Production-ready event tracking system
- RESTful API with versioned endpoints (`/api/v1/events`, `/api/v1/receivers`, `/api/v1/groups`)
- OIDC/Keycloak authentication support
- Deployed and operational

**EPR MCP Python Reference Implementation**
- Basic MCP protocol implementation in Python
- Demonstrates tool definitions and stdio transport
- Not production-grade (lacks authentication, validation, rate limiting)
- Serves as reference for tool schema design

**Architecture Documentation**
- Comprehensive architecture document exists (`xzepr_mcp_rust_architecture.md`)
- Security gap analysis completed
- Design decisions documented
- Module structure defined

### Identified Issues

**Security Gaps (Critical - Must Address)**
- No authentication/authorization implemented
- No input validation or injection detection
- No rate limiting or abuse prevention
- No audit logging or monitoring
- Session security undefined for StreamableHTTP transport

**Missing Core Functionality**
- No XZepr HTTP client with retry/circuit breaker
- No configuration management system
- No MCP protocol handlers
- No error handling framework
- No observability instrumentation

**Production Readiness Gaps**
- No deployment artifacts (Docker, Kubernetes)
- No CLI implementation
- No health check endpoints with OpenAPI documentation
- No performance testing baseline
- No security testing suite

## Implementation Phases

### Phase 1: Foundation & Security Infrastructure

**Duration**: 18-24 days (revised from 15-20 days)
**Priority**: Critical (Blocking all other work)

This phase establishes the secure foundation required for all subsequent development. Security controls are implemented first, not retrofitted later.

#### Task 1.1: Project Setup & Configuration System

**Objective**: Initialize complete project structure and implement configuration management with security defaults

**Duration**: 3 days

**Files to Create**:
- `xzepr-mcp/Cargo.toml` - Project manifest with security-focused dependencies
- `xzepr-mcp/src/main.rs` - Application entry point with CLI parsing
- `xzepr-mcp/src/lib.rs` - Library root for testability
- `xzepr-mcp/src/config/mod.rs` - Configuration module
- `xzepr-mcp/src/config/settings.rs` - Settings struct with validation
- `xzepr-mcp/src/validation/mod.rs` - Validation module root (stub)
- `xzepr-mcp/src/api/mod.rs` - API module root (stub)
- `xzepr-mcp/src/xzepr/mod.rs` - XZepr client module root (stub)
- `xzepr-mcp/src/auth/mod.rs` - Auth module root (stub)
- `xzepr-mcp/src/mcp/mod.rs` - MCP module root (stub)
- `xzepr-mcp/src/utils/mod.rs` - Utils module root
- `xzepr-mcp/src/error.rs` - Error types module
- `xzepr-mcp/config/default.yaml` - Default configuration
- `xzepr-mcp/config/production.yaml` - Production configuration template

**Dependencies to Add** (grouped by functional area):

Configuration & CLI:
- `config` (0.14+) - Multi-source configuration
- `serde` (1.0+) with derive feature
- `serde_json` (1.0+)
- `serde_yaml` (0.9+)
- `clap` (4.0+) with derive feature for CLI
- `secrecy` (0.8+) for sensitive value handling

MCP Protocol:
- `rmcp` (latest) - Rust MCP SDK
- `axum` (0.7+) - HTTP framework for transport
- `tokio` (1.35+) with full feature

HTTP Client:
- `reqwest` (0.11+) with rustls-tls feature
- `tower` (0.4+) - Middleware framework
- `tower-http` (0.5+) with cors, limit, trace features

Authentication:
- `jsonwebtoken` (9.0+) - JWT validation
- `openidconnect` (3.0+) - OIDC discovery and JWKS

Validation:
- `validator` (0.16+) with derive feature
- `regex` (1.10+) - Pattern matching for injection detection
- `semver` (1.0+) - Semantic version validation

Observability:
- `tracing` (0.1+)
- `tracing-subscriber` (0.3+) with json, env-filter features
- `tracing-opentelemetry` (0.22+)
- `prometheus` (0.13+)

OpenAPI:
- `utoipa` (4.0+) with axum feature - OpenAPI generation
- `utoipa-swagger-ui` (6.0+) with axum feature - OpenAPI UI

Rate Limiting:
- `governor` (0.6+) - Token bucket rate limiting

Utilities:
- `ulid` (1.1+) - ULID generation and validation
- `thiserror` (1.0+) - Error derive macros
- `uuid` (1.6+) - Session ID generation

Development:
- `mockito` (1.0+) - HTTP mocking for tests
- `wiremock` (0.6+) - Additional HTTP mocking
- `test-log` (0.2+) - Capture logs in tests

**Implementation Steps**:
1. Initialize Cargo workspace with complete module structure (reference architecture doc)
2. Create stub `mod.rs` files for all modules (validation, api, xzepr, auth, mcp, utils)
3. Implement `Settings` struct with nested config types (`XZeprConfig`, `ServerConfig`, `OidcConfig`, `LoggingConfig`, `RateLimitConfig`)
4. Add configuration loading precedence: defaults → file → env vars → CLI args
5. Implement validation for required fields (OIDC issuer, audience, client_id)
6. Use `secrecy::Secret<String>` for tokens, client secrets, API keys
7. Add configuration file schema validation
8. Wire up basic logging with `tracing-subscriber`

**Configuration File Structure**:

```yaml
# config/default.yaml
xzepr:
  url: "http://localhost:8080"
  timeout: 30

server:
  host: "127.0.0.1"
  port: 3000
  mcp_path: "/mcp"
  rate_limit:
    requests_per_second: 10
    burst_size: 20
    per_tool_limits:
      fetch_event: 100
      search_events: 20
      create_event: 10
      fetch_receiver: 100
      search_receivers: 20
      create_receiver: 10
      fetch_group: 100
      search_groups: 20
      create_group: 10

logging:
  level: "info"
  json: false

oidc:
  issuer_url: "https://keycloak.example.com/realms/xzepr"
  client_id: "xzepr-mcp"
  jwks_cache_ttl: 3600
  audience: "xzepr-mcp"
  required_scopes:
    - "xzepr:read"
    - "xzepr:write"
```

**Testing Requirements**:
- Unit tests for configuration loading from each source
- Test precedence order (CLI overrides env overrides file)
- Test validation failures for missing required fields (issuer, audience, client_id)
- Test sensitive value redaction in debug output
- Test default value application

**Success Criteria**:
- [ ] All module directories created with stub files
- [ ] All dependencies added to Cargo.toml
- [ ] Configuration loads from YAML files with environment variable overrides
- [ ] CLI arguments properly override all other sources
- [ ] Sensitive values never appear in logs or debug output
- [ ] Validation errors provide clear messages for missing/invalid config
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with >80% coverage

**Validation Gate**: Configuration system must be fully functional before proceeding to Task 1.2.

---

#### Task 1.2: OIDC/JWT Authentication Implementation (MANDATORY)

**Objective**: Implement secure token validation with JWKS caching and scope enforcement

**Duration**: 5-7 days (revised from 4-5 days)

**Files to Create**:
- `xzepr-mcp/src/auth/mod.rs` - Authentication module root (expand stub)
- `xzepr-mcp/src/auth/jwt.rs` - JWT validation logic
- `xzepr-mcp/src/auth/jwks.rs` - JWKS cache implementation
- `xzepr-mcp/src/auth/scopes.rs` - Scope extraction and validation
- `xzepr-mcp/src/auth/middleware.rs` - Authentication middleware for HTTP
- `xzepr-mcp/src/auth/fingerprint.rs` - Token fingerprinting for audit

**Implementation Steps**:
1. Implement `JwksCache` struct with TTL-based refresh from OIDC issuer
2. Create JWT validation function that verifies:
   - Signature against JWKS public key
   - `iss` claim matches configured issuer
   - `aud` claim contains `xzepr-mcp` (CRITICAL for confused deputy prevention)
   - `exp` claim (token not expired)
   - `nbf` claim if present (not before time)
3. Extract scopes from token claims (support both `scope` string and `scopes` array)
4. Implement scope checking functions:
   - `has_scope(&self, scope: &str) -> bool`
   - `requires_any_scope(&self, scopes: &[&str]) -> Result<()>`
   - `requires_all_scopes(&self, scopes: &[&str]) -> Result<()>`
5. Create authentication middleware that:
   - Extracts Bearer token from Authorization header
   - Validates JWT and extracts claims
   - Adds validated user context to request extensions
6. Implement token fingerprinting (SHA-256 of token, first 16 chars for audit logs)
7. Add automatic JWKS refresh on validation failure (kid not found)
8. Implement OIDC discovery endpoint fetching

**Security Requirements** (MANDATORY):
- MUST validate JWT signature against JWKS
- MUST verify `iss` claim matches configured issuer exactly
- MUST verify `aud` claim contains `xzepr-mcp` (prevents confused deputy attacks)
- MUST check `exp` claim (reject expired tokens)
- MUST check `nbf` claim if present (not before time)
- MUST enforce scope requirements per tool
- NO bypass option - validation always enabled in production
- Token fingerprints in audit logs (never full tokens or secrets)
- MUST use constant-time comparison for security-sensitive string checks

**Per-Tool Scope Requirements**:
- Read operations require `xzepr:read`:
  - `fetch_event`, `fetch_receiver`, `fetch_group`
  - `search_events`, `search_receivers`, `search_groups`
- Write operations require `xzepr:write`:
  - `create_event`, `create_receiver`, `create_group`

**JWKS Cache Implementation**:

```rust
pub struct JwksCache {
    keys: Arc<RwLock<HashMap<String, Jwk>>>,
    last_refresh: Arc<RwLock<Instant>>,
    ttl: Duration,
    client: reqwest::Client,
    jwks_url: String,
}

impl JwksCache {
    pub async fn new(issuer_url: &str, ttl: Duration) -> Result<Self, AuthError> {
        // Discover JWKS URL from OIDC .well-known endpoint
        // Fetch initial keys
    }

    pub async fn get_key(&self, kid: &str) -> Result<Jwk, AuthError> {
        // Check cache
        // If expired or key not found, refresh
        // Return key
    }

    async fn refresh(&self) -> Result<(), AuthError> {
        // Fetch JWKS from issuer
        // Update cache atomically
        // Update last_refresh timestamp
    }
}
```

**Testing Requirements**:
- Unit tests with mock JWKS endpoint (use mockito/wiremock)
- Test valid token acceptance with correct claims
- Test rejection of expired tokens (`exp` in past)
- Test rejection of tokens not yet valid (`nbf` in future)
- Test rejection of invalid signatures
- Test rejection of wrong audience (CRITICAL TEST)
- Test rejection of wrong issuer
- Test rejection of tokens without required scopes
- Test scope extraction from both `scope` string and `scopes` array formats
- Test JWKS cache refresh on TTL expiry
- Test JWKS cache refresh on unknown kid
- Integration test with real Keycloak test instance (optional, recommended)
- Test token fingerprint generation uniqueness

**Success Criteria**:
- [ ] Valid JWT with correct audience and scopes accepted
- [ ] Invalid/expired tokens rejected with descriptive errors
- [ ] Wrong audience tokens rejected (CRITICAL)
- [ ] Tokens without required scopes rejected per tool
- [ ] JWKS keys cached and refreshed correctly
- [ ] Token validation latency <5ms (after initial JWKS fetch)
- [ ] Token fingerprints generated consistently
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with >80% coverage

**Validation Gate**: JWT validation must reject all invalid tokens before proceeding to Task 1.3.

---

#### Task 1.3: Input Validation & Injection Detection (MANDATORY)

**Objective**: Implement multi-layer input validation and prompt injection detection

**Duration**: 4-5 days (revised from 3-4 days)

**Files to Create**:
- `xzepr-mcp/src/validation/mod.rs` - Validation module root (expand stub)
- `xzepr-mcp/src/validation/injection.rs` - Injection pattern detection
- `xzepr-mcp/src/validation/sanitize.rs` - Content sanitization
- `xzepr-mcp/src/validation/ulid.rs` - ULID validation
- `xzepr-mcp/src/validation/semver.rs` - Semantic version validation
- `xzepr-mcp/src/validation/json.rs` - Recursive JSON validation
- `xzepr-mcp/src/validation/payload.rs` - Payload size validation

**Implementation Steps**:
1. Create input validation framework using `validator` crate
2. Implement prompt injection detection with pattern matching
3. Add content sanitization for text inputs (remove control chars, normalize whitespace)
4. Implement ULID format validation (26 chars, Crockford Base32)
5. Implement semantic version validation using `semver` crate
6. Add recursive JSON validation with depth limit (max 10 levels)
7. Add payload size validation (64KB max for JSON payloads)
8. Implement query parameter sanitization for search operations
9. Add content-type validation
10. Create validation middleware for all MCP tool inputs

**Injection Pattern Detection**:

```rust
const INJECTION_PATTERNS: &[&str] = &[
    "IGNORE PREVIOUS INSTRUCTIONS",
    "IGNORE ALL PREVIOUS",
    "DISREGARD ALL",
    "NEW INSTRUCTIONS:",
    "SYSTEM:",
    "<|im_start|>",
    "<|im_end|>",
    "You are now",
    "Disregard all",
    "Forget everything",
    "Act as",
    "Pretend you are",
    "[INST]",
    "[/INST]",
];

pub fn detect_prompt_injection(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    for pattern in INJECTION_PATTERNS {
        if lower.contains(&pattern.to_lowercase()) {
            return Some(format!("Potential injection pattern detected: {}", pattern));
        }
    }

    // Check for excessive repetition
    if has_excessive_repetition(&lower) {
        return Some("Excessive character repetition detected".to_string());
    }

    // Check for control characters
    if has_control_characters(text) {
        return Some("Control characters detected".to_string());
    }

    None
}

pub fn sanitize_input(input: &str, max_length: usize) -> String {
    // Remove control characters except \n, \r, \t
    let cleaned: String = input.chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
        .collect();

    // Normalize whitespace
    let normalized = cleaned.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // Truncate to max length
    normalized.chars().take(max_length).collect()
}

pub fn validate_json_depth(value: &serde_json::Value, max_depth: usize) -> Result<(), ValidationError> {
    fn check_depth(value: &serde_json::Value, current_depth: usize, max_depth: usize) -> Result<(), ValidationError> {
        if current_depth > max_depth {
            return Err(ValidationError::JsonDepthExceeded { max: max_depth });
        }

        match value {
            serde_json::Value::Object(map) => {
                for v in map.values() {
                    check_depth(v, current_depth + 1, max_depth)?;
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    check_depth(v, current_depth + 1, max_depth)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    check_depth(value, 0, max_depth)
}
```

**Validation Rules**:
- ULID: 26 characters, Crockford Base32 alphabet only
- Semantic Version: Valid semver format (e.g., "1.2.3", "1.0.0-alpha")
- JSON payload: Max 64KB size, max 10 levels depth
- Text inputs: Max 10,000 characters after sanitization
- Query parameters: Max 255 characters, alphanumeric + limited special chars
- Event names: Max 255 chars, no control characters
- Descriptions: Max 2,000 chars, sanitized

**Testing Requirements**:
- Unit tests for each validation function
- Test all injection patterns detected correctly
- Test sanitization removes control characters
- Test ULID validation accepts valid ULIDs and rejects invalid
- Test semver validation with valid and invalid versions
- Test JSON depth limit enforcement
- Test payload size rejection (>64KB)
- Test recursive JSON with deeply nested objects
- Test query parameter injection attempts
- Test edge cases (empty strings, null bytes, Unicode)
- Performance test validation overhead (<1ms per validation)

**Success Criteria**:
- [ ] All injection patterns detected correctly
- [ ] Input sanitization removes dangerous content
- [ ] ULID validation accurate
- [ ] Semver validation accurate
- [ ] JSON depth limits enforced
- [ ] Payload size limits enforced (64KB)
- [ ] Query parameters sanitized
- [ ] >95% detection rate for known injection patterns
- [ ] Clear error messages for validation failures
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with >80% coverage

**Validation Gate**: Input validation must block all test attack patterns before proceeding to Task 1.4.

---

#### Task 1.4: XZepr HTTP Client with Resilience

**Objective**: Implement resilient HTTP client for XZepr API with retry and circuit breaker

**Duration**: 3-4 days

**Files to Create**:
- `xzepr-mcp/src/xzepr/mod.rs` - XZepr client module root (expand stub)
- `xzepr-mcp/src/xzepr/client.rs` - HTTP client implementation
- `xzepr-mcp/src/xzepr/types.rs` - XZepr domain types (Event, EventReceiver, EventReceiverGroup)
- `xzepr-mcp/src/xzepr/error.rs` - XZepr client errors
- `xzepr-mcp/src/xzepr/retry.rs` - Retry policy implementation
- `xzepr-mcp/src/xzepr/circuit_breaker.rs` - Circuit breaker implementation

**Implementation Steps**:
1. Create `XZeprClient` struct with `reqwest::Client`
2. Implement retry policy with exponential backoff
3. Implement circuit breaker (failure threshold, success threshold, timeout)
4. Add connection pooling configuration
5. Implement all XZepr API methods:
   - `fetch_event(id: &str) -> Result<Event>`
   - `create_event(input: CreateEventInput) -> Result<Event>`
   - `search_events(criteria: SearchEventsInput) -> Result<Vec<Event>>`
   - `fetch_receiver(id: &str) -> Result<EventReceiver>`
   - `create_receiver(input: CreateReceiverInput) -> Result<EventReceiver>`
   - `search_receivers(criteria: SearchReceiversInput) -> Result<Vec<EventReceiver>>`
   - `fetch_group(id: &str) -> Result<EventReceiverGroup>`
   - `create_group(input: CreateGroupInput) -> Result<EventReceiverGroup>`
   - `search_groups(criteria: SearchGroupsInput) -> Result<Vec<EventReceiverGroup>>`
6. Add request/response logging with tracing spans
7. Add timeout configuration per operation type
8. Implement token passthrough (Bearer token from MCP request)

**Retry Policy**:
- Max attempts: 3
- Initial backoff: 100ms
- Max backoff: 2000ms
- Backoff multiplier: 2.0
- Retry on: 408, 429, 500, 502, 503, 504
- Do NOT retry on: 400, 401, 403, 404, 409

**Circuit Breaker**:
- Failure threshold: 5 consecutive failures
- Success threshold: 2 consecutive successes (to close)
- Timeout: 30 seconds in open state
- States: Closed (normal) → Open (failing) → Half-Open (testing)

**Testing Requirements**:
- Unit tests with mock HTTP responses (use mockito)
- Test successful API calls return correct data
- Test retry on transient errors (503)
- Test no retry on permanent errors (404)
- Test circuit breaker opens after threshold failures
- Test circuit breaker closes after successful recovery
- Test timeout enforcement
- Test connection pool reuse
- Integration test with real XZepr test instance (optional)

**Success Criteria**:
- [ ] All 9 XZepr API methods implemented
- [ ] Retry policy handles transient failures
- [ ] Circuit breaker prevents cascading failures
- [ ] Timeouts enforced correctly
- [ ] Connection pooling configured
- [ ] Request/response logging with correlation IDs
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with >80% coverage

---

#### Task 1.5: Error Handling Framework

**Objective**: Implement comprehensive error types and conversion

**Duration**: 1-2 days

**Files to Create/Modify**:
- `xzepr-mcp/src/error.rs` - Application-wide error types (expand)

**Implementation Steps**:
1. Define `McpError` enum with thiserror
2. Define `XZeprError` enum in `xzepr/error.rs`
3. Define `AuthError` enum in `auth/mod.rs`
4. Define `ValidationError` enum in `validation/mod.rs`
5. Implement error conversions (From trait implementations)
6. Add error context preservation
7. Add user-friendly error messages
8. Ensure no sensitive data in error messages

**Error Types**:

```rust
#[derive(Error, Debug)]
pub enum McpError {
    #[error("XZepr API error: {0}")]
    XZeprApi(#[from] XZeprError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error in field '{field}': {message}")]
    Validation { field: String, message: String },

    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("MCP protocol error: {0}")]
    Protocol(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token expired at {0}")]
    TokenExpired(String),

    #[error("Wrong audience: expected {expected}, got {actual}")]
    WrongAudience { expected: String, actual: String },

    #[error("Insufficient scope: {tool} requires {required}")]
    InsufficientScope { tool: String, required: String },

    #[error("JWKS error: {0}")]
    JwksError(String),
}
```

**Testing Requirements**:
- Test error creation and formatting
- Test error conversions
- Test error context preservation
- Test no sensitive data leakage in error messages

**Success Criteria**:
- [ ] All error types defined
- [ ] Error conversions implemented
- [ ] Clear error messages
- [ ] No sensitive data in errors
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with >80% coverage

---

#### Task 1.6: Rate Limiting Implementation (MANDATORY)

**Objective**: Implement per-user and per-tool rate limiting with standard headers

**Duration**: 2-3 days

**Files to Create**:
- `xzepr-mcp/src/mcp/rate_limit.rs` - Rate limiting middleware
- `xzepr-mcp/src/mcp/rate_limit_config.rs` - Rate limit configuration

**Implementation Steps**:
1. Integrate `governor` crate with token bucket algorithm
2. Configure global limits: 10 requests/second per user, burst of 20
3. Configure per-tool limits (requests per minute):
   - Fetch operations (fetch_event, fetch_receiver, fetch_group): 100/min
   - Search operations (search_events, search_receivers, search_groups): 20/min
   - Create operations (create_event, create_receiver, create_group): 10/min
4. Implement rate limit middleware that:
   - Extracts user identity from validated JWT (`sub` claim)
   - Applies user-specific limits (not global)
   - Returns 429 with Retry-After header when exceeded
   - Adds rate limit headers to all responses
5. Add rate limit headers to responses:
   - `X-RateLimit-Limit`: limit value (requests per minute)
   - `X-RateLimit-Remaining`: remaining requests in current window
   - `X-RateLimit-Reset`: unix timestamp of window reset
   - `Retry-After`: seconds to wait (on 429 only)
6. Implement rate limit metrics for monitoring
7. Add rate limit event logging

**Rate Limit Configuration**:

```rust
pub struct RateLimitConfig {
    pub global: RateLimit,
    pub per_tool: HashMap<String, RateLimit>,
}

pub struct RateLimit {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut per_tool = HashMap::new();

        // Fetch operations: 100 req/min
        per_tool.insert("fetch_event".to_string(), RateLimit { requests_per_minute: 100, burst_size: 10 });
        per_tool.insert("fetch_receiver".to_string(), RateLimit { requests_per_minute: 100, burst_size: 10 });
        per_tool.insert("fetch_group".to_string(), RateLimit { requests_per_minute: 100, burst_size: 10 });

        // Search operations: 20 req/min (more expensive)
        per_tool.insert("search_events".to_string(), RateLimit { requests_per_minute: 20, burst_size: 5 });
        per_tool.insert("search_receivers".to_string(), RateLimit { requests_per_minute: 20, burst_size: 5 });
        per_tool.insert("search_groups".to_string(), RateLimit { requests_per_minute: 20, burst_size: 5 });

        // Create operations: 10 req/min (write operations)
        per_tool.insert("create_event".to_string(), RateLimit { requests_per_minute: 10, burst_size: 3 });
        per_tool.insert("create_receiver".to_string(), RateLimit { requests_per_minute: 10, burst_size: 3 });
        per_tool.insert("create_group".to_string(), RateLimit { requests_per_minute: 10, burst_size: 3 });

        Self {
            global: RateLimit { requests_per_minute: 600, burst_size: 20 }, // 10 req/s
            per_tool,
        }
    }
}
```

**Response Headers**:

```
HTTP/1.1 200 OK
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1704672060

HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1704672060
Retry-After: 23
```

**Testing Requirements**:
- Unit tests for rate limit calculations
- Test user-specific rate limiting (different users, independent limits)
- Test per-tool rate limit differentiation
- Test global rate limit enforcement
- Test burst allowance behavior
- Test rate limit header presence and accuracy
- Test 429 response format with Retry-After header
- Test rate limit reset after window expiry
- Performance test rate limiting overhead (<0.5ms)

**Success Criteria**:
- [ ] Rate limiting enforced per user (JWT sub claim)
- [ ] Per-tool limits differentiated correctly
- [ ] Global limits enforced
- [ ] Rate limit headers present in all responses
- [ ] 429 responses include Retry-After header
- [ ] Rate limits configurable via config file
- [ ] Rate limit events logged for monitoring
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with >80% coverage

---

#### Task 1.7: Observability Infrastructure

**Objective**: Implement structured logging, metrics, and tracing

**Duration**: 2-3 days

**Files to Create**:
- `xzepr-mcp/src/observability/mod.rs` - Observability module
- `xzepr-mcp/src/observability/metrics.rs` - Prometheus metrics
- `xzepr-mcp/src/observability/tracing.rs` - Distributed tracing setup
- `xzepr-mcp/src/observability/logging.rs` - Structured logging setup

**Implementation Steps**:
1. Set up `tracing` with structured logging (JSON format for production)
2. Implement Prometheus metrics:
   - Request counters by tool and status
   - Request duration histograms
   - Error counters by type
   - Active connections gauge
   - Circuit breaker state gauge
3. Set up OpenTelemetry tracing (optional, for distributed tracing)
4. Add `/metrics` endpoint for Prometheus scraping
5. Implement audit logging for security events
6. Add request correlation IDs

**Audit Logging Schema**:

All audit events MUST include:
- `timestamp`: RFC-3339 format (e.g., `2025-01-07T18:12:07.982682Z`)
- `level`: INFO (success) or WARN (failure)
- `event_type`: One of:
  - `tool_invocation` - MCP tool called
  - `auth_failure` - Authentication failed
  - `rate_limit_exceeded` - Rate limit hit
  - `injection_detected` - Prompt injection attempt
  - `validation_failure` - Input validation failed
  - `circuit_breaker_opened` - Circuit breaker opened
- `user_id`: JWT `sub` claim (if authenticated)
- `token_fingerprint`: SHA-256 of JWT (first 16 chars, if authenticated)
- `tool`: Tool name (if applicable)
- `input_fingerprint`: SHA-256 of input JSON (if applicable)
- `result`: success | error
- `error_type`: Error category (if error)
- `duration_ms`: Execution time (if applicable)
- `request_id`: Correlation ID (UUID)

**Events to Log** (MANDATORY):
1. Tool invocation (start and completion with duration)
2. Authentication failures with reason:
   - Invalid token signature
   - Expired token
   - Wrong audience
   - Insufficient scope
   - Missing token
3. Rate limit exceeded events with user and tool
4. Injection pattern detection events with pattern matched
5. Validation failures with field and reason
6. XZepr API errors with status code
7. Circuit breaker state changes

**Prometheus Metrics**:

```rust
// Counters
xzepr_mcp_requests_total{tool, status} - Total requests by tool and status (success/error)
xzepr_mcp_errors_total{tool, error_type} - Total errors by tool and type
xzepr_api_calls_total{endpoint, method, status} - XZepr API calls

// Histograms
xzepr_mcp_request_duration_seconds{tool} - Request duration by tool
xzepr_api_call_duration_seconds{endpoint} - XZepr API call latency
xzepr_mcp_payload_size_bytes{tool, direction} - Request/response sizes

// Gauges
xzepr_mcp_active_connections - Current active MCP connections
xzepr_api_connection_pool_size - XZepr client connection pool size
xzepr_mcp_circuit_breaker_state{state} - Circuit breaker state (0=closed, 1=open, 2=half-open)
```

**Testing Requirements**:
- Test structured logging output format
- Test audit log completeness (all required fields)
- Test metrics increment correctly
- Test metrics endpoint returns Prometheus format
- Test correlation IDs propagate through request chain
- Test sensitive data not logged

**Success Criteria**:
- [ ] Structured logging configured (JSON in production)
- [ ] All security events logged with complete audit schema
- [ ] Prometheus metrics exposed at `/metrics`
- [ ] Correlation IDs present in all logs
- [ ] No sensitive data in logs (tokens, passwords, etc.)
- [ ] Metrics increment correctly for requests, errors, durations
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with >80% coverage

---

#### Phase 1 Deliverables

**Code Artifacts**:
- [ ] Complete module structure (config, mcp, api, auth, xzepr, validation, utils, error)
- [ ] Configuration system with multi-source loading
- [ ] OIDC/JWT authentication with JWKS caching
- [ ] Input validation and injection detection framework
- [ ] XZepr HTTP client with retry and circuit breaker
- [ ] Error handling framework
- [ ] Rate limiting middleware
- [ ] Observability infrastructure (metrics, logging, tracing)

**Documentation**:
- [ ] Configuration reference (`docs/reference/configuration.md`)
- [ ] Authentication setup guide (`docs/how_to/configure_oidc_authentication.md`)
- [ ] Security controls explanation (`docs/explanation/security_controls.md`)

**Tests**:
- [ ] Unit tests for all components (>80% coverage)
- [ ] Integration tests for configuration loading
- [ ] Security tests for auth validation
- [ ] Attack simulation tests for injection detection

**Configuration Files**:
- [ ] `config/default.yaml` - Default configuration
- [ ] `config/production.yaml` - Production template
- [ ] `.env.example` - Environment variable example

---

#### Phase 1 Success Criteria

**Functional**:
- [ ] Configuration loads from all sources with correct precedence
- [ ] JWT validation rejects all invalid tokens (including wrong audience)
- [ ] Input validation blocks all test injection patterns
- [ ] XZepr client successfully calls all API endpoints
- [ ] Rate limiting enforces per-user and per-tool limits
- [ ] Metrics and logs capture all security events

**Security** (CRITICAL):
- [ ] No bypass option for JWT validation
- [ ] Audience validation prevents confused deputy attacks
- [ ] Scope enforcement blocks unauthorized tool access
- [ ] Injection detection catches known attack patterns
- [ ] Rate limiting prevents abuse
- [ ] Audit logs capture all security-relevant events

**Quality**:
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes (zero errors)
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes (zero warnings)
- [ ] `cargo test --all-features` passes with >80% coverage
- [ ] No `unwrap()` or `expect()` without justification

**Validation Gate**: All Phase 1 success criteria must pass before proceeding to Phase 2.

---

### Phase 2: MCP Protocol Implementation

**Duration**: 12-15 days (revised from 10-12 days)
**Priority**: High (Depends on Phase 1)

This phase implements the MCP protocol server and tool handlers, integrating with the security infrastructure built in Phase 1.

#### Task 2.1: MCP Server Setup with StreamableHTTP Transport

**Objective**: Initialize MCP server using rmcp with StreamableHTTP transport and secure session management

**Duration**: 4-5 days

**Files to Create**:
- `xzepr-mcp/src/mcp/mod.rs` - MCP module root (expand stub)
- `xzepr-mcp/src/mcp/server.rs` - Server setup and lifecycle
- `xzepr-mcp/src/mcp/transport.rs` - StreamableHTTP transport configuration
- `xzepr-mcp/src/mcp/session.rs` - Session management with security
- `xzepr-mcp/src/mcp/context.rs` - Request context with user info

**Dependencies** (should already be in Cargo.toml):
- `rmcp` (latest) - Rust MCP SDK
- `axum` (0.7+) - HTTP framework for transport
- `tokio` (1.35+) with full feature
- `uuid` (1.6+) - Session ID generation

**Implementation Steps**:
1. Configure `StreamableHttpService` with `LocalSessionManager`
2. Implement secure session ID generation:
   - Use cryptographically random UUIDs (v4)
   - Session IDs must be unpredictable
3. Add session binding to JWT `sub` claim:
   - Store user_id with session
   - Verify user_id matches on each request
   - Prevent token substitution attacks
4. Implement session expiration:
   - 30 minute idle timeout
   - Track last activity timestamp
   - Clean up expired sessions
5. Add session rotation on authentication events:
   - Generate new session ID on JWT refresh
   - Invalidate old session ID
6. Wire up authentication middleware from Phase 1
7. Wire up rate limiting middleware from Phase 1
8. Wire up input validation from Phase 1
9. Add HTTP server configuration (host, port, CORS)
10. Implement graceful shutdown handling

**Session Security Implementation**:

```rust
pub struct SecureSession {
    pub id: String,              // Cryptographically random UUID
    pub user_id: String,         // JWT sub claim
    pub created_at: Instant,
    pub last_activity: Instant,
    pub token_fingerprint: String, // SHA-256 of JWT
}

impl SecureSession {
    pub fn new(user_id: String, token_fingerprint: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            token_fingerprint,
        }
    }

    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn verify_user(&self, user_id: &str) -> bool {
        // Constant-time comparison
        self.user_id == user_id
    }
}
```

**Security Requirements**:
- MUST generate cryptographically secure session IDs
- MUST bind sessions to authenticated user identity (JWT sub)
- MUST expire sessions after 30 minutes idle
- MUST rotate session IDs on re-authentication
- MUST reject requests with invalid session IDs
- MUST validate user_id matches session on each request
- MUST clean up expired sessions periodically

**Testing Requirements**:
- Unit tests for session generation and validation
- Test session binding to JWT claims
- Test session expiration enforcement
- Test session rotation on re-auth
- Test rejection of invalid session IDs
- Test user_id verification prevents token substitution
- Integration test with rmcp client
- Load test session management (1000+ concurrent sessions)

**Success Criteria**:
- [ ] MCP server starts and accepts StreamableHTTP connections
- [ ] Sessions properly bound to user identities
- [ ] Session expiration enforced correctly (30 min)
- [ ] Session rotation works on re-authentication
- [ ] Invalid session IDs rejected
- [ ] Authentication middleware integrated
- [ ] Rate limiting middleware integrated
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with >80% coverage

---

#### Task 2.2: MCP Tool Definitions with Security Metadata

**Objective**: Define all MCP tools with input schemas, security requirements, and resource descriptions

**Duration**: 2-3 days

**Files to Create**:
- `xzepr-mcp/src/mcp/tools.rs` - Tool definitions and schemas
- `xzepr-mcp/src/mcp/schemas.rs` - JSON schema definitions for tool inputs
- `xzepr-mcp/src/mcp/permissions.rs` - Per-tool permission mappings

**Implementation Steps**:
1. Define tool schemas for all 9 operations with complete input validation:
   - `fetch_event` - Fetch event by ULID
   - `create_event` - Create new event
   - `search_events` - Search events with filters
   - `fetch_receiver` - Fetch receiver by ULID
   - `create_receiver` - Create new receiver
   - `search_receivers` - Search receivers with filters
   - `fetch_group` - Fetch group by ULID
   - `create_group` - Create new group
   - `search_groups` - Search groups with filters
2. Map each tool to required scopes:
   - Read tools require `xzepr:read`
   - Write tools require `xzepr:write`
3. Add resource descriptions for MCP resources protocol
4. Define input validation rules per tool (using schemas from Phase 1)
5. Add tool metadata:
   - Rate limit tier (fetch/search/create)
   - Estimated latency (fast/medium/slow)
   - Required permissions
   - Resource type

**Tool-to-Scope Mapping**:

```rust
pub fn get_required_scopes(tool: &str) -> &[&str] {
    match tool {
        // Read operations - require xzepr:read
        "fetch_event" | "search_events" => &["xzepr:read"],
        "fetch_receiver" | "search_receivers" => &["xzepr:read"],
        "fetch_group" | "search_groups" => &["xzepr:read"],

        // Write operations - require xzepr:write
        "create_event" => &["xzepr:write"],
        "create_receiver" => &["xzepr:write"],
        "create_group" => &["xzepr:write"],

        _ => &[],
    }
}
```

**Tool Definitions**:

```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub required_scopes: Vec<String>,
    pub rate_limit_tier: RateLimitTier,
}

pub enum RateLimitTier {
    Fetch,   // 100 req/min
    Search,  // 20 req/min
    Create,  // 10 req/min
}
```

**Testing Requirements**:
- Unit tests for schema validation
- Test tool registration with MCP server
- Test permission mapping lookup for all tools
- Validate JSON schema definitions against MCP spec
- Test schema validation rejects invalid inputs

**Success Criteria**:
- [ ] All 9 tools registered with correct schemas
- [ ] Permission mappings enforce read/write separation
- [ ] Tool metadata accurate and complete
- [ ] Schemas validate correctly against MCP specification
- [ ] Input validation rules integrated
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with >80% coverage

---

#### Task 2.3: Tool Handler Implementation with Security Integration

**Objective**: Implement tool request handlers that enforce authentication, validation, and rate limiting

**Duration**: 4-5 days

**Files to Create**:
- `xzepr-mcp/src/mcp/handlers/mod.rs` - Handler module root
- `xzepr-mcp/src/mcp/handlers/events.rs` - Event tool handlers (fetch, create, search)
- `xzepr-mcp/src/mcp/handlers/receivers.rs` - Receiver tool handlers
- `xzepr-mcp/src/mcp/handlers/groups.rs` - Group tool handlers
- `xzepr-mcp/src/mcp/handlers/common.rs` - Shared handler utilities

**Implementation Steps**:
1. Implement handler function for each tool that:
   - Extracts validated JWT from request context
   - Validates JWT scopes for tool (Phase 1 auth)
   - Validates input using schemas (Phase 1 validation)
   - Checks injection patterns (Phase 1 validation)
   - Calls XZepr client with retry/circuit breaker (Phase 1 client)
   - Formats response according to MCP spec
   - Logs audit event with all required fields (Phase 1 observability)
2. Add automatic audit logging to all handlers:
   - Log tool invocation start
   - Log completion with duration
   - Log all errors with context
3. Implement error mapping from XZepr errors to MCP errors
4. Add request tracing spans to all handlers (OpenTelemetry)
5. Implement idempotency keys for write operations (optional)
6. Add response size limits (prevent memory exhaustion - 10MB max)

**Handler Pattern** (all handlers follow this):

```rust
pub async fn handle_fetch_event(
    ctx: &RequestContext,
    input: FetchEventInput,
) -> Result<Event, McpError> {
    let span = tracing::info_span!("fetch_event", request_id = %ctx.request_id);
    let _enter = span.enter();

    // 1. Validate JWT scopes
    ctx.scopes.requires_any_scope(&["xzepr:read"])?;

    // 2. Validate input
    input.validate()?;

    // 3. Check injection patterns
    if let Some(pattern) = detect_prompt_injection(&input.id) {
        return Err(McpError::Validation {
            field: "id".to_string(),
            message: pattern,
        });
    }

    // 4. Call XZepr API
    let event = ctx.xzepr_client.fetch_event(&input.id).await?;

    // 5. Log audit event
    audit_log::log_tool_invocation(
        "fetch_event",
        &ctx.user_id,
        &ctx.token_fingerprint,
        &input.id,
        "success",
        span.elapsed(),
    );

    Ok(event)
}
```

**Security Requirements** (enforced in all handlers):
- MUST validate JWT scopes before executing tool
- MUST validate all inputs before calling XZepr API
- MUST check for injection patterns in all string inputs
- MUST sanitize inputs before passing to XZepr
- MUST audit log all tool invocations with outcomes
- MUST enforce rate limits via middleware (automatic)
- MUST handle errors securely (no sensitive data leakage)
- MUST limit response sizes (10MB max)

**Testing Requirements**:
- Unit tests for each handler function
- Test successful tool execution
- Test rejection when insufficient scopes
- Test rejection when invalid input
- Test rejection when injection pattern detected
- Test error mapping (XZepr errors → MCP errors)
- Test audit logging for all outcomes
- Test response size limit enforcement
- Integration tests with mock XZepr API

**Success Criteria**:
- [ ] All 9 tool handlers implemented
- [ ] Scope validation enforced before execution
- [ ] Input validation integrated
- [ ] Injection detection integrated
- [ ] Audit logging captures all invocations
- [ ] Error handling secure (no data leakage)
- [ ] Response size limits enforced
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with >80% coverage

---

#### Task 2.4: Response Formatting & Error Handling

**Objective**: Implement MCP-compliant response formatting and error responses

**Duration**: 1-2 days

**Files to Create**:
- `xzepr-mcp/src/mcp/responses.rs` - Response formatting utilities
- `xzepr-mcp/src/mcp/errors.rs` - MCP error response formatting

**Implementation Steps**:
1. Implement MCP success response formatting
2. Implement MCP error response formatting
3. Add error code mapping (XZepr → MCP)
4. Ensure no sensitive data in error responses
5. Add response validation

**Testing Requirements**:
- Test response format compliance with MCP spec
- Test error responses don't leak sensitive data
- Test error code mapping

**Success Criteria**:
- [ ] Responses comply with MCP specification
- [ ] Error responses secure and informative
- [ ] Error code mapping accurate
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass with >80% coverage

---

#### Phase 2 Deliverables

**Code Artifacts**:
- [ ] MCP server with StreamableHTTP transport
- [ ] Secure session management with binding and rotation
- [ ] All 9 MCP tools defined with schemas
- [ ] All 9 tool handlers implemented with security
- [ ] Response formatting and error handling

**Documentation**:
- [ ] MCP tools reference (`docs/reference/mcp_tools.md`)
- [ ] Session security explanation (`docs/explanation/session_security.md`)
- [ ] Handler implementation guide (`docs/how_to/add_new_tool.md`)

**Tests**:
- [ ] Unit tests for all handlers (>80% coverage)
- [ ] Integration tests with MCP client
- [ ] Security tests for session management

---

#### Phase 2 Success Criteria

**Functional**:
- [ ] MCP server accepts StreamableHTTP connections
- [ ] All 9 tools respond correctly to valid requests
- [ ] Sessions managed securely with binding and expiration
- [ ] Responses comply with MCP specification

**Security**:
- [ ] Session IDs cryptographically secure
- [ ] Sessions bound to user identities
- [ ] Scope validation enforced in all handlers
- [ ] Input validation integrated in all handlers
- [ ] Audit logging captures all tool invocations

**Quality**:
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes (zero errors)
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes (zero warnings)
- [ ] `cargo test --all-features` passes with >80% coverage

**Validation Gate**: All Phase 2 success criteria must pass before proceeding to Phase 3.

---

### Phase 3: Testing, Documentation & Security Validation

**Duration**: 12-15 days (no change)
**Priority**: High (Depends on Phase 2)

This phase validates the implementation through comprehensive testing, security validation, and documentation completion.

#### Task 3.1: Comprehensive Unit Testing

**Objective**: Achieve >80% test coverage with comprehensive unit tests

**Duration**: 3-4 days

**Files to Create**:
- Unit tests in each module (`tests` submodule)
- Test utilities and fixtures

**Testing Focus**:
1. Configuration loading and validation
2. JWT validation (all failure scenarios)
3. Input validation (all injection patterns)
4. XZepr client (retry, circuit breaker)
5. Rate limiting logic
6. Session management
7. Tool handlers (success and failure paths)
8. Error handling and conversion

**Success Criteria**:
- [ ] >80% code coverage achieved
- [ ] All edge cases tested
- [ ] All error paths tested
- [ ] Tests run fast (<30 seconds total)

---

#### Task 3.2: Integration & End-to-End Testing

**Objective**: Test complete request flows with real components

**Duration**: 3-4 days

**Files to Create**:
- `xzepr-mcp/tests/integration_tests.rs` - Integration test suite
- `xzepr-mcp/tests/e2e_tests.rs` - End-to-end test suite
- `xzepr-mcp/tests/fixtures/` - Test fixtures and mock data

**Testing Focus**:
1. Complete request flows (auth → validation → handler → XZepr → response)
2. MCP client integration (using rmcp client)
3. Mock XZepr API responses (using wiremock)
4. Session lifecycle testing
5. Rate limit enforcement testing
6. Circuit breaker behavior testing
7. Error propagation testing

**Success Criteria**:
- [ ] All integration tests pass
- [ ] E2E tests cover happy paths for all tools
- [ ] Mock XZepr API comprehensive
- [ ] Tests isolated and repeatable

---

#### Task 3.3: Security Testing & Attack Simulation

**Objective**: Validate security controls against known attack patterns

**Duration**: 3-4 days

**Files to Create**:
- `xzepr-mcp/tests/security_tests.rs` - Security test suite
- `xzepr-mcp/tests/attack_simulations.rs` - Attack simulation tests

**Testing Focus**:
1. JWT attacks:
   - Expired tokens
   - Wrong audience
   - Invalid signatures
   - Missing scopes
   - Token replay
2. Injection attacks:
   - All known prompt injection patterns
   - JSON payload injection
   - Query parameter injection
3. Rate limit attacks:
   - Burst attempts
   - Sustained high rate
   - Per-tool limit bypass attempts
4. Session attacks:
   - Session fixation
   - Session hijacking
   - Token substitution
5. Input validation attacks:
   - Oversized payloads
   - Deeply nested JSON
   - Invalid ULIDs
   - Malformed semver

**Success Criteria**:
- [ ] All attack simulations blocked by security controls
- [ ] Security controls log all attack attempts
- [ ] No false positives in normal usage
- [ ] Security test coverage >95%

---

#### Task 3.4: Performance & Load Testing

**Objective**: Establish performance baselines and identify bottlenecks

**Duration**: 2-3 days

**Files to Create**:
- `xzepr-mcp/benches/` - Benchmark suite using criterion
- Performance test scripts

**Testing Focus**:
1. Request throughput (baseline: 100 req/s)
2. Response latency (p50, p95, p99)
3. Resource usage (CPU, memory)
4. Connection pool behavior
5. Rate limit overhead
6. Validation overhead
7. Session management scalability

**Performance Targets**:
- Throughput: 100 req/s (baseline without caching)
- Latency p50: <50ms
- Latency p95: <200ms
- Latency p99: <500ms
- Memory: <512MB under load
- CPU: <50% single core

**Success Criteria**:
- [ ] Performance baselines documented
- [ ] No memory leaks detected
- [ ] Throughput meets targets
- [ ] Latency meets targets
- [ ] Bottlenecks identified and documented

---

#### Task 3.5: Documentation Completion

**Objective**: Complete all documentation following Diataxis framework

**Duration**: 2-3 days

**Documentation to Create/Update**:

**Tutorials** (`docs/tutorials/`):
- `quickstart.md` - Getting started with XZepr MCP
- `first_tool_call.md` - Making your first tool call

**How-To Guides** (`docs/how_to/`):
- `configure_oidc_authentication.md` - OIDC setup with Keycloak
- `configure_server.md` - Server configuration
- `deploy_production.md` - Production deployment
- `troubleshoot_errors.md` - Common errors and solutions
- `monitor_server.md` - Monitoring and observability

**Explanations** (`docs/explanation/`):
- `architecture.md` - Architecture overview (update)
- `security_controls.md` - Security architecture
- `session_security.md` - Session management
- `design_decisions.md` - Key design decisions
- `performance_characteristics.md` - Performance analysis

**Reference** (`docs/reference/`):
- `api.md` - Health/monitoring API reference
- `configuration.md` - Configuration reference
- `mcp_tools.md` - MCP tools reference
- `error_codes.md` - Error code reference
- `metrics.md` - Prometheus metrics reference

**README.md Updates**:
- Installation instructions
- Quick start guide
- Configuration examples
- Security notes
- Contributing guidelines

**Success Criteria**:
- [ ] All documentation follows Diataxis framework
- [ ] All filenames use lowercase_with_underscores.md
- [ ] No emojis in documentation
- [ ] Code examples tested and working
- [ ] README.md comprehensive
- [ ] OpenAPI spec documented

---

#### Phase 3 Deliverables

**Tests**:
- [ ] Unit tests (>80% coverage)
- [ ] Integration tests
- [ ] End-to-end tests
- [ ] Security tests (>95% attack detection)
- [ ] Performance benchmarks

**Documentation**:
- [ ] Complete Diataxis documentation set
- [ ] README.md updated
- [ ] Configuration examples
- [ ] Security guides
- [ ] Troubleshooting guides

**Performance Reports**:
- [ ] Baseline performance metrics
- [ ] Bottleneck analysis
- [ ] Resource usage profiles

---

#### Phase 3 Success Criteria

**Testing**:
- [ ] >80% code coverage achieved
- [ ] All integration tests pass
- [ ] All security tests pass (>95% attack detection)
- [ ] Performance baselines documented

**Documentation**:
- [ ] All documentation complete and accurate
- [ ] Code examples tested
- [ ] Diataxis framework followed
- [ ] Security guidance comprehensive

**Quality**:
- [ ] `cargo fmt --all` passes
- [ ] `cargo check --all-targets --all-features` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] All tests pass consistently

**Validation Gate**: All Phase 3 success criteria must pass before proceeding to Phase 4.

---

### Phase 4: Production Readiness & Deployment

**Duration**: 12-14 days (revised from 10-12 days)
**Priority**: High (Depends on Phase 3)

This phase prepares the server for production deployment with health checks, CLI, containers, and operational tooling.

#### Task 4.1a: OpenAPI Specification Generation (NEW)

**Objective**: Generate OpenAPI 3.0 specification for health and monitoring endpoints

**Duration**: 2 days

**Files to Create**:
- `xzepr-mcp/src/api/mod.rs` - API module root (expand stub)
- `xzepr-mcp/src/api/health.rs` - Health check endpoint implementation
- `xzepr-mcp/src/api/openapi.rs` - OpenAPI spec generation and UI setup

**Dependencies** (should already be in Cargo.toml):
- `utoipa` (4.0+) with axum feature
- `utoipa-swagger-ui` (6.0+) with axum feature

**Implementation Steps**:
1. Create health check endpoint at `/health` returning 200 with status JSON
2. Create readiness check endpoint at `/ready` (checks XZepr connectivity)
3. Annotate endpoints with `utoipa` macros for OpenAPI generation
4. Generate OpenAPI spec and expose at `/api/v1/openapi.json`
5. Mount Swagger UI at `/api/v1/docs`
6. Add version info from Cargo.toml to OpenAPI spec
7. Document all response codes and error formats
8. Add authentication requirements to OpenAPI spec

**Health Check Response Format**:

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "checks": {
    "xzepr_connectivity": "ok",
    "jwks_cache": "ok",
    "circuit_breaker": "closed"
  }
}
```

**Readiness Check Response**:

```json
{
  "ready": true,
  "checks": {
    "xzepr_api": "ok",
    "jwks
