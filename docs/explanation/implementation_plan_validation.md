# Implementation Plan Validation Report

## Executive Summary

This document provides a comprehensive validation of the XZepr MCP implementation plan against the architecture specification, AGENTS.md guidelines, and PLAN.md requirements. The analysis identifies alignment, gaps, and required refactoring to ensure production-ready security and correctness.

**Overall Assessment**: The implementation plan requires **moderate refactoring** to fully align with the architecture document's security requirements and module structure.

**Key Findings**:
- Architecture alignment: 75% (needs modular structure clarification)
- Security requirements: 85% (missing some critical controls)
- Testing requirements: 90% (well-covered)
- Documentation requirements: 95% (excellent)
- Timeline accuracy: 70% (underestimates security work)

---

## Validation Results

### 1. Architecture Alignment

#### Module Structure

**Architecture Specifies**:
```
src/
├── main.rs
├── lib.rs
├── config/          # Configuration management
├── mcp/             # MCP protocol (server, tools, handlers, schemas)
├── api/             # Health endpoints and OpenAPI
├── auth/            # OIDC/JWT validation
├── xzepr/           # XZepr HTTP client
├── validation/      # Input validation and sanitization
├── error.rs         # Application errors
└── utils/           # Utilities
```

**Implementation Plan References**:
- Uses similar structure but doesn't explicitly define `validation/` module
- References `src/validation/` in Task 1.3 but not in initial project setup (Task 1.1)
- Missing explicit `api/` module setup for health endpoints
- XZepr client referenced as generic "HTTP client" without dedicated module

**Gap**: Implementation plan Task 1.1 should explicitly create ALL module directories upfront.

**Recommendation**: 
- Update Task 1.1 to create complete module structure from architecture
- Add file creation for `src/validation/mod.rs`, `src/api/mod.rs`, `src/xzepr/mod.rs`
- Reference architecture module diagram in Task 1.1

**Status**: NEEDS REFACTORING

---

#### Transport Implementation

**Architecture Specifies**: StreamableHTTP using `rmcp` crate with `StreamableHttpService` and `LocalSessionManager`

**Implementation Plan**: 
- Task 2.1 correctly specifies StreamableHTTP transport
- Correctly references `rmcp`, `StreamableHttpService`, `LocalSessionManager`
- Includes session security requirements

**Status**: ALIGNED ✓

---

#### OpenAPI Requirements

**Architecture Specifies**: 
- OpenAPI required for health and monitoring endpoints
- Use `utoipa` crate
- Expose spec at `/api/v1/openapi.json`
- Expose UI at `/api/v1/docs`

**Implementation Plan**:
- Task 4.1 mentions health endpoints
- Does NOT explicitly mention OpenAPI generation
- Does NOT specify utoipa dependency
- Does NOT mention OpenAPI spec/UI endpoints

**Gap**: OpenAPI implementation is completely missing from the plan.

**Recommendation**:
- Add Task 4.1a: "OpenAPI Specification Generation"
- Add `utoipa` and `utoipa-swagger-ui` dependencies
- Specify creation of `src/api/openapi.rs`
- Require OpenAPI spec validation tests
- Add 1-2 days to Phase 4 timeline

**Status**: CRITICAL GAP - NEEDS ADDITION

---

### 2. Security Requirements Validation

#### OIDC/JWT Authentication

**Architecture Requirements**:
- MANDATORY validation (no bypass option)
- Validate signature, issuer, audience, expiration, scopes
- Audience must be `xzepr-mcp` (prevents confused deputy)
- JWKS caching with TTL
- Token fingerprinting for audit logs
- Per-tool scope enforcement (`xzepr:read`, `xzepr:write`)

**Implementation Plan**:
- Task 1.2 marked as MANDATORY ✓
- Includes JWKS caching ✓
- Includes scope extraction and validation ✓
- Includes token fingerprinting ✓
- Explicitly states "NO bypass option" ✓
- Requires audience validation ✓

**Minor Issue**: Plan doesn't explicitly mention the `openidconnect` crate for OIDC discovery, but does list it in dependencies.

**Status**: WELL ALIGNED ✓

---

#### Input Validation and Sanitization

**Architecture Requirements**:
- Multi-layer validation (schema, ULID, injection, sanitization, JSON, query params)
- Injection pattern detection with specific patterns
- 64KB max payload size
- Recursive JSON validation
- Validation before ANY processing

**Implementation Plan**:
- Task 1.3 marked as MANDATORY ✓
- Includes injection pattern detection ✓
- Lists specific injection patterns ✓
- Includes sanitization functions ✓

**Gaps**:
- Does NOT mention 64KB payload size limit
- Does NOT explicitly mention recursive JSON validation depth limits
- Does NOT mention query parameter validation
- Missing validation for semver format (version fields)

**Recommendation**:
- Add explicit payload size validation requirement to Task 1.3
- Add recursive JSON validation with depth limit (max 10 levels)
- Add query parameter sanitization
- Add semver validation using `semver` crate
- Add 1 day to Task 1.3 duration

**Status**: NEEDS ENHANCEMENT

---

#### Rate Limiting

**Architecture Requirements**:
- Global: 10 req/s per user
- Burst: 20 requests
- Per-tool limits: fetch (100/min), search (20/min), create (10/min)
- Rate limit headers in responses
- User-specific limits (not global)
- 429 response when exceeded

**Implementation Plan**:
- Task 1.6 includes rate limiting marked MANDATORY ✓
- Uses `governor` crate ✓
- Mentions per-tool limits ✓

**Gaps**:
- Does NOT specify exact rate limit values from architecture
- Does NOT mention rate limit response headers
- Does NOT explicitly state user-specific (not global) limits

**Recommendation**:
- Update Task 1.6 to include specific rate limit values from architecture
- Add requirement for X-RateLimit-* headers
- Clarify rate limiting is per-user (keyed by JWT `sub` claim)
- Add test requirement for rate limit header presence

**Status**: NEEDS CLARIFICATION

---

#### Session Security

**Architecture Requirements**:
- Cryptographically secure session IDs
- Session binding to JWT `sub` claim
- 30-minute idle timeout
- Session rotation on re-authentication
- Session ID validation

**Implementation Plan**:
- Task 2.1 includes session security requirements ✓
- Mentions secure session ID generation ✓
- Includes session binding ✓
- Includes session expiration ✓
- Includes session rotation ✓

**Status**: WELL ALIGNED ✓

---

#### Audit Logging

**Architecture Requirements**:
- Log all tool invocations with: user, tool, input fingerprint, result, duration
- Token fingerprints in logs (never full tokens)
- Structured JSON format
- Auth failures logged
- Rate limit events logged
- Suspicious patterns logged

**Implementation Plan**:
- Task 1.7 covers observability infrastructure
- Mentions structured logging ✓
- Task 2.3 mentions automatic audit logging ✓

**Gaps**:
- Does NOT explicitly list all required audit log fields
- Does NOT mention logging auth failures specifically
- Does NOT mention logging rate limit events
- Does NOT mention logging suspicious injection patterns

**Recommendation**:
- Update Task 1.7 to specify comprehensive audit log schema
- Add explicit requirements for auth/rate-limit/injection event logging
- Add audit log validation tests
- Reference security monitoring requirements

**Status**: NEEDS ENHANCEMENT

---

### 3. Testing Requirements

**Architecture Requirements**:
- Unit test coverage >80%
- Integration tests with mock services
- Security tests (attack simulation)
- Performance tests (baseline throughput)
- All quality checks must pass (fmt, check, clippy, test)

**Implementation Plan**:
- Phase 3 dedicated to testing ✓
- Task 3.1: Comprehensive unit testing ✓
- Task 3.2: Integration & E2E testing ✓
- Task 3.3: Security testing & attack simulation ✓
- Task 3.4: Performance & load testing ✓
- All tasks require >80% coverage ✓

**Status**: EXCELLENT ALIGNMENT ✓

---

### 4. Dependencies Validation

**Architecture Specifies**:

Core Dependencies:
- `rmcp` (MCP transport)
- `axum` (HTTP framework)
- `tokio` (async runtime)
- `reqwest` (HTTP client)
- `serde`, `serde_json`, `serde_yaml`
- `config` (configuration)
- `jsonwebtoken`, `openidconnect` (auth)
- `utoipa` (OpenAPI)
- `governor` (rate limiting)
- `tracing`, `tracing-opentelemetry`
- `prometheus` (metrics)
- `validator` (validation)
- `ulid` (identifiers)
- `thiserror` (errors)

**Implementation Plan Lists**:
- Most dependencies correctly listed
- Missing: `utoipa`, `utoipa-swagger-ui`, `semver`, `regex`
- Missing: `tower` and `tower-http` (for middleware)

**Recommendation**:
- Add missing dependencies to Phase 1 Task 1.1
- Group dependencies by functional area for clarity
- Specify minimum versions where critical

**Status**: NEEDS ENHANCEMENT

---

### 5. Documentation Requirements

**AGENTS.md Requirements**:
- Use `.yaml` extension (not `.yml`)
- Use lowercase_with_underscores.md for filenames
- No emojis in documentation
- Documentation in `docs/explanation/` with implementation summaries
- Follow Diataxis framework

**Implementation Plan**:
- Task 3.5 covers documentation completion ✓
- References Diataxis categories ✓
- Specifies lowercase filenames ✓
- Lists comprehensive documentation deliverables ✓

**Minor Issue**: Plan uses `.yml` in one example (should be `.yaml`)

**Recommendation**: Update config file examples to use `.yaml` extension

**Status**: MINOR FIX NEEDED

---

### 6. Timeline and Resource Assessment

**Implementation Plan Estimates**:
- Phase 1: 15-20 days (Foundation & Security)
- Phase 2: 10-12 days (MCP Protocol)
- Phase 3: 12-15 days (Testing & Documentation)
- Phase 4: 10-12 days (Production Readiness)
- Total: 47-59 days (9-12 weeks)

**Architecture Complexity Analysis**:
- 9 MCP tools to implement
- 5 major security controls (auth, validation, rate-limit, audit, session)
- StreamableHTTP transport setup
- OpenAPI generation (not in current estimates)
- Comprehensive testing suite

**Timeline Validation**:

Phase 1 (15-20 days):
- Task 1.1 (Project setup): 2-3 days - REASONABLE
- Task 1.2 (OIDC/JWT): 4-5 days - UNDERESTIMATED (should be 5-7 days with JWKS caching and testing)
- Task 1.3 (Input validation): 3-4 days - REASONABLE (with enhancements: 4-5 days)
- Task 1.4 (XZepr client): 3-4 days - REASONABLE
- Task 1.5 (Error handling): 1-2 days - REASONABLE
- Task 1.6 (Rate limiting): 2-3 days - REASONABLE
- Task 1.7 (Observability): 2-3 days - REASONABLE

**Revised Phase 1**: 17-24 days (add 2-4 days)

Phase 2 (10-12 days):
- Session security adds complexity not fully accounted for
- 9 tool handlers with security integration is substantial work
- **Revised**: 12-15 days

Phase 3 (12-15 days):
- Comprehensive testing scope is large
- Security testing requires attack simulation
- **Estimate is reasonable**

Phase 4 (10-12 days):
- Missing OpenAPI work (add 2 days)
- **Revised**: 12-14 days

**Revised Total**: 53-68 days (10-13 weeks)

**Recommendation**: Update timeline estimates and add 1-2 week buffer for unforeseen issues.

**Status**: UNDERESTIMATED - NEEDS UPDATE

---

## Critical Gaps Summary

### Must Fix Before Implementation

1. **OpenAPI Implementation Missing**
   - Impact: HIGH (required by architecture)
   - Effort: 2 days
   - Action: Add Task 4.1a with utoipa implementation

2. **Validation Module Not in Project Setup**
   - Impact: MEDIUM (causes confusion during implementation)
   - Effort: 1 hour
   - Action: Update Task 1.1 to create all module directories

3. **Rate Limit Specifications Incomplete**
   - Impact: MEDIUM (implementation will lack clarity)
   - Effort: 1 hour
   - Action: Add specific rate limit values to Task 1.6

4. **Audit Logging Requirements Incomplete**
   - Impact: MEDIUM (security monitoring gaps)
   - Effort: 2 hours
   - Action: Expand Task 1.7 with comprehensive audit schema

5. **Payload Size Limits Missing**
   - Impact: MEDIUM (DoS vulnerability)
   - Effort: 1 hour
   - Action: Add 64KB limit to Task 1.3

6. **Timeline Underestimated**
   - Impact: HIGH (project planning accuracy)
   - Effort: 1 hour
   - Action: Update phase durations

---

## Recommended Refactoring

### Task 1.1: Project Setup (Enhancement Required)

**Add to "Files to Create" section**:

```
- `xzepr-mcp/src/validation/mod.rs` - Validation module root
- `xzepr-mcp/src/api/mod.rs` - API module root
- `xzepr-mcp/src/xzepr/mod.rs` - XZepr client module root
- `xzepr-mcp/src/auth/mod.rs` - Auth module root (stub)
- `xzepr-mcp/src/mcp/mod.rs` - MCP module root (stub)
- `xzepr-mcp/src/utils/mod.rs` - Utils module root
```

**Add to "Dependencies to Add" section**:

```
- `utoipa` (4.0+) with axum feature - OpenAPI generation
- `utoipa-swagger-ui` (6.0+) with axum feature - OpenAPI UI
- `tower` (0.4+) - Middleware framework
- `tower-http` (0.5+) with cors, limit, trace features
- `semver` (1.0+) - Semantic version validation
- `regex` (1.10+) - Pattern matching for injection detection
```

**Update duration**: 2-3 days → 3 days (to account for complete module structure)

---

### Task 1.3: Input Validation (Enhancement Required)

**Add to "Implementation Steps" section**:

```
7. Add payload size validation (64KB max for JSON payloads)
8. Implement recursive JSON validation with depth limit (max 10 levels)
9. Add query parameter sanitization for search operations
10. Implement semantic version validation using `semver` crate
11. Add content-type validation
```

**Add to "Testing Requirements" section**:

```
- Test payload size rejection (>64KB)
- Test recursive JSON depth limit enforcement
- Test query parameter injection attempts
- Test malformed semver version strings
```

**Update duration**: 3-4 days → 4-5 days

---

### Task 1.6: Rate Limiting (Clarification Required)

**Replace "Implementation Steps" section with**:

```
1. Integrate `governor` crate with token bucket algorithm
2. Configure global limits: 10 requests/second per user, burst of 20
3. Configure per-tool limits:
   - Fetch operations (fetch_event, fetch_receiver, fetch_group): 100 requests/minute
   - Search operations (search_*): 20 requests/minute  
   - Create operations (create_*): 10 requests/minute
4. Implement rate limit middleware that:
   - Extracts user identity from validated JWT (`sub` claim)
   - Applies user-specific limits (not global)
   - Returns 429 with Retry-After header when exceeded
   - Adds rate limit headers to all responses
5. Add rate limit headers to responses:
   - X-RateLimit-Limit: limit value
   - X-RateLimit-Remaining: remaining requests
   - X-RateLimit-Reset: unix timestamp of reset
   - Retry-After: seconds to wait (on 429 only)
```

**Add to "Testing Requirements" section**:

```
- Test user-specific rate limiting (different users, independent limits)
- Test per-tool rate limit differentiation
- Test rate limit header presence and accuracy
- Test burst allowance behavior
- Test 429 response format with Retry-After header
```

---

### Task 1.7: Observability (Enhancement Required)

**Add new subsection after "Implementation Steps"**:

```markdown
**Audit Logging Schema**:

All audit events must include:
- `timestamp`: RFC-3339 format
- `level`: INFO (success) or WARN (failure)
- `event_type`: tool_invocation | auth_failure | rate_limit_exceeded | injection_detected
- `user_id`: JWT sub claim
- `token_fingerprint`: SHA-256 of JWT (first 16 chars)
- `tool`: tool name (if applicable)
- `input_fingerprint`: SHA-256 of input (if applicable)
- `result`: success | error
- `error_type`: error category (if error)
- `duration_ms`: execution time
- `request_id`: correlation ID

**Events to Log**:
- Tool invocation (start and completion)
- Authentication failures (invalid token, expired, wrong audience, insufficient scope)
- Rate limit exceeded events
- Injection pattern detection events
- XZepr API errors
- Circuit breaker state changes
```

---

### NEW TASK: Task 4.1a - OpenAPI Specification Generation

**Insert before existing Task 4.1**

**Objective**: Generate OpenAPI 3.0 specification for health and monitoring endpoints

**Files to Create**:
- `xzepr-mcp/src/api/mod.rs` - API module root
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

**Duration**: 2 days

---

### Timeline Updates

**Phase 1: Foundation & Security Infrastructure**
- Original: 15-20 days
- Revised: 18-24 days (+3-4 days for enhancements)

**Phase 2: MCP Protocol Implementation**
- Original: 10-12 days
- Revised: 12-15 days (+2-3 days for session security complexity)

**Phase 3: Testing, Documentation & Security Validation**
- Original: 12-15 days
- Revised: 12-15 days (no change, already comprehensive)

**Phase 4: Production Readiness & Deployment**
- Original: 10-12 days
- Revised: 12-14 days (+2 days for OpenAPI work)

**Total Duration**:
- Original: 47-59 days (9-12 weeks)
- Revised: 54-68 days (10-13 weeks)

**Recommendation**: Budget 14 weeks (70 days) with 2-week buffer for unforeseen issues.

---

## AGENTS.md Compliance

### File Extensions

**Requirement**: Use `.yaml` for YAML files, `.md` for Markdown

**Implementation Plan**: 
- All Markdown files use `.md` ✓
- One config example uses `.yml` (should be `.yaml`)

**Action**: Fix config file extension in examples

**Status**: MINOR FIX NEEDED

---

### Markdown Naming

**Requirement**: lowercase_with_underscores.md (except README.md)

**Implementation Plan**:
- All documentation follows correct naming ✓
- `implementation_plan.md` - correct ✓
- `security_gap_analysis.md` - correct ✓

**Status**: COMPLIANT ✓

---

### Quality Gates

**Requirement**: ALL must pass before task complete
- `cargo fmt --all`
- `cargo check --all-targets --all-features`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` (>80% coverage)

**Implementation Plan**: Every task includes these checks in success criteria ✓

**Status**: COMPLIANT ✓

---

### Documentation Requirements

**Requirement**: Create documentation in `docs/explanation/` for every feature

**Implementation Plan**: Task 3.5 includes comprehensive documentation deliverables ✓

**Status**: COMPLIANT ✓

---

## PLAN.md Compliance

### API Versioning

**Requirement**: API endpoints versioned as `api/v1/<endpoint>`

**Architecture**: Health endpoints at `/health` and `/ready` (not versioned)

**Note**: MCP protocol endpoints don't use REST versioning (MCP has its own protocol versioning). Health endpoints are typically not versioned as they're operational, not API.

**Status**: ACCEPTABLE (non-MCP endpoints are operational, not API)

---

### OpenAPI Documentation

**Requirement**: OpenAPI documentation for services with endpoints

**Current**: Missing from implementation plan

**Action**: Added Task 4.1a (see refactoring section)

**Status**: FIXED

---

### Configuration

**Requirement**: Environment variables, CLI options, and/or config files

**Implementation Plan**: Task 1.1 implements all three methods with correct precedence ✓

**Status**: COMPLIANT ✓

---

### ULID Identifiers

**Requirement**: ULID preferred over UUID

**Architecture**: Specifies ULID for all identifiers ✓

**Implementation Plan**: References ULID validation ✓

**Status**: COMPLIANT ✓

---

### RFC-3339 Timestamps

**Requirement**: Use RFC-3339 format like `2025-11-07T18:12:07.982682Z`

**Architecture**: Specifies RFC-3339 format ✓

**Implementation Plan**: References ISO-8601/RFC-3339 ✓

**Status**: COMPLIANT ✓

---

### Test Coverage

**Requirement**: >80% coverage

**Implementation Plan**: Every task requires >80% coverage ✓

**Status**: COMPLIANT ✓

---

### Diataxis Documentation

**Requirement**: Follow Diataxis framework

**Implementation Plan**: Task 3.5 references Diataxis and organizes docs correctly ✓

**Status**: COMPLIANT ✓

---

## Final Recommendations

### Immediate Actions (Before Starting Implementation)

1. **Update Task 1.1** - Add all module directories and missing dependencies (1 hour)
2. **Add Task 4.1a** - OpenAPI specification generation (copy from this document) (15 min)
3. **Enhance Task 1.3** - Add payload limits and recursive validation (30 min)
4. **Clarify Task 1.6** - Add specific rate limit values (30 min)
5. **Enhance Task 1.7** - Add comprehensive audit logging schema (1 hour)
6. **Update timelines** - Revise all phase durations (30 min)
7. **Fix config extensions** - Change `.yml` to `.yaml` in examples (5 min)

**Total Refactoring Effort**: 4 hours

---

### Implementation Phase Adjustments

**Phase 1 Changes**:
- Extend by 3-4 days for enhancements
- Ensure all modules created upfront
- Add comprehensive validation requirements

**Phase 2 Changes**:
- Extend by 2-3 days for session security
- Add explicit OpenAPI endpoint testing

**Phase 4 Changes**:
- Add Task 4.1a for OpenAPI
- Extend by 2 days

---

### Risk Mitigation

**Identified Risks**:

1. **OIDC Integration Complexity** (HIGH)
   - Mitigation: Allocate extra time for JWKS testing
   - Consider mock JWKS server for development

2. **Rate Limiting User Identification** (MEDIUM)
   - Mitigation: Ensure JWT `sub` extraction happens early
   - Test with multiple concurrent users

3. **Session Security Implementation** (MEDIUM)
   - Mitigation: Study rmcp LocalSessionManager documentation
   - Implement session storage abstraction for future Redis

4. **Timeline Pressure** (HIGH)
   - Mitigation: Use revised timeline estimates
   - Add 2-week buffer for unknowns
   - Consider Phase 2 sub-phase split

---

## Conclusion

The implementation plan is **well-structured and comprehensive** but requires **moderate refactoring** to fully align with the architecture specification. The primary gaps are:

1. Missing OpenAPI implementation (critical)
2. Incomplete validation requirements (medium)
3. Underestimated timeline (high impact on planning)
4. Minor security specification gaps (audit logging, rate limits)

**Estimated Refactoring Effort**: 4 hours of document updates

**Revised Implementation Timeline**: 54-68 days (10-13 weeks) + 2-week buffer = 14 weeks total

**Recommendation**: Apply the refactoring changes outlined in this document before beginning implementation. The architecture foundation is solid, and with these adjustments, the implementation plan will provide clear, actionable guidance for building a production-ready, secure XZepr MCP server.

---

## Approval Checklist

Before proceeding with implementation, verify:

- [ ] All critical gaps addressed
- [ ] OpenAPI task added (Task 4.1a)
- [ ] Module structure complete in Task 1.1
- [ ] Security requirements fully specified
- [ ] Timeline estimates updated
- [ ] All dependencies listed
- [ ] AGENTS.md compliance verified
- [ ] PLAN.md compliance verified
- [ ] Architecture alignment confirmed
- [ ] Risk mitigation plans documented

**Document Version**: 1.0  
**Date**: 2025-01-07  
**Validated Against**:
- `xzepr_mcp_rust_architecture.md` (updated with security)
- `implementation_plan.md` (current version)
- `AGENTS.md` (xzepr-mcp)
- `PLAN.md` (xzepr)

---
