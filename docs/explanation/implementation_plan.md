# XZepr MCP Production Implementation Plan

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
- No health check endpoints
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

**Module Structure Reference**: See `architecture.md` section "Module Structure"

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

**Testing Requirements**:

- Unit tests for configuration loading from each source
- Test precedence order (CLI overrides env overrides file)
- Test validation failures for missing required fields
- Test sensitive value redaction in debug output

**Success Criteria**:

- [ ] Configuration loads from YAML files with environment variable overrides
- [ ] CLI arguments properly override all other sources
- [ ] Sensitive values never appear in logs or debug output
- [ ] Validation errors provide clear messages for missing/invalid config
- [ ] All tests pass with >80% coverage

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

**Dependencies** (should already be in Cargo.toml from Task 1.1):

- `jsonwebtoken` (9.0+) - JWT validation
- `openidconnect` (3.0+) - OIDC discovery and JWKS
- `reqwest` (0.11+) with rustls-tls feature
- `tower` (0.4+) - Middleware framework
- `tower-http` (0.5+) - HTTP middleware utilities

**Implementation Steps**:

1. Implement `JwksCache` struct with TTL-based refresh from OIDC issuer
2. Create JWT validation function that verifies signature, issuer, audience, expiration
3. Extract scopes from token claims (support `scope` string and `scopes` array)
4. Implement scope checking: `has_scope()`, `requires_any_scope()`, `requires_all_scopes()`
5. Create authentication middleware that extracts Bearer token from Authorization header
6. Implement token fingerprinting (SHA-256 of token for audit logs)
7. Add automatic JWKS refresh on validation failure (kid not found)

**Security Requirements**:

- MUST validate JWT signature against JWKS
- MUST verify `iss` claim matches configured issuer
- MUST verify `aud` claim contains `xzepr-mcp`
- MUST check `exp` claim (token not expired)
- MUST enforce scope requirements per tool
- NO bypass option - validation always enabled in production
- Token fingerprints in audit logs (never full tokens)

**Testing Requirements**:

- Unit tests with mock JWKS endpoint
- Test valid token acceptance
- Test rejection of expired tokens
- Test rejection of invalid signatures
- Test rejection of wrong audience
- Test scope extraction from both claim formats
- Test JWKS cache refresh on TTL expiry
- Integration test with real Keycloak test instance

**Success Criteria**:

- [ ] JWT validation enforces all required claims
- [ ] JWKS cache refreshes automatically from issuer
- [ ] Invalid tokens are rejected with clear error messages
- [ ] Token fingerprints appear in structured logs
- [ ] Middleware integrates cleanly with rmcp transport
- [ ] All security tests pass with >90% coverage

#### Task 1.3: Input Validation & Injection Detection (MANDATORY)

**Objective**: Implement multi-layered input validation to prevent injection attacks and data corruption

**Duration**: 4-5 days (revised from 3-4 days)

**Files to Create**:

- `xzepr-mcp/src/validation/mod.rs` - Validation module root (expand stub)
- `xzepr-mcp/src/validation/schema.rs` - JSON schema validation
- `xzepr-mcp/src/validation/sanitize.rs` - Input sanitization functions
- `xzepr-mcp/src/validation/injection.rs` - Injection pattern detection
- `xzepr-mcp/src/validation/limits.rs` - Size and rate limit enforcement
- `xzepr-mcp/src/validation/ulid.rs` - ULID validation
- `xzepr-mcp/src/validation/semver.rs` - Semantic version validation
- `xzepr-mcp/src/validation/json.rs` - Recursive JSON validation
- `xzepr-mcp/src/validation/payload.rs` - Payload size validation

**Dependencies** (should already be in Cargo.toml from Task 1.1):

- `validator` (0.16+) - Validation framework with derive macros
- `regex` (1.10+) - Pattern matching for injection detection
- `ulid` (1.1+) - ULID validation and generation
- `semver` (1.0+) - Semantic version validation

**Implementation Steps**:

1. Define injection detection patterns (prompt injection, SQL, XSS, command injection)
2. Implement `detect_prompt_injection()` with pattern matching
3. Implement `sanitize_input()` for string fields (trim, normalize, escape)
4. Add recursive JSON validation for payload fields
5. Implement ULID format validation
6. Implement semantic version validation
7. Add size limits (max string length, max JSON depth, max array size)
8. Create validation middleware that runs before tool handlers
9. Add payload size validation (64KB max for JSON payloads)
10. Implement recursive JSON validation with depth limit (max 10 levels)
11. Add query parameter sanitization for search operations
12. Implement semantic version validation using `semver` crate
13. Add content-type validation

**Security Requirements**:

- MUST detect and reject prompt injection patterns
- MUST validate ULID format for all ID fields
- MUST validate semantic versions for version fields
- MUST enforce size limits on all inputs
- MUST recursively validate nested JSON in payloads
- MUST sanitize string inputs before passing to XZepr API
- Log all validation failures with sanitized input samples

**Validation Rules**:

- ULID: 26 characters, Crockford Base32 alphabet only
- Semantic Version: Valid semver format (e.g., "1.2.3", "1.0.0-alpha")
- JSON payload: Max 64KB size, max 10 levels depth
- Text inputs: Max 10,000 characters after sanitization
- Query parameters: Max 255 characters, alphanumeric + limited special chars
- Event names: Max 255 chars, no control characters
- Descriptions: Max 2,000 chars, sanitized

**Testing Requirements**:

- Unit tests for each injection pattern
- Test ULID validation with valid/invalid inputs
- Test semver validation with valid and invalid versions
- Test size limit enforcement
- Test recursive JSON validation
- Test sanitization preserves valid data
- Attack vector tests with known injection payloads
- Test payload size rejection (>64KB)
- Test recursive JSON depth limit enforcement
- Test query parameter injection attempts
- Test malformed semver version strings

**Success Criteria**:

- [ ] All injection patterns detected and rejected
- [ ] ULID and semver validation working correctly
- [ ] Size limits prevent resource exhaustion
- [ ] Sanitization preserves valid data
- [ ] Validation errors provide actionable feedback
- [ ] All security tests pass with >90% coverage

#### Task 1.4: XZepr HTTP Client with Resilience

**Objective**: Implement robust HTTP client for XZepr API with retry, circuit breaker, and timeout handling

**Files to Create**:

- `xzepr-mcp/src/client/mod.rs` - Client module root
- `xzepr-mcp/src/client/xzepr.rs` - XZepr HTTP client implementation
- `xzepr-mcp/src/client/retry.rs` - Retry policy with exponential backoff
- `xzepr-mcp/src/client/circuit_breaker.rs` - Circuit breaker implementation
- `xzepr-mcp/src/client/auth.rs` - Client-side authentication token injection

**Dependencies to Add**:

- `reqwest` (0.11+) - HTTP client
- `reqwest-middleware` (0.2+) - Request middleware
- `reqwest-retry` (0.3+) - Retry middleware
- `tower` (0.4+) - Service abstractions

**Implementation Steps**:

1. Create `XZeprClient` struct with base URL and auth config
2. Implement retry policy: max 3 attempts, exponential backoff (100ms, 200ms, 400ms)
3. Implement circuit breaker: open after 5 consecutive failures, half-open after 30s
4. Add authenticated request wrapper (injects Bearer token or API key)
5. Implement methods: `fetch_event()`, `create_event()`, `search_events()`, etc.
6. Add request/response logging with structured fields
7. Implement timeout handling (default 30s, configurable per request)

**Testing Requirements**:

- Unit tests with mock HTTP server
- Test retry on transient failures (503, network timeout)
- Test circuit breaker opens after failure threshold
- Test circuit breaker closes after success in half-open
- Test authentication token injection
- Test timeout enforcement
- Integration tests against real XZepr test instance

**Success Criteria**:

- [ ] Client handles transient failures gracefully with retries
- [ ] Circuit breaker prevents cascading failures
- [ ] All XZepr API operations supported
- [ ] Timeouts prevent hung requests
- [ ] Structured logging includes request/response metadata
- [ ] All tests pass with >80% coverage

#### Task 1.5: Error Handling Framework

**Objective**: Implement comprehensive error types with proper context and propagation

**Files to Create**:

- `xzepr-mcp/src/error.rs` - Error type definitions
- `xzepr-mcp/src/error/mcp.rs` - MCP-specific errors
- `xzepr-mcp/src/error/xzepr.rs` - XZepr API errors

**Dependencies to Add**:

- `thiserror` (1.0+) - Error derive macros
- `anyhow` (1.0+) - Error context for application code

**Implementation Steps**:

1. Define `McpError` enum with variants: `XZeprApi`, `Config`, `Validation`, `Protocol`, `Serialization`, `Auth`, `RateLimit`
2. Define `XZeprError` enum with variants: `Connection`, `Timeout`, `Http`, `Auth`, `NotFound`, `InvalidResponse`
3. Implement `From` conversions between error types
4. Add context methods for error chain building
5. Implement error-to-MCP-response mapping
6. Add structured error logging

**Testing Requirements**:

- Unit tests for error construction
- Test error conversion chains
- Test error message formatting
- Test context preservation through propagation

**Success Criteria**:

- [ ] All error paths return descriptive errors
- [ ] Error context preserved through call stack
- [ ] MCP clients receive actionable error messages
- [ ] Errors logged with appropriate severity levels
- [ ] All tests pass with >80% coverage

#### Task 1.6: Rate Limiting Implementation (MANDATORY)

**Objective**: Implement per-user and per-tool rate limiting to prevent abuse

**Duration**: 2-3 days

**Files to Create**:

- `xzepr-mcp/src/mcp/rate_limit.rs` - Rate limiting middleware
- `xzepr-mcp/src/mcp/rate_limit_config.rs` - Rate limit configuration

**Dependencies** (should already be in Cargo.toml from Task 1.1):

- `governor` (0.6+) - Rate limiting library

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
   - X-RateLimit-Limit: limit value (requests per minute)
   - X-RateLimit-Remaining: remaining requests in current window
   - X-RateLimit-Reset: unix timestamp of window reset
   - Retry-After: seconds to wait (on 429 only)
6. Implement rate limit metrics for monitoring
7. Add rate limit event logging

**Security Requirements**:

- MUST enforce global rate limit to prevent resource exhaustion
- MUST enforce per-user limits based on JWT `sub` claim
- MUST enforce per-tool limits for expensive operations
- MUST log all rate limit violations for security monitoring
- Rate limit state MUST be memory-safe (no DoS via state explosion)

**Testing Requirements**:

- Unit tests for rate limit calculation
- Test global limit enforcement
- Test per-user limit enforcement
- Test per-tool limit enforcement
- Test rate limit header presence
- Load tests to verify limits under concurrency
- Test user-specific rate limiting (different users, independent limits)
- Test per-tool rate limit differentiation
- Test rate limit header presence and accuracy
- Test burst allowance behavior
- Test 429 response format with Retry-After header

**Success Criteria**:

- [ ] Global rate limit prevents resource exhaustion
- [ ] Per-user limits enforce fair usage
- [ ] Per-tool limits protect expensive operations
- [ ] Rate limit headers guide client retry behavior
- [ ] All violations logged for security analysis
- [ ] All tests pass with >85% coverage

#### Task 1.7: Observability Infrastructure

**Objective**: Implement structured logging, metrics, and tracing for production monitoring

**Duration**: 2-3 days

**Files to Create**:

- `xzepr-mcp/src/observability/mod.rs` - Observability module
- `xzepr-mcp/src/observability/logging.rs` - Structured logging setup
- `xzepr-mcp/src/observability/metrics.rs` - Prometheus metrics
- `xzepr-mcp/src/observability/tracing.rs` - OpenTelemetry tracing

**Dependencies** (should already be in Cargo.toml from Task 1.1):

- `tracing` (0.1+) - Structured logging and tracing
- `tracing-subscriber` (0.3+) with json, env-filter features
- `tracing-opentelemetry` (0.22+) - OpenTelemetry integration
- `prometheus` (0.13+) - Metrics collection

**Implementation Steps**:

1. Configure structured JSON logging with tracing-subscriber
2. Define Prometheus metrics: request counts, latencies, error rates, active connections
3. Implement OpenTelemetry tracing with automatic span creation
4. Add request correlation IDs
5. Implement `/metrics` endpoint for Prometheus scraping
6. Add log sampling for high-volume operations
7. Configure log levels per module

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

**Metrics to Expose**:

- `xzepr_mcp_requests_total` (counter) - Labels: tool, status, user
- `xzepr_mcp_request_duration_seconds` (histogram) - Labels: tool
- `xzepr_mcp_errors_total` (counter) - Labels: error_type, tool
- `xzepr_mcp_active_connections` (gauge)
- `xzepr_mcp_rate_limit_rejections_total` (counter) - Labels: user, tool
- `xzepr_mcp_auth_failures_total` (counter) - Labels: reason

**Testing Requirements**:

- Unit tests for metrics registration
- Test log output format (JSON structure)
- Test metrics incremented correctly
- Test trace span creation and propagation
- Integration test with OTLP collector

**Success Criteria**:

- [ ] All requests logged in structured JSON format
- [ ] Metrics exposed on `/metrics` endpoint
- [ ] Traces exported to OTLP collector (when configured)
- [ ] Correlation IDs link logs, metrics, and traces
- [ ] Performance overhead <5% with tracing enabled
- [ ] All tests pass with >80% coverage

#### Phase 1 Deliverables

**Code Artifacts**:

- Complete configuration system with security defaults
- OIDC/JWT authentication with JWKS caching
- Input validation and injection detection
- XZepr HTTP client with resilience patterns
- Error handling framework
- Rate limiting middleware
- Observability infrastructure

**Documentation**:

- Configuration guide (`docs/how_to/configure_server.md`)
- OIDC setup guide (`docs/how_to/configure_oidc_authentication.md`)
- Security guidelines (`docs/explanation/security_controls.md`)
- API error reference (`docs/reference/error_codes.md`)

**Testing**:

- Unit test coverage >80% for all modules
- Integration tests for auth and XZepr client
- Security tests for injection detection
- Load tests for rate limiting

#### Phase 1 Success Criteria

**Functional Requirements**:

- [ ] Configuration loads from all sources with proper precedence
- [ ] JWT validation enforces all security requirements
- [ ] Input validation blocks all known injection patterns
- [ ] XZepr client successfully communicates with backend
- [ ] Rate limits enforce resource protection
- [ ] Metrics and logs capture all security events

**Security Requirements**:

- [ ] No authentication bypass possible
- [ ] All tokens validated against JWKS
- [ ] All inputs validated and sanitized
- [ ] Rate limits prevent abuse
- [ ] Security events audited in logs

**Quality Requirements**:

- [ ] All tests pass with >80% coverage
- [ ] Zero clippy warnings with `-D warnings`
- [ ] All code formatted with `cargo fmt`
- [ ] Documentation complete for all public APIs

### Phase 2: MCP Protocol Implementation

**Duration**: 12-15 days (revised from 10-12 days)
**Priority**: High (Depends on Phase 1)

This phase implements the MCP protocol server and tool handlers, integrating with the security infrastructure built in Phase 1.

#### Task 2.1: MCP Server Setup with StreamableHTTP Transport

**Objective**: Initialize MCP server using rmcp with StreamableHTTP transport and session management

**Files to Create**:

- `xzepr-mcp/src/mcp/mod.rs` - MCP module root
- `xzepr-mcp/src/mcp/server.rs` - Server setup and lifecycle
- `xzepr-mcp/src/mcp/transport.rs` - StreamableHTTP transport configuration
- `xzepr-mcp/src/mcp/session.rs` - Session management with security

**Dependencies to Add**:

- `rmcp` (latest) - Rust MCP SDK
- `axum` (0.7+) - HTTP framework for transport
- `tokio` (1.35+) with full feature
- `uuid` (1.6+) - Session ID generation

**Implementation Steps**:

1. Configure `StreamableHttpService` with `LocalSessionManager`
2. Implement secure session ID generation (cryptographically random)
3. Add session binding to JWT `sub` claim (prevent token substitution)
4. Implement session expiration (30 minute idle timeout)
5. Add session rotation on authentication events
6. Wire up authentication middleware from Phase 1
7. Wire up rate limiting middleware from Phase 1
8. Add health check endpoint at `/health`

**Security Requirements**:

- MUST generate cryptographically secure session IDs
- MUST bind sessions to authenticated user identity
- MUST expire sessions after idle timeout
- MUST rotate session IDs on re-authentication
- MUST reject requests with invalid session IDs

**Testing Requirements**:

- Unit tests for session generation and validation
- Test session binding to JWT claims
- Test session expiration enforcement
- Test session rotation
- Integration test with rmcp client

**Success Criteria**:

- [ ] MCP server starts and accepts StreamableHTTP connections
- [ ] Sessions properly bound to user identities
- [ ] Session expiration enforced correctly
- [ ] Health check endpoint returns 200 with valid status
- [ ] All tests pass with >80% coverage

#### Task 2.2: MCP Tool Definitions with Security Metadata

**Objective**: Define all MCP tools with input schemas, security requirements, and resource descriptions

**Files to Create**:

- `xzepr-mcp/src/mcp/tools.rs` - Tool definitions and schemas
- `xzepr-mcp/src/mcp/schemas.rs` - JSON schema definitions
- `xzepr-mcp/src/mcp/permissions.rs` - Per-tool permission mappings

**Implementation Steps**:

1. Define tool schemas for all 9 operations: `fetch_event`, `create_event`, `search_events`, `fetch_receiver`, `create_receiver`, `search_receivers`, `fetch_group`, `create_group`, `search_groups`
2. Map each tool to required scopes: read tools require `xzepr:read`, write tools require `xzepr:write`
3. Add resource descriptions for MCP resources protocol
4. Define input validation rules per tool
5. Add tool metadata: rate limit tier, estimated latency, required permissions

**Tool-to-Scope Mapping**:

- `fetch_event` → `xzepr:read`
- `create_event` → `xzepr:write`
- `search_events` → `xzepr:read`
- `fetch_receiver` → `xzepr:read`
- `create_receiver` → `xzepr:write`
- `search_receivers` → `xzepr:read`
- `fetch_group` → `xzepr:read`
- `create_group` → `xzepr:write`
- `search_groups` → `xzepr:read`

**Testing Requirements**:

- Unit tests for schema validation
- Test tool registration with MCP server
- Test permission mapping lookup
- Validate JSON schema definitions against MCP spec

**Success Criteria**:

- [ ] All 9 tools registered with correct schemas
- [ ] Permission mappings enforce read/write separation
- [ ] Tool metadata accurate and complete
- [ ] Schemas validate correctly against MCP specification
- [ ] All tests pass with >80% coverage

#### Task 2.3: Tool Handler Implementation with Security Integration

**Objective**: Implement tool request handlers that enforce authentication, validation, and rate limiting

**Files to Create**:

- `xzepr-mcp/src/mcp/handlers/mod.rs` - Handler module root
- `xzepr-mcp/src/mcp/handlers/events.rs` - Event tool handlers
- `xzepr-mcp/src/mcp/handlers/receivers.rs` - Receiver tool handlers
- `xzepr-mcp/src/mcp/handlers/groups.rs` - Group tool handlers
- `xzepr-mcp/src/mcp/handlers/common.rs` - Shared handler utilities

**Implementation Steps**:

1. Implement handler function for each tool that: validates JWT scopes, validates input, calls XZepr client, formats response, logs audit event
2. Add automatic audit logging to all handlers (user, tool, input fingerprint, result, duration)
3. Implement error mapping from XZepr errors to MCP errors
4. Add request tracing spans to all handlers
5. Implement idempotency keys for write operations
6. Add response size limits (prevent memory exhaustion)

**Security Requirements**:

- MUST validate JWT scopes before executing tool
- MUST validate all inputs before calling XZepr API
- MUST sanitize inputs before passing to XZepr
- MUST audit log all tool invocations with outcomes
- MUST enforce rate limits via middleware
- MUST handle errors securely (no sensitive data leakage)

**Testing Requirements**:

- Unit tests for each handler with mock XZepr client
- Test scope enforcement (reject insufficient permissions)
- Test input validation integration
- Test error handling and response formatting
- Test audit log generation
- Integration tests with real XZepr API

**Success Criteria**:

- [ ] All 9 tool handlers implemented and working
- [ ] Scope enforcement prevents unauthorized operations
- [ ] Input validation blocks invalid requests
- [ ] Audit logs capture all security-relevant events
- [ ] Errors provide actionable feedback without leaking sensitive data
- [ ] All tests pass with >85% coverage

#### Task 2.4: Response Formatting & Error Handling

**Objective**: Implement consistent response formatting and error handling for all MCP tools

**Files to Create**:

- `xzepr-mcp/src/mcp/responses.rs` - Response formatting utilities
- `xzepr-mcp/src/mcp/errors.rs` - MCP error response builders

**Implementation Steps**:

1. Implement response formatters for each XZepr entity type
2. Add timestamp formatting (RFC-3339)
3. Implement pagination response wrappers for search results
4. Create error response builder with proper MCP error structure
5. Add correlation ID to all responses
6. Implement response sanitization (remove internal fields)

**Testing Requirements**:

- Unit tests for response formatting
- Test timestamp conversion to RFC-3339
- Test pagination metadata accuracy
- Test error response structure matches MCP spec
- Test response sanitization removes internal fields

**Success Criteria**:

- [ ] All responses formatted consistently
- [ ] Timestamps in RFC-3339 format
- [ ] Pagination metadata accurate
- [ ] Error responses match MCP specification
- [ ] No internal implementation details leak in responses
- [ ] All tests pass with >80% coverage

#### Phase 2 Deliverables

**Code Artifacts**:

- Complete MCP server with StreamableHTTP transport
- Secure session management implementation
- All 9 tool handlers with security integration
- Response formatting and error handling

**Documentation**:

- Tool usage guide (`docs/how_to/use_mcp_tools.md`)
- MCP client integration guide (`docs/how_to/integrate_mcp_clients.md`)
- Tool reference (`docs/reference/tool_reference.md`)
- Error handling guide (`docs/how_to/handle_errors.md`)

**Testing**:

- Unit test coverage >85% for handlers
- Integration tests with rmcp client
- Security tests for scope enforcement
- End-to-end tests with Claude Desktop

#### Phase 2 Success Criteria

**Functional Requirements**:

- [ ] MCP server accepts client connections via StreamableHTTP
- [ ] All 9 tools respond correctly to valid requests
- [ ] Sessions managed securely with binding and expiration
- [ ] Responses formatted correctly per MCP specification

**Security Requirements**:

- [ ] Scope enforcement prevents unauthorized tool access
- [ ] Input validation integrated with all handlers
- [ ] Audit logs capture all tool invocations
- [ ] Rate limits enforced per tool and user

**Quality Requirements**:

- [ ] All tests pass with >85% coverage
- [ ] Zero clippy warnings
- [ ] All code formatted and documented
- [ ] Integration tests pass with MCP clients

### Phase 3: Testing, Documentation & Security Validation

**Duration**: 8-10 days
**Priority**: High (Depends on Phase 2)

This phase ensures production quality through comprehensive testing, documentation, and security validation.

#### Task 3.1: Comprehensive Unit Testing

**Objective**: Achieve >85% code coverage with comprehensive unit tests for all modules

**Testing Focus Areas**:

1. Configuration loading and validation
2. JWT validation with various token conditions
3. Input validation and injection detection
4. XZepr client with mock HTTP responses
5. Rate limiting edge cases
6. Handler logic with mocked dependencies
7. Error handling and propagation
8. Session management

**Implementation Steps**:

1. Add unit tests to all modules created in Phase 1 and 2
2. Use test fixtures for common test data
3. Implement mock XZepr API server for testing
4. Add test helpers for JWT generation
5. Create test cases for all error paths
6. Add property-based tests for input validation
7. Measure coverage with `cargo tarpaulin`

**Success Criteria**:

- [ ] Overall code coverage >85%
- [ ] All modules have >80% coverage
- [ ] All error paths tested
- [ ] All security controls tested
- [ ] Tests run in <30 seconds

#### Task 3.2: Integration & End-to-End Testing

**Objective**: Validate system behavior with real integrations and end-to-end workflows

**Files to Create**:

- `xzepr-mcp/tests/integration_auth.rs` - Authentication integration tests
- `xzepr-mcp/tests/integration_tools.rs` - Tool execution integration tests
- `xzepr-mcp/tests/integration_security.rs` - Security controls integration tests
- `xzepr-mcp/tests/e2e_workflows.rs` - End-to-end user workflows

**Test Scenarios**:

1. Complete authentication flow with real Keycloak
2. All 9 tools with real XZepr API
3. Rate limiting under concurrent load
4. Session lifecycle (creation, refresh, expiration)
5. Error recovery and retry behavior
6. Complete workflows: create receiver → create event → search events

**Dependencies**:

- Test Keycloak instance (Docker)
- Test XZepr instance (Docker)
- rmcp test client

**Success Criteria**:

- [ ] All integration tests pass against test instances
- [ ] End-to-end workflows complete successfully
- [ ] Security controls function correctly in integration
- [ ] Performance acceptable under test load

#### Task 3.3: Security Testing & Attack Simulation

**Objective**: Validate security controls against known attack vectors

**Files to Create**:

- `xzepr-mcp/tests/security_injection.rs` - Injection attack tests
- `xzepr-mcp/tests/security_auth.rs` - Authentication bypass tests
- `xzepr-mcp/tests/security_abuse.rs` - Abuse and DoS tests

**Attack Vectors to Test**:

1. Prompt injection attempts in all text fields
2. JWT tampering (signature, claims, expiry)
3. Token replay attacks
4. Session hijacking attempts
5. Rate limit bypass attempts
6. Input size attacks (memory exhaustion)
7. Invalid ULID/version format attacks

**Implementation Steps**:

1. Create attack payload library with known injection patterns
2. Implement negative test cases for each attack vector
3. Verify all attacks are blocked and logged
4. Test error messages don't leak sensitive information
5. Verify audit logs capture all attack attempts

**Success Criteria**:

- [ ] All injection attacks detected and blocked
- [ ] No authentication bypass possible
- [ ] Rate limits cannot be circumvented
- [ ] All attacks logged with security context
- [ ] Error messages safe from information leakage

#### Task 3.4: Performance & Load Testing

**Objective**: Validate performance targets and identify bottlenecks

**Files to Create**:

- `xzepr-mcp/tests/performance_baseline.rs` - Baseline performance tests
- `xzepr-mcp/benches/tool_handlers.rs` - Handler micro-benchmarks

**Dependencies to Add**:

- `criterion` (0.5+) - Benchmarking framework

**Performance Targets** (from architecture doc):

- Baseline throughput: 100 req/s sustained
- P99 latency: <200ms for read operations
- P99 latency: <500ms for write operations
- Memory usage: <200MB under normal load
- CPU usage: <50% under normal load

**Test Scenarios**:

1. Sustained load at 100 req/s for 10 minutes
2. Burst traffic: 200 req/s for 1 minute
3. Read-heavy workload (90% reads, 10% writes)
4. Write-heavy workload (50% reads, 50% writes)
5. Concurrent users: 50 simultaneous connections

**Metrics to Collect**:

- Request latency (P50, P95, P99)
- Throughput (req/s)
- Memory usage (RSS)
- CPU usage (%)
- Error rate
- Connection count

**Success Criteria**:

- [ ] Baseline throughput target met (100 req/s)
- [ ] Latency targets met (P99 <200ms reads, <500ms writes)
- [ ] Resource usage within limits (<200MB, <50% CPU)
- [ ] No memory leaks under sustained load
- [ ] Error rate <0.1% under normal load

#### Task 3.5: Documentation Completion

**Objective**: Create comprehensive documentation for all audiences

**Documents to Create**:

**Tutorials** (`docs/tutorials/`):

- `getting_started.md` - Quick start guide
- `first_integration.md` - First MCP client integration

**How-To Guides** (`docs/how_to/`):

- `configure_server.md` - Server configuration
- `configure_oidc_authentication.md` - OIDC setup
- `use_mcp_tools.md` - Tool usage examples
- `integrate_mcp_clients.md` - Client integration
- `monitor_production.md` - Production monitoring
- `troubleshoot_issues.md` - Troubleshooting guide
- `handle_errors.md` - Error handling

**Reference** (`docs/reference/`):

- `tool_reference.md` - Complete tool documentation
- `configuration_reference.md` - All config options
- `error_codes.md` - Error code reference
- `api_specification.md` - OpenAPI spec reference
- `metrics_reference.md` - Prometheus metrics

**Explanation** (`docs/explanation/`):

- `security_controls.md` - Security architecture
- `design_decisions.md` - Key design decisions
- `architecture_overview.md` - System architecture
- `performance_characteristics.md` - Performance profile

**Success Criteria**:

- [ ] All documents follow Diataxis framework
- [ ] Code examples tested and working
- [ ] All configuration options documented
- [ ] All error codes documented
- [ ] Security guidance complete

#### Phase 3 Deliverables

**Testing Artifacts**:

- Complete unit test suite (>85% coverage)
- Integration test suite
- Security test suite with attack simulations
- Performance test suite with benchmarks
- Test documentation and procedures

**Documentation Artifacts**:

- Complete documentation set following Diataxis
- OpenAPI specification
- Security runbooks
- Troubleshooting guides
- Architecture diagrams

**Quality Metrics**:

- Test coverage report
- Performance benchmark results
- Security test results
- Documentation completeness checklist

#### Phase 3 Success Criteria

**Testing Requirements**:

- [ ] Unit test coverage >85%
- [ ] All integration tests pass
- [ ] All security tests pass (attack vectors blocked)
- [ ] Performance targets met

**Documentation Requirements**:

- [ ] All documentation complete per Diataxis framework
- [ ] All code examples tested and working
- [ ] All configuration options documented
- [ ] Security runbooks complete

**Quality Requirements**:

- [ ] Zero clippy warnings
- [ ] All public APIs documented
- [ ] No known security vulnerabilities
- [ ] Performance baselines established

### Phase 4: Production Readiness & Deployment

**Duration**: 12-14 days (revised from 10-12 days)
**Priority**: High (Depends on Phase 3)

This phase prepares the system for production deployment with containers, CLI, health checks, and operational tooling.

#### Task 4.1a: OpenAPI Specification Generation

**Objective**: Generate OpenAPI 3.0 specification for health and monitoring endpoints

**Duration**: 2 days

**Files to Create**:

- `xzepr-mcp/src/api/mod.rs` - API module root (expand stub)
- `xzepr-mcp/src/api/health.rs` - Health check endpoint implementation
- `xzepr-mcp/src/api/openapi.rs` - OpenAPI spec generation and UI setup

**Dependencies** (should already be in Cargo.toml from Task 1.1):

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

**Health Check Response Format**:

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "checks": {
    "xzepr_connectivity": "ok",
    "jwks_cache": "ok"
  }
}
```

**Testing Requirements**:

- Unit tests for health check logic
- Integration test for OpenAPI spec generation
- Validate OpenAPI spec against 3.0 schema
- Test Swagger UI renders correctly
- Test health endpoint returns 200
- Test readiness endpoint detects XZepr unavailability

**Success Criteria**:

- [ ] Health endpoint accessible at `/health`
- [ ] Readiness endpoint accessible at `/ready`
- [ ] OpenAPI spec accessible at `/api/v1/openapi.json`
- [ ] Swagger UI accessible at `/api/v1/docs`
- [ ] OpenAPI spec validates against 3.0 schema
- [ ] All endpoints documented in spec
- [ ] All tests pass with >80% coverage

---

#### Task 4.1b: Health Check & Monitoring Endpoints

**Objective**: Implement comprehensive health checks and monitoring endpoints for orchestration

**Duration**: 2-3 days

**Files to Create**:

- `xzepr-mcp/src/api/mod.rs` - API module for HTTP endpoints
- `xzepr-mcp/src/api/health.rs` - Health check endpoint
- `xzepr-mcp/src/api/metrics.rs` - Metrics endpoint
- `xzepr-mcp/src/api/openapi.rs` - OpenAPI specification

**Dependencies to Add**:

- `utoipa` (4.0+) - OpenAPI documentation
- `utoipa-swagger-ui` (5.0+) - Swagger UI

**Implementation Steps**:

1. Implement `/health` endpoint: returns 200 with component status
2. Implement `/health/live` endpoint: liveness probe (process alive)
3. Implement `/health/ready` endpoint: readiness probe (can accept traffic)
4. Check health of: OIDC JWKS reachability, XZepr API connectivity, session store
5. Implement `/metrics` endpoint: Prometheus metrics from Phase 1
6. Generate OpenAPI spec with utoipa macros
7. Expose OpenAPI spec at `/api/v1/openapi.json`
8. Add Swagger UI at `/api/v1/docs`

**Health Check Response Format**:

```
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "components": {
    "jwks_cache": "healthy",
    "xzepr_api": "healthy",
    "session_store": "healthy"
  }
}
```

**Success Criteria**:

- [ ] Health endpoints return correct status
- [ ] Liveness probe detects process health
- [ ] Readiness probe detects service readiness
- [ ] OpenAPI spec generated and exposed
- [ ] Swagger UI accessible and functional

#### Task 4.2: CLI Implementation

**Objective**: Create command-line interface for server management and testing

**Files to Create**:

- `xzepr-mcp/src/cli/mod.rs` - CLI module
- `xzepr-mcp/src/cli/commands.rs` - Command definitions
- `xzepr-mcp/src/cli/serve.rs` - Server command
- `xzepr-mcp/src/cli/validate.rs` - Configuration validation command
- `xzepr-mcp/src/cli/test_auth.rs` - Authentication test command

**CLI Commands**:

- `xzepr-mcp serve` - Start MCP server
- `xzepr-mcp validate-config` - Validate configuration file
- `xzepr-mcp test-auth` - Test OIDC authentication
- `xzepr-mcp version` - Show version information
- `xzepr-mcp health-check` - Check health of running server

**Implementation Steps**:

1. Implement CLI structure with clap subcommands
2. Add `serve` command with graceful shutdown handling
3. Add `validate-config` command that loads and validates config
4. Add `test-auth` command that validates JWT against OIDC
5. Add version command with build metadata
6. Add health-check command that calls `/health` endpoint
7. Add JSON output option for automation

**Success Criteria**:

- [ ] All CLI commands implemented and working
- [ ] Server starts and stops gracefully
- [ ] Config validation provides clear error messages
- [ ] Auth testing validates real tokens
- [ ] CLI output supports both human and JSON formats

#### Task 4.3: Docker & Container Deployment

**Objective**: Create production-ready container images and deployment configurations

**Files to Create**:

- `xzepr-mcp/Dockerfile` - Multi-stage production build
- `xzepr-mcp/.dockerignore` - Docker ignore rules
- `xzepr-mcp/docker-compose.yaml` - Local development stack
- `xzepr-mcp/deploy/kubernetes/deployment.yaml` - Kubernetes deployment
- `xzepr-mcp/deploy/kubernetes/service.yaml` - Kubernetes service
- `xzepr-mcp/deploy/kubernetes/configmap.yaml` - Configuration
- `xzepr-mcp/deploy/kubernetes/secret.yaml.template` - Secret template

**Dockerfile Requirements**:

1. Multi-stage build: builder stage + runtime stage
2. Use distroless or Alpine base for minimal attack surface
3. Run as non-root user
4. Set proper security labels
5. Health check instruction
6. Expose port 8080

**Docker Compose Stack**:

- xzepr-mcp server
- XZepr backend (for testing)
- Keycloak (for testing)
- Prometheus (for monitoring)
- Grafana (for visualization)

**Kubernetes Manifests**:

1. Deployment with 2 replicas
2. Service (ClusterIP)
3. ConfigMap for configuration
4. Secret for sensitive values
5. HorizontalPodAutoscaler (scale 2-10 based on CPU)
6. PodDisruptionBudget (minAvailable: 1)
7. NetworkPolicy (restrict ingress/egress)

**Success Criteria**:

- [ ] Docker image builds successfully
- [ ] Image size <50MB (compressed)
- [ ] Container runs as non-root
- [ ] Docker Compose stack starts all services
- [ ] Kubernetes manifests deploy successfully
- [ ] Health checks work in Kubernetes

#### Task 4.4: Security Hardening & Production Configuration

**Objective**: Apply final security hardening and create production configuration templates

**Files to Create**:

- `xzepr-mcp/config/production.yaml` - Production config template
- `xzepr-mcp/deploy/security/seccomp-profile.json` - Seccomp profile
- `xzepr-mcp/deploy/security/apparmor-profile` - AppArmor profile
- `xzepr-mcp/SECURITY.md` - Security policy and reporting

**Security Hardening Steps**:

1. Create seccomp profile limiting system calls
2. Create AppArmor/SELinux profile
3. Configure TLS for all external connections
4. Set strict file permissions in container
5. Disable unnecessary features in production
6. Enable all security headers in HTTP responses
7. Configure audit logging to external SIEM
8. Set up automated vulnerability scanning

**Production Configuration**:

1. TLS required for OIDC and XZepr connections
2. Strict rate limits (lower than development)
3. Audit logging to external system
4. Metrics retention policy
5. Session timeout: 30 minutes
6. JWKS cache TTL: 1 hour
7. Request timeout: 30 seconds
8. Circuit breaker thresholds

**Success Criteria**:

- [ ] Seccomp profile limits system calls
- [ ] Container passes security scan (no HIGH/CRITICAL vulnerabilities)
- [ ] TLS enforced for all external connections
- [ ] Production configuration validated
- [ ] Security headers present in all responses
- [ ] Audit logs export to SIEM

#### Task 4.5: Operational Runbooks & Incident Response

**Objective**: Create operational procedures and incident response playbooks

**Documents to Create**:

- `docs/operations/deployment_checklist.md` - Pre-deployment checklist
- `docs/operations/monitoring_guide.md` - Monitoring and alerting
- `docs/operations/incident_response.md` - Incident response procedures
- `docs/operations/backup_recovery.md` - Backup and recovery
- `docs/operations/scaling_guide.md` - Scaling procedures

**Runbook Sections**:

1. Deployment procedures (rolling update, rollback)
2. Monitoring dashboards and alerts
3. Common issues and resolutions
4. Performance tuning guidelines
5. Security incident response
6. Disaster recovery procedures

**Alerting Rules** (Prometheus):

- High error rate (>1% for 5 minutes)
- High latency (P99 >500ms for 5 minutes)
- Authentication failures spike (>10/min)
- Rate limit violations spike (>50/min)
- Circuit breaker open
- JWKS refresh failures
- Memory usage >80%

**Success Criteria**:

- [ ] Deployment checklist complete
- [ ] Monitoring dashboards created
- [ ] Alert rules configured
- [ ] Incident response procedures documented
- [ ] Runbooks tested in staging

#### Phase 4 Deliverables

**Deployment Artifacts**:

- Production Docker image
- Docker Compose stack for local development
- Kubernetes manifests for production
- Security profiles (seccomp, AppArmor)
- Production configuration templates

**Operational Tools**:

- CLI for server management
- Health check endpoints
- Monitoring dashboards
- Alert rules
- Operational runbooks

**Documentation**:

- Deployment guide
- Operations manual
- Incident response procedures
- Security policy
- Scaling guide

#### Phase 4 Success Criteria

**Deployment Requirements**:

- [ ] Docker image builds and runs securely
- [ ] Kubernetes deployment successful
- [ ] Health checks integrated with orchestration
- [ ] TLS enforced for external connections
- [ ] Security profiles applied

**Operational Requirements**:

- [ ] CLI provides management capabilities
- [ ] Monitoring dashboards functional
- [ ] Alerts configured and tested
- [ ] Runbooks complete and tested
- [ ] Incident response procedures documented

**Security Requirements**:

- [ ] Container passes security scan
- [ ] All connections use TLS
- [ ] Audit logs export to SIEM
- [ ] Security headers enforced
- [ ] Vulnerability scanning automated

## Risk Assessment & Mitigation

### High-Risk Items

**Risk: OIDC Integration Complexity**

- **Impact**: Authentication failure blocks all functionality
- **Likelihood**: Medium
- **Mitigation**: Implement OIDC integration early (Phase 1), extensive testing with real Keycloak, fallback to local testing with mock JWKS server

**Risk: Performance Targets Not Met**

- **Impact**: System unusable under production load
- **Likelihood**: Low-Medium
- **Mitigation**: Performance testing in Phase 3, early profiling, circuit breaker prevents cascading failures, horizontal scaling with Kubernetes

**Risk: Security Vulnerabilities Discovered Late**

- **Impact**: Delays production deployment, requires extensive rework
- **Likelihood**: Low (with security-first approach)
- **Mitigation**: Security testing throughout all phases, attack simulation in Phase 3, external security audit before production

**Risk: XZepr API Changes**

- **Impact**: Client integration breaks
- **Likelihood**: Low
- **Mitigation**: Version XZepr API endpoints, integration tests catch breaking changes, contract testing

### Medium-Risk Items

**Risk: Documentation Incomplete**

- **Impact**: Users cannot adopt system
- **Likelihood**: Medium
- **Mitigation**: Documentation task in every phase, dedicated Phase 3 documentation completion, examples tested as code

**Risk: Test Coverage Insufficient**

- **Impact**: Bugs reach production
- **Likelihood**: Medium
- **Mitigation**: Coverage requirements >85%, automated coverage reporting, pre-commit hooks enforce testing

**Risk: Rate Limiting Too Aggressive**

- **Impact**: False positives block legitimate users
- **Likelihood**: Medium
- **Mitigation**: Configurable rate limits, monitoring for false positives, gradual tightening in production

## Dependencies & Sequencing

### Critical Path

```
Phase 1.1-1.3 (Config, Auth, Validation)
    → Phase 1.4 (XZepr Client)
        → Phase 2.1-2.3 (MCP Server & Handlers)
            → Phase 3 (Testing & Docs)
                → Phase 4 (Production Deployment)
```

### Parallel Work Opportunities

**During Phase 1**:

- Configuration system (Task 1.1) can be developed independently
- OIDC authentication (Task 1.2) and input validation (Task 1.3) can be parallel
- Observability (Task 1.7) can be developed alongside other tasks

**During Phase 2**:

- Tool definitions (Task 2.2) can be written before server setup completes
- Documentation can start while handlers are being implemented

**During Phase 3**:

- Documentation writing can happen in parallel with testing
- Performance testing can happen while security testing is ongoing

**During Phase 4**:

- Docker and Kubernetes manifests can be developed in parallel
- Operational runbooks can be written while deployment artifacts are created

### External Dependencies

**XZepr Backend**:

- Required: Phase 1.4 (client testing), Phase 2.3 (handler testing)
- Mitigation: Use Docker Compose to run local XZepr instance

**Keycloak Instance**:

- Required: Phase 1.2 (OIDC testing), Phase 3.2 (integration testing)
- Mitigation: Use Docker Compose with Keycloak container

**MCP Clients** (Claude Desktop, VSCode):

- Required: Phase 2.4 (client integration testing), Phase 3.2 (E2E testing)
- Mitigation: Use rmcp test client for automated testing

## Resource Requirements

### Development Team

**Minimum Team**:

- 1 Senior Rust Developer (security focus)
- 1 Rust Developer (MCP protocol focus)
- 1 DevOps Engineer (deployment focus)

**Optimal Team**:

- 1 Tech Lead (architecture and security)
- 2 Senior Rust Developers
- 1 Security Engineer (part-time, consulting)
- 1 DevOps Engineer
- 1 Technical Writer (part-time, Phase 3-4)

### Infrastructure

**Development**:

- Development machines with Docker
- Git repository with CI/CD
- Test XZepr instance
- Test Keycloak instance

**Testing**:

- Staging environment (Kubernetes cluster)
- Performance testing environment
- Security testing tools (Burp Suite, OWASP ZAP)

**Production**:

- Kubernetes cluster (or equivalent orchestration)
- Monitoring stack (Prometheus, Grafana)
- Logging aggregation (ELK, Loki, or cloud solution)
- SIEM for audit logs

## Timeline Summary

### Total Duration: 54-68 days (10-13 weeks)

**Recommended Budget**: 14 weeks (including 2-week buffer)

**Phase 1**: 18-24 days (Weeks 1-4)

- Foundation, security infrastructure, XZepr client

**Phase 2**: 12-15 days (Weeks 5-7)

- MCP protocol implementation, tool handlers

**Phase 3**: 12-15 days (Weeks 7-10)

- Comprehensive testing, documentation, security validation

**Phase 4**: 12-14 days (Weeks 10-12)

- Production readiness, deployment, operations

### Milestones

**M1: Security Foundation Complete** (End of Phase 1, Week 4)

- OIDC authentication working
- Input validation enforced
- Rate limiting operational
- XZepr client functional

**M2: MCP Protocol Operational** (End of Phase 2, Week 6)

- All 9 tools working
- MCP server accepting clients
- Security integration complete

**M3: Production Quality Achieved** (End of Phase 3, Week 8)

- Test coverage >85%
- Security tests passing
- Performance targets met
- Documentation complete

**M4: Production Deployment Ready** (End of Phase 4, Week 10)

- Container images built
- Kubernetes deployment successful
- Monitoring operational
- Runbooks complete

## Success Metrics

### Technical Metrics

**Code Quality**:

- Test coverage >85%
- Zero clippy warnings with `-D warnings`
- All public APIs documented
- Security scan: zero HIGH/CRITICAL vulnerabilities

**Performance**:

- Throughput: ≥100 req/s sustained
- Latency: P99 <200ms (reads), <500ms (writes)
- Memory: <200MB RSS under normal load
- CPU: <50% under normal load

**Security**:

- All security tests passing
- Zero authentication bypasses
- All injection attacks blocked
- Audit logs capturing 100% of security events

**Reliability**:

- Uptime: 99.9% (excluding planned maintenance)
- Error rate: <0.1% under normal load
- Circuit breaker prevents cascading failures
- Graceful degradation under overload

### Operational Metrics

**Deployment**:

- Time to deploy: <5 minutes
- Rollback time: <2 minutes
- Zero-downtime updates

**Monitoring**:

- All critical alerts configured
- Mean time to detect (MTTD): <5 minutes
- Mean time to respond (MTTR): <30 minutes

**Documentation**:

- All Diataxis categories complete
- User feedback: documentation sufficient for onboarding
- All code examples tested and working

## Validation Gates

### Phase Completion Gates

Each phase must pass these gates before proceeding:

**Phase 1 Gate**:

- [ ] All Phase 1 tasks complete
- [ ] Unit tests passing with >80% coverage
- [ ] Integration tests passing for auth and XZepr client
- [ ] Security tests passing for validation and auth
- [ ] Configuration system validated
- [ ] Code review approved

**Phase 2 Gate**:

- [ ] All Phase 2 tasks complete
- [ ] MCP server accepts client connections
- [ ] All 9 tools functional
- [ ] Integration tests passing with MCP clients
- [ ] Security integration validated
- [ ] Code review approved

**Phase 3 Gate**:

- [ ] All Phase 3 tasks complete
- [ ] Test coverage >85%
- [ ] All security tests passing
- [ ] Performance targets met
- [ ] Documentation complete
- [ ] External security review passed (if applicable)

**Phase 4 Gate**:

- [ ] All Phase 4 tasks complete
- [ ] Docker image built and scanned
- [ ] Kubernetes deployment successful in staging
- [ ] Health checks operational
- [ ] Monitoring and alerting functional
- [ ] Runbooks tested
- [ ] Production readiness review approved

## Post-Implementation

### Phase 5: Advanced Features (Optional, Future)

These features were identified in the architecture but deferred to reduce initial scope:

**Caching Layer** (Redis):

- Cache frequently accessed events, receivers, groups
- Reduce XZepr API load
- Target: 500+ req/s with caching

**Token Refresh Automation**:

- Automatic token refresh when nearing expiry
- Reduced user friction
- Requires careful security analysis

**Advanced Rate Limiting**:

- Adaptive rate limits based on user tier
- Cost-based rate limiting (weight by operation cost)
- Distributed rate limiting for multi-instance deployments

**Enhanced Observability**:

- Distributed tracing with full call graphs
- Custom business metrics
- Anomaly detection with ML

**Multi-Region Support**:

- Regional XZepr API endpoints
- Geo-distributed rate limiting
- Regional JWKS caching

### Continuous Improvement

**Regular Reviews**:

- Monthly security reviews
- Quarterly performance reviews
- Dependency vulnerability scanning (weekly)
- OIDC configuration audits (monthly)

**Monitoring & Iteration**:

- Monitor production metrics
- Gather user feedback
- Iterate on rate limits based on actual usage
- Update injection detection patterns as new threats emerge

**Documentation Maintenance**:

- Keep documentation synchronized with code
- Update examples as API evolves
- Maintain troubleshooting guides with real incidents

## References

### Internal Documentation

- Architecture: `docs/explanation/xzepr_mcp_rust_architecture.md`
- Security Analysis: `docs/explanation/security_gap_analysis.md`
- Security Checklist: `docs/explanation/security_implementation_checklist.md`
- OIDC Configuration: `docs/how_to/configure_oidc_authentication.md`
- AGENTS.md: Development guidelines and rules

### External Resources

- MCP Specification: https://spec.modelcontextprotocol.io/
- MCP Security Guide: https://modelcontextprotocol.io/docs/concepts/security
- rmcp SDK: https://github.com/modelcontextprotocol/rust-sdk
- XZepr API Documentation: (internal)
- Keycloak Documentation: https://www.keycloak.org/docs/latest/

### Standards

- RFC 3339: Timestamp format
- RFC 7519: JWT specification
- OpenID Connect Core 1.0
- OpenAPI Specification 3.0
- OWASP Top 10

---

**Document Metadata**:

- Created: 2025
- Version: 1.0
- Status: Approved for Implementation
- Owner: XZepr MCP Development Team
