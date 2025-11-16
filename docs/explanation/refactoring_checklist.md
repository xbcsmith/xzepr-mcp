# Implementation Plan Refactoring Checklist

## Overview

This checklist provides step-by-step instructions for applying the refactoring changes identified in the validation analysis. Complete all items before beginning implementation.

**Estimated Time**: 4 hours
**Priority**: MANDATORY before implementation begins
**Validation Document**: `validation_summary.md`

---

## Checklist Items

### 1. Update Task 1.1: Project Setup (30 minutes)

**File**: `implementation_plan.md` - Task 1.1

**Actions**:

- [ ] Add to "Files to Create" section:
  ```
  - `xzepr-mcp/src/validation/mod.rs` - Validation module root (stub)
  - `xzepr-mcp/src/api/mod.rs` - API module root (stub)
  - `xzepr-mcp/src/xzepr/mod.rs` - XZepr client module root (stub)
  ```

- [ ] Add to "Dependencies to Add" section:
  ```
  OpenAPI:
  - `utoipa` (4.0+) with axum feature - OpenAPI generation
  - `utoipa-swagger-ui` (6.0+) with axum feature - OpenAPI UI

  Middleware:
  - `tower` (0.4+) - Middleware framework
  - `tower-http` (0.5+) with cors, limit, trace features

  Validation:
  - `semver` (1.0+) - Semantic version validation
  - `regex` (1.10+) - Pattern matching (if not already listed)
  ```

- [ ] Update duration: Change "2-3 days" to "3 days"

- [ ] Add reference to architecture doc:
  ```
  **Module Structure Reference**: See `architecture.md` section "Module Structure"
  ```

**Verification**: All module directories and dependencies listed

---

### 2. Add Task 4.1a: OpenAPI Specification (15 minutes)

**File**: `implementation_plan.md` - Insert before Task 4.1

**Actions**:

- [ ] Add new task section:

```markdown
#### Task 4.1a: OpenAPI Specification Generation (NEW)

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
```

- [ ] Update Task 4.1 numbering to 4.1b (renumber all subsequent tasks)

**Verification**: Task 4.1a exists before health checks task

---

### 3. Enhance Task 1.3: Input Validation (30 minutes)

**File**: `implementation_plan.md` - Task 1.3

**Actions**:

- [ ] Add to "Implementation Steps" section (after existing steps):
  ```
  7. Add payload size validation (64KB max for JSON payloads)
  8. Implement recursive JSON validation with depth limit (max 10 levels)
  9. Add query parameter sanitization for search operations
  10. Implement semantic version validation using `semver` crate
  11. Add content-type validation
  ```

- [ ] Add to "Validation Rules" section (create if doesn't exist):
  ```
  **Validation Rules**:
  - JSON payload: Max 64KB size, max 10 levels depth
  - Query parameters: Max 255 characters, alphanumeric + limited special chars
  - Semantic Version: Valid semver format (e.g., "1.2.3", "1.0.0-alpha")
  ```

- [ ] Add to "Testing Requirements" section:
  ```
  - Test payload size rejection (>64KB)
  - Test recursive JSON depth limit enforcement
  - Test query parameter injection attempts
  - Test malformed semver version strings
  ```

- [ ] Update duration: Change "3-4 days" to "4-5 days"

**Verification**: Payload size, recursive JSON, query params, and semver validation mentioned

---

### 4. Clarify Task 1.6: Rate Limiting (30 minutes)

**File**: `implementation_plan.md` - Task 1.6

**Actions**:

- [ ] Replace "Implementation Steps" with:
  ```
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
     - X-RateLimit-Limit: limit value
     - X-RateLimit-Remaining: remaining requests
     - X-RateLimit-Reset: unix timestamp of reset
     - Retry-After: seconds to wait (on 429 only)
  ```

- [ ] Add to "Testing Requirements" section:
  ```
  - Test user-specific rate limiting (different users, independent limits)
  - Test per-tool rate limit differentiation
  - Test rate limit header presence and accuracy
  - Test burst allowance behavior
  - Test 429 response format with Retry-After header
  ```

**Verification**: Specific rate limit values and headers documented

---

### 5. Enhance Task 1.7: Observability (1 hour)

**File**: `implementation_plan.md` - Task 1.7

**Actions**:

- [ ] Add new subsection after "Implementation Steps":
  ```markdown
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
  ```

**Verification**: Complete audit logging schema and events list added

---

### 6. Update All Phase Timelines (30 minutes)

**File**: `implementation_plan.md` - All phase headers

**Actions**:

- [ ] Update Phase 1 header:
  ```
  Change: **Duration**: 15-20 days
  To:     **Duration**: 18-24 days (revised from 15-20 days)
  ```

- [ ] Update Phase 2 header:
  ```
  Change: **Duration**: 10-12 days
  To:     **Duration**: 12-15 days (revised from 10-12 days)
  ```

- [ ] Phase 3 header (no change needed, verify it says):
  ```
  **Duration**: 12-15 days
  ```

- [ ] Update Phase 4 header:
  ```
  Change: **Duration**: 10-12 days
  To:     **Duration**: 12-14 days (revised from 10-12 days)
  ```

- [ ] Update "Timeline Summary" section (if exists):
  ```
  Change: Total Duration: 47-59 days (9-12 weeks)
  To:     Total Duration: 54-68 days (10-13 weeks)

  Add:    **Recommended Budget**: 14 weeks (including 2-week buffer)
  ```

**Verification**: All phase durations updated, total reflects 54-68 days

---

### 7. Fix Config File Extensions (5 minutes)

**File**: `implementation_plan.md` - Search entire document

**Actions**:

- [ ] Search for all instances of `.yml`
- [ ] Replace with `.yaml`
- [ ] Common locations:
  - Configuration file examples
  - Docker compose file references
  - CI/CD pipeline references

**Verification**: No `.yml` extensions remain (except in "wrong example" comparisons)

---

### 8. Add Revision History (5 minutes)

**File**: `implementation_plan.md` - After "Overview" section

**Actions**:

- [ ] Add revision history section:
  ```markdown
  **Revision History**:
  - Version 1.0: Initial implementation plan
  - Version 2.0: Refactored based on validation analysis (2025-01-07)
    - Added OpenAPI specification generation (Task 4.1a)
    - Enhanced input validation requirements (Task 1.3)
    - Clarified rate limiting specifications (Task 1.6)
    - Enhanced audit logging requirements (Task 1.7)
    - Updated module structure setup (Task 1.1)
    - Revised timeline estimates (+7-9 days)
  ```

**Verification**: Revision history documents major changes

---

## Verification Steps

After completing all checklist items:

### Document Validation

- [ ] All checklist items marked complete
- [ ] No `.yml` extensions remain (except in examples of what NOT to do)
- [ ] All timelines updated consistently
- [ ] OpenAPI task exists and is complete
- [ ] All enhancement sections added

### Content Validation

- [ ] Task 1.1 includes all module directories
- [ ] Task 1.1 includes all dependencies
- [ ] Task 1.3 includes payload size, JSON depth, semver validation
- [ ] Task 1.6 includes specific rate limit values and headers
- [ ] Task 1.7 includes complete audit logging schema
- [ ] Task 4.1a (OpenAPI) exists with full specifications

### Timeline Validation

- [ ] Phase 1: 18-24 days
- [ ] Phase 2: 12-15 days
- [ ] Phase 3: 12-15 days
- [ ] Phase 4: 12-14 days
- [ ] Total: 54-68 days mentioned in summary
- [ ] 14-week recommended budget mentioned

### Cross-Reference Validation

- [ ] All changes align with `validation_summary.md`
- [ ] All changes align with `architecture.md`
- [ ] All changes comply with `AGENTS.md`
- [ ] All changes comply with `PLAN.md`

---

## Post-Refactoring Actions

After refactoring is complete:

1. **Review Meeting** (1 hour)
   - Walk through changes with team
   - Confirm understanding of new requirements
   - Address questions

2. **Update Project Schedule** (30 minutes)
   - Update project timeline to 14 weeks
   - Allocate resources according to revised estimates
   - Set phase milestones

3. **Set Up Development Environment** (1 day)
   - Install required tools
   - Set up mock services (Keycloak, XZepr)
   - Configure CI/CD pipeline

4. **Begin Phase 1 Implementation**
   - Follow Golden Workflow from AGENTS.md
   - Implement Task 1.1 first (project setup)
   - Run quality checks after each task

---

## Sign-Off

**Refactoring Completed By**: ___________________
**Date**: ___________________
**Reviewed By**: ___________________
**Date**: ___________________
**Approved for Implementation**: ___________________
**Date**: ___________________

---

## Notes

Use this space to document any issues or decisions made during refactoring:

```

---

**Document Version**: 1.0
**Date**: 2025-01-07
**Status**: Ready for Use
