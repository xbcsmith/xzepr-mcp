# Security & Quality Improvements Implementation Plan

## Overview

This implementation plan addresses security hardening, quality-of-life improvements, and completion of stub implementations identified during a comprehensive code review of the XZepr MCP Server. The plan follows a phased approach, prioritizing critical bug fixes before feature enhancements.

**Key Features:**

- Fix critical compilation errors in `SecurityCheck` enum
- Implement functional rate limiting with `governor` crate (in-memory)
- Add security response headers middleware (CSP, HSTS, X-Frame-Options)
- Complete XZepr HTTP client with circuit breaker pattern (RwLock-based)
- Add mutual TLS (mTLS) support for service-to-service authentication
- Create OpenAPI documentation for MCP endpoints
- Implement real health checks with XZepr connectivity testing

## Current State Analysis

### Existing Infrastructure

The codebase has a solid security foundation:

| Component | Location | Status |
|-----------|----------|--------|
| JWT/OIDC Authentication | `src/auth/jwt.rs` | ✅ Complete |
| Session Management | `src/auth/session.rs` | ✅ Complete |
| Input Validation | `src/middleware/validation.rs` | ✅ Complete |
| Security Config | `src/config/settings.rs` | ⚠️ Bug (compile error) |
| Rate Limiter | `src/middleware/rate_limit.rs` | 🔶 Stub |
| XZepr Client | `src/client/xzepr.rs` | 🔶 Stub |
| Observability | `src/observability/mod.rs` | 🔶 Stub |
| MCP Server | `src/mcp/server.rs` | ⚠️ Hardcoded health checks |
| Security Headers | N/A | ❌ Missing |
| mTLS Support | N/A | ❌ Missing |
| OpenAPI Docs | N/A | ❌ Missing |

### Identified Issues

1. **Critical Bug**: `SecurityCheck` enum in `src/config/settings.rs` is missing `Hash`, `Eq`, `PartialEq` derives and is not re-exported from `src/config/mod.rs`, causing compilation failure.

2. **Stub Implementations**: Rate limiter always returns `Ok(())`, XZepr client returns mock data, observability init is a no-op.

3. **Missing Security Headers**: No Content-Security-Policy, Strict-Transport-Security, X-Frame-Options, X-Content-Type-Options, or X-XSS-Protection headers in HTTP responses.

4. **No mTLS**: Service-to-service communication lacks certificate-based mutual authentication.

5. **Hardcoded Health Checks**: The `/health` endpoint always reports `xzepr_connectivity: true` without actual connectivity verification.

6. **Missing OpenAPI**: Per PLAN.md service checklist, REST APIs require OpenAPI documentation, but none exists for MCP endpoints.

## Implementation Phases

### Phase 1: Critical Bug Fixes

Fix compilation errors and ensure the codebase builds successfully.

#### Task 1.1: Fix SecurityCheck Enum

**File:** `src/config/settings.rs`

Add required derive macros to `SecurityCheck` enum:

- Add `#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]`
- This enables use in `HashSet<SecurityCheck>` for detection configuration

**File:** `src/config/mod.rs`

Update re-exports to include:

- `SecurityCheck` enum
- `DetectionConfig` struct

#### Task 1.2: Verify Build

Run `cargo build` and `cargo test` to confirm all compilation errors are resolved.

#### Task 1.3: Testing Requirements

- All existing unit tests must pass
- `cargo clippy` must report no new warnings

#### Task 1.4: Deliverables

- [ ] `SecurityCheck` enum compiles with `HashSet` usage
- [ ] All existing tests pass
- [ ] No clippy warnings introduced

#### Task 1.5: Success Criteria

- `cargo build --release` completes without errors
- `cargo test` reports 0 failures
- CI pipeline (if configured) passes

---

### Phase 2: Rate Limiter Implementation

Implement functional rate limiting using the `governor` crate with in-memory storage.

#### Task 2.1: Rate Limiter Core

**File:** `src/middleware/rate_limit.rs`

Implement sliding window rate limiting:

- Use `governor::RateLimiter` with `InMemoryState`
- Create per-user rate limit buckets using `DashMap<String, RateLimiter>`
- Support global, per-user, and per-tool rate limits from `RateLimitConfig`
- Return `Error::RateLimit` with retry-after duration when exceeded

**Structures to implement:**

```
RateLimiter
├── global_limiter: Governor<...>
├── user_limiters: DashMap<String, Governor<...>>
└── tool_limiters: DashMap<String, Governor<...>>
```

#### Task 2.2: Rate Limit Headers

Add standard rate limit response headers:

- `X-RateLimit-Limit`: Maximum requests allowed
- `X-RateLimit-Remaining`: Requests remaining in window
- `X-RateLimit-Reset`: Timestamp when window resets (RFC-3339)

#### Task 2.3: Integration

Update `src/mcp/server.rs` `handle_tool_call` to:

- Extract rate limit info from limiter response
- Add rate limit headers to successful responses
- Return 429 Too Many Requests with `Retry-After` header when exceeded

#### Task 2.4: Testing Requirements

- Unit tests for `check_limit` with various scenarios
- Test global limit exhaustion
- Test per-user limit exhaustion
- Test per-tool limit exhaustion
- Test limit reset after window expires

#### Task 2.5: Deliverables

- [ ] Functional `check_limit` implementation
- [ ] Rate limit headers on all responses
- [ ] 429 responses when limits exceeded
- [ ] Unit tests with >80% coverage

#### Task 2.6: Success Criteria

- Rate limiting prevents abuse as configured
- Headers accurately reflect remaining quota
- Tests demonstrate correct window behavior

---

### Phase 3: Security Response Headers

Add security-focused HTTP response headers to all endpoints.

#### Task 3.1: Security Headers Middleware

**File:** `src/middleware/security_headers.rs` (new file)

Create Axum middleware layer that adds headers to all responses:

| Header | Value | Purpose |
|--------|-------|---------|
| `X-Content-Type-Options` | `nosniff` | Prevent MIME-type sniffing |
| `X-Frame-Options` | `DENY` | Prevent clickjacking |
| `X-XSS-Protection` | `1; mode=block` | Enable XSS filter |
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` | Enforce HTTPS |
| `Content-Security-Policy` | `default-src 'self'` | Restrict resource loading |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Control referrer info |
| `Permissions-Policy` | `geolocation=(), microphone=()` | Disable browser features |

#### Task 3.2: Configuration Support

**File:** `src/config/settings.rs`

Add `SecurityHeadersConfig` to `SecurityConfig`:

- `enable_hsts: bool` (default: true in production)
- `hsts_max_age_secs: u64` (default: 31536000)
- `csp_policy: String` (default: "default-src 'self'")
- `frame_options: String` (default: "DENY")

#### Task 3.3: Integration

Update `src/mcp/server.rs` `build_router` to:

- Add security headers middleware layer
- Make headers configurable via settings

#### Task 3.4: Testing Requirements

- Unit tests verifying each header is present
- Test header values match configuration
- Test headers present on error responses

#### Task 3.5: Deliverables

- [ ] `SecurityHeadersMiddleware` implementation
- [ ] Configuration options for header values
- [ ] Headers on all responses (success and error)
- [ ] Unit tests for header presence

#### Task 3.6: Success Criteria

- Security scanner (e.g., Mozilla Observatory) reports A+ rating
- All configured headers present in responses

---

### Phase 4: XZepr Client with Circuit Breaker

Complete XZepr HTTP client implementation with retry logic and circuit breaker pattern.

#### Task 4.1: Circuit Breaker Implementation

**File:** `src/client/circuit_breaker.rs` (new file)

Implement minimal circuit breaker using `RwLock`:

```
CircuitBreaker
├── state: RwLock<CircuitState>
├── failure_count: AtomicU32
├── last_failure_time: RwLock<Option<Instant>>
├── threshold: u32
└── timeout: Duration

CircuitState { Closed, Open, HalfOpen }
```

**Behavior:**

- **Closed**: Requests pass through, failures increment counter
- **Open**: Requests fail immediately with `CircuitBreakerOpen` error
- **Half-Open**: Single probe request allowed; success closes, failure reopens

#### Task 4.2: Retry Logic

**File:** `src/client/xzepr.rs`

Implement exponential backoff retry:

- Use `retry_backoff_ms` from `XzeprConfig` as base
- Exponential backoff: `base * 2^attempt` with jitter
- Maximum retries from `max_retries` config
- Retry only on transient errors (5xx, timeout, connection)

#### Task 4.3: Complete API Methods

Implement actual HTTP calls for:

- `get_event(event_id)` → `GET /api/v1/events/{id}`
- `create_event(...)` → `POST /api/v1/events`
- `search_events(...)` → `GET /api/v1/events?...`

Include proper error mapping to `XzeprApiError` variants.

#### Task 4.4: Integration

Update circuit breaker state on request results:

- Record success on 2xx responses
- Record failure on 5xx, timeout, connection errors
- Do not record failure on 4xx (client errors)

#### Task 4.5: Testing Requirements

- Unit tests for circuit breaker state transitions
- Integration tests with `wiremock` for API calls
- Test retry behavior with transient failures
- Test circuit opens after threshold failures

#### Task 4.6: Deliverables

- [ ] `CircuitBreaker` struct with state machine
- [ ] Retry logic with exponential backoff
- [ ] Functional API client methods
- [ ] Tests for all state transitions

#### Task 4.7: Success Criteria

- Circuit opens after configured failures
- Circuit closes after timeout + successful probe
- Retries succeed on transient failures
- No retries on client errors (4xx)

---

### Phase 5: Mutual TLS (mTLS) Support

Add certificate-based mutual authentication for service-to-service communication.

#### Task 5.1: TLS Configuration

**File:** `src/config/settings.rs`

Add `TlsConfig` to `ServerConfig`:

```yaml
tls:
  enabled: bool
  cert_path: PathBuf
  key_path: PathBuf
  ca_cert_path: PathBuf  # For client verification
  require_client_cert: bool  # Enable mTLS
  min_version: String  # "1.2" or "1.3"
```

#### Task 5.2: Server TLS Setup

**File:** `src/mcp/server.rs`

Update server initialization to:

- Load server certificate and private key
- Configure `rustls` with client certificate verification
- Use `axum_server::tls_rustls` for TLS listener
- Validate minimum TLS version

#### Task 5.3: Client TLS Setup

**File:** `src/client/xzepr.rs`

Update `XzeprClient::new` to:

- Load client certificate for outbound mTLS
- Configure `reqwest` with client identity
- Add CA certificate for server verification

#### Task 5.4: Certificate Validation

Implement certificate verification:

- Validate certificate chain against CA
- Check certificate not expired
- Verify Subject Alternative Names (SANs)
- Optional: Check certificate revocation (CRL/OCSP)

#### Task 5.5: Testing Requirements

- Integration tests with self-signed certificates
- Test connection rejection without client cert (when required)
- Test connection rejection with invalid cert
- Test successful mTLS handshake

#### Task 5.6: Deliverables

- [ ] `TlsConfig` configuration struct
- [ ] Server-side mTLS support
- [ ] Client-side mTLS support
- [ ] Certificate validation logic
- [ ] Integration tests with test certificates

#### Task 5.7: Success Criteria

- Server rejects connections without valid client cert (when enabled)
- Client rejects servers with invalid certificates
- TLS 1.2+ enforced, older versions rejected

---

### Phase 6: Health Check Implementation

Replace hardcoded health checks with actual connectivity verification.

#### Task 6.1: XZepr Connectivity Check

**File:** `src/client/xzepr.rs`

Add health check method:

- `health_check() -> Result<bool>` 
- Call XZepr health endpoint (e.g., `GET /api/v1/health`)
- Timeout after 5 seconds
- Return health status

#### Task 6.2: Update Health Endpoint

**File:** `src/mcp/server.rs`

Update `health_check` handler:

- Call `xzepr_client.health_check()`
- Track actual uptime using `Instant::now()` at startup
- Include circuit breaker state in response
- Return 503 if any critical check fails

#### Task 6.3: Kubernetes Probes

Add separate endpoints for K8s:

- `/health/ready` - Readiness probe (all dependencies available)
- `/health/live` - Liveness probe (process is running)

#### Task 6.4: Testing Requirements

- Unit tests for health check logic
- Integration tests with mock XZepr responses
- Test degraded state reporting

#### Task 6.5: Deliverables

- [ ] Real XZepr connectivity check
- [ ] Accurate uptime tracking
- [ ] Separate readiness/liveness probes
- [ ] Degraded state detection

#### Task 6.6: Success Criteria

- Health endpoint reflects actual system state
- Kubernetes probes work correctly
- Circuit breaker state visible in health response

---

### Phase 7: OpenAPI Documentation

Add OpenAPI 3.0 documentation for all MCP endpoints.

#### Task 7.1: OpenAPI Annotations

**Files:** `src/mcp/server.rs`, `src/models/*.rs`

Add `utoipa` annotations:

- `#[utoipa::path(...)]` on all handler functions
- `#[derive(ToSchema)]` on request/response structs
- Document all parameters, responses, and error codes

#### Task 7.2: Swagger UI Integration

Update `build_router` to serve:

- `/api/doc/openapi.json` - OpenAPI spec
- `/api/doc/swagger-ui` - Interactive documentation

#### Task 7.3: Security Schemes

Document authentication:

- Bearer JWT token in Authorization header
- Required scopes for each endpoint
- Session ID cookie/header

#### Task 7.4: Testing Requirements

- Validate OpenAPI spec is valid JSON
- Test Swagger UI loads correctly
- Verify all endpoints documented

#### Task 7.5: Deliverables

- [ ] OpenAPI annotations on all endpoints
- [ ] Swagger UI integration
- [ ] Security scheme documentation
- [ ] Example requests/responses

#### Task 7.6: Success Criteria

- OpenAPI spec validates against 3.0 schema
- All endpoints discoverable in Swagger UI
- Authentication documented correctly

---

## Dependency Changes

Add to `Cargo.toml` if not present:

```toml
# Rate limiting (already present)
governor = "0.6"
dashmap = "5.5"

# mTLS
rustls = "0.21"
rustls-pemfile = "1.0"
tokio-rustls = "0.24"
axum-server = { version = "0.5", features = ["tls-rustls"] }

# OpenAPI (already present)
utoipa = { version = "4.2", features = ["axum_extras", "chrono"] }
utoipa-swagger-ui = { version = "6.0", features = ["axum"] }
```

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| mTLS breaks existing clients | High | Feature flag, gradual rollout |
| Rate limiter too aggressive | Medium | Conservative defaults, monitoring |
| Circuit breaker flapping | Medium | Proper threshold tuning, half-open state |
| OpenAPI maintenance burden | Low | Generate from code annotations |

## Timeline Estimate

| Phase | Effort | Dependencies |
|-------|--------|--------------|
| Phase 1: Bug Fixes | 1-2 hours | None |
| Phase 2: Rate Limiter | 4-6 hours | Phase 1 |
| Phase 3: Security Headers | 2-3 hours | Phase 1 |
| Phase 4: XZepr Client | 6-8 hours | Phase 1 |
| Phase 5: mTLS | 8-10 hours | Phase 4 |
| Phase 6: Health Checks | 2-3 hours | Phase 4 |
| Phase 7: OpenAPI | 4-6 hours | Phase 1 |

**Total Estimated Effort:** 27-38 hours

## Copyright

SPDX-License-Identifier: Apache-2.0
