# XZepr MCP Rust Server Architecture

## Overview

This document outlines the architecture for implementing an MCP (Model Context Protocol) server for XZepr in Rust. The MCP server will provide AI assistants with structured access to XZepr's event tracking and provenance registry capabilities through a standardized protocol interface.

The XZepr MCP server bridges AI agents and the XZepr High-Performance Event Tracking Server, enabling intelligent event management, querying, and analysis through natural language interactions.

## Current State Analysis

### Existing Infrastructure

#### XZepr Core (Rust)

XZepr is a production-ready event tracking system built in Rust with:

- **Layered Architecture**: Domain-driven design with clear separation between API, application, domain, and infrastructure layers
- **Multiple API Styles**: REST (versioned at `/api/v1`), GraphQL, and health check endpoints
- **Domain Entities**:
  - `Event` - Core event entity with ULID-based IDs
  - `EventReceiver` - Event receiver/handler definitions
  - `EventReceiverGroup` - Grouped receiver configurations
- **Value Objects**: Type-safe IDs using ULID (EventId, EventReceiverId, EventReceiverGroupId)
- **Database**: PostgreSQL with SQLx for type-safe queries
- **Authentication**: JWT, OAuth2, OpenID Connect with Argon2 password hashing
- **Observability**: OpenTelemetry, Prometheus metrics, structured logging
- **Messaging**: Kafka event publishing
- **Configuration**: Environment variables, config files, command-line options

#### EPR MCP Python Reference Implementation

The existing Python implementation (epr-mcp-python) provides:

- **FastMCP Framework**: HTTP-based MCP server using the fastmcp library
- **Auto-generated Tools**: MCP tools derived from OpenAPI specification
- **Core Operations**:
  - `fetchEvent` - Fetch event by ID
  - `fetchReceiver` - Fetch event receiver by ID
  - `fetchGroup` - Fetch event receiver group by ID
  - `createEvent` - Create new events
  - `createReceiver` - Create new event receivers
  - `createGroup` - Create new event receiver groups
  - `searchEvents` - Search events with filters
  - `searchReceivers` - Search receivers with filters
  - `searchGroups` - Search groups with filters
  - `healthCheck` - Health status endpoint
- **OpenAPI Integration**: YAML/JSON spec serving, Swagger UI
- **Schema Validation**: Pydantic models for request/response validation
- **Error Handling**: Connection errors, timeouts, HTTP status errors
- **Transport**: HTTP with `/mcp` endpoint at port 8000

### Identified Issues

1. **No Rust MCP Implementation**: Currently no MCP server exists for XZepr in Rust
2. **Integration Gap**: Need seamless integration between MCP protocol and XZepr's REST/GraphQL APIs
3. **Type Safety**: Must maintain Rust's type safety guarantees while implementing MCP protocol
4. **Authentication**: Need to handle XZepr's JWT/OAuth2 authentication in MCP context
5. **Performance**: Should leverage Rust's performance characteristics for high-throughput scenarios
6. **Error Handling**: Comprehensive error handling matching both MCP and XZepr patterns

## Architecture Design

### System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                         AI Clients                               │
│              (Claude Desktop, VSCode, etc.)                      │
└────────────────────────────┬────────────────────────────────────┘
                             │ MCP Protocol (HTTP/SSE)
┌────────────────────────────▼────────────────────────────────────┐
│                    XZepr MCP Server (Rust)                       │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │               MCP Protocol Layer                          │  │
│  │  - Tool Registration & Discovery                          │  │
│  │  - Request/Response Handling                              │  │
│  │  - Schema Validation                                      │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │              Business Logic Layer                         │  │
│  │  - Event Operations                                       │  │
│  │  - Receiver Management                                    │  │
│  │  - Group Management                                       │  │
│  │  - Search & Query Logic                                   │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │               XZepr Client Layer                          │  │
│  │  - HTTP Client                                            │  │
│  │  - Authentication                                         │  │
│  │  - Request Building                                       │  │
│  │  - Response Parsing                                       │  │
│  └───────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │ HTTP REST API
┌────────────────────────────▼────────────────────────────────────┐
│                     XZepr Server                                 │
│              (Event Tracking Backend)                            │
└─────────────────────────────────────────────────────────────────┘
```

### Technology Stack

#### Core Dependencies

- **MCP SDK**: `rmcp` - Rust MCP implementation with StreamableHTTP support
- **HTTP Client**: `reqwest` 0.12 - HTTP client for XZepr API communication
- **Async Runtime**: `tokio` 1.38 - Async runtime (required by rmcp)
- **Serialization**: `serde` 1.0, `serde_json` 1.0 - JSON handling
- **Error Handling**: `thiserror` 1.0, `anyhow` 1.0
- **Configuration**: `config` 0.14, `dotenvy` 0.15
- **Validation**: `validator` 0.18
- **IDs**: `ulid` 1.1 - Matching XZepr's ID strategy
- **Time**: `chrono` 0.4 - Date/time handling with RFC-3339 format
- **Logging**: `tracing` 0.1, `tracing-subscriber` 0.3, `tracing-opentelemetry` 0.21
- **Metrics**: `prometheus` 0.13 - Prometheus metrics exposition
- **OpenAPI**: `utoipa` 4.2 - Compile-time OpenAPI 3.0 generation
- **OpenAPI UI**: `utoipa-swagger-ui` 6.0, `utoipa-rapidoc` 3.0 - API documentation UIs
- **Resilience**: `tower` 0.4 - Retry, timeout, circuit breaker middleware
- **OIDC/JWT**: `jsonwebtoken` 9.2 - JWT validation and parsing
- **OIDC**: `openidconnect` 3.4 - OpenID Connect client (optional token validation)
- **Rate Limiting**: `governor` 0.6 - Token bucket rate limiting
- **Security**: `regex` 1.10 - Pattern matching for injection detection

#### Development Dependencies

- **Testing**: `tokio-test` 0.4, `mockall` 0.13
- **Benchmarking**: `criterion` 0.5

### Module Structure

```
xzepr-mcp/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── AGENTS.md
├── PLAN.md
├── .gitignore
├── .markdownlint.json
├── .markdownlintignore
├── .yamllint.yaml
├── Makefile
├── config/
│   └── default.yaml          # Default configuration
├── docs/
│   ├── explanation/
│   │   ├── xzepr_mcp_rust_architecture.md
│   │   └── design_decisions.md
│   ├── how_to/
│   │   ├── configure_server.md
│   │   └── deploy_production.md
│   ├── reference/
│   │   ├── api.md
│   │   ├── configuration.md
│   │   └── mcp_tools.md
│   └── tutorials/
│       └── quickstart.md
├── src/
│   ├── main.rs               # Binary entry point
│   ├── lib.rs                # Library root
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs       # Configuration structures
│   ├── mcp/
│   │   ├── mod.rs
│   │   ├── server.rs         # MCP server implementation (rmcp)
│   │   ├── tools.rs          # MCP tool definitions
│   │   ├── handlers.rs       # Tool handler implementations
│   │   └── schemas.rs        # MCP input/output schemas
│   ├── api/
│   │   ├── mod.rs
│   │   ├── health.rs         # Health check endpoints
│   │   └── openapi.rs        # OpenAPI spec generation (utoipa)
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── oidc.rs           # OIDC validator
│   │   ├── jwt.rs            # JWT parsing and validation
│   │   └── jwks.rs           # JWKS fetching and caching
│   ├── xzepr/
│   │   ├── mod.rs
│   │   ├── client.rs         # XZepr HTTP client
│   │   ├── types.rs          # XZepr domain types
│   │   └── error.rs          # XZepr client errors
│   ├── error.rs              # Application-wide errors
│   └── utils/
│       ├── mod.rs
│       └── validation.rs     # Validation utilities
├── tests/
│   ├── integration_tests.rs
│   └── fixtures/
│       └── mock_responses.json
└── examples/
    ├── simple_client.rs
    └── claude_config.json
```

### Data Models

#### MCP Tool Schemas

```rust
// Event-related schemas
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchEventInput {
    /// Unique identifier of the event (ULID format)
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateEventInput {
    pub name: String,
    pub version: String,
    pub release: String,
    pub platform_id: String,
    pub package: String,
    pub description: String,
    pub payload: serde_json::Value,
    pub success: bool,
    pub event_receiver_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchEventsInput {
    pub name: Option<String>,
    pub version: Option<String>,
    pub platform_id: Option<String>,
    pub package: Option<String>,
    pub success: Option<bool>,
    pub event_receiver_id: Option<String>,
}

// Receiver-related schemas
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchReceiverInput {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateReceiverInput {
    pub name: String,
    #[serde(rename = "type")]
    pub receiver_type: String,
    pub version: String,
    pub description: String,
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchReceiversInput {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub receiver_type: Option<String>,
    pub version: Option<String>,
}

// Group-related schemas
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchGroupInput {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateGroupInput {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub version: String,
    pub description: String,
    pub event_receiver_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchGroupsInput {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub group_type: Option<String>,
    pub version: Option<String>,
}
```

#### XZepr Response Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// ULID identifier (26 characters, sortable by creation time)
    pub id: String,

    /// Event name (max 255 chars, required)
    pub name: String,

    /// Semantic version (e.g., "1.0.0", max 50 chars)
    pub version: String,

    /// Release identifier (max 100 chars)
    pub release: String,

    /// Platform ULID (26 characters)
    pub platform_id: String,

    /// Package name (max 255 chars)
    pub package: String,

    /// Event description (max 1000 chars, optional)
    pub description: String,

    /// Event payload (arbitrary JSON, max 64KB)
    pub payload: serde_json::Value,

    /// Success flag (true/false)
    pub success: bool,

    /// Event receiver ULID (26 characters)
    pub event_receiver_id: String,

    /// Creation timestamp (RFC-3339 format, e.g., "2025-11-07T18:12:07.982682Z")
    /// Serialized from chrono::DateTime<Utc> automatically
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventReceiver {
    /// ULID identifier (26 characters)
    pub id: String,

    /// Receiver name (max 255 chars, required)
    pub name: String,

    /// Receiver type (max 50 chars, e.g., "webhook", "kafka")
    #[serde(rename = "type")]
    pub receiver_type: String,

    /// Semantic version (e.g., "1.0.0", max 50 chars)
    pub version: String,

    /// Receiver description (max 1000 chars, optional)
    pub description: String,

    /// JSON schema for validation (JSON Schema draft-07)
    pub schema: serde_json::Value,

    /// SHA-256 fingerprint (64 hex chars)
    pub fingerprint: String,

    /// Creation timestamp (RFC-3339 format)
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventReceiverGroup {
    /// ULID identifier (26 characters)
    pub id: String,

    /// Group name (max 255 chars, required)
    pub name: String,

    /// Group type (max 50 chars, e.g., "fanout", "round_robin")
    #[serde(rename = "type")]
    pub group_type: String,

    /// Semantic version (e.g., "1.0.0", max 50 chars)
    pub version: String,

    /// Group description (max 1000 chars, optional)
    pub description: String,

    /// Whether group is enabled for processing
    pub enabled: bool,

    /// Array of receiver ULIDs (max 100 receivers per group)
    pub event_receiver_ids: Vec<String>,

    /// Creation timestamp (RFC-3339 format)
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update timestamp (RFC-3339 format)
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// SHA-256 fingerprint (64 hex chars)
    pub fingerprint: String,
}
```

**Validation Rules**:

- **ULID Format**: 26 uppercase alphanumeric characters (Crockford Base32)
- **Semantic Versions**: Must match `major.minor.patch` format (e.g., "1.0.0")
- **String Lengths**: Enforced at API level, validated before XZepr API calls
- **Timestamps**: Auto-serialized to RFC-3339 format by chrono/serde
- **JSON Payload**: Max 64KB per event, validated for well-formed JSON
- **Receiver IDs**: Must exist in XZepr system, validated on create/update

### Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// XZepr server configuration
    pub xzepr: XZeprConfig,

    /// MCP server configuration
    pub server: ServerConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// OIDC/Keycloak configuration
    pub oidc: Option<OidcConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XZeprConfig {
    /// XZepr API base URL
    pub url: String,

    /// API authentication token (JWT from Keycloak)
    pub token: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Enable API key authentication (alternative to OIDC)
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    /// Keycloak issuer URL (e.g., "https://keycloak.example.com/realms/xzepr")
    pub issuer_url: String,

    /// Client ID for token validation
    pub client_id: String,

    /// Client secret (optional, for token introspection)
    pub client_secret: Option<String>,

    /// Cache JWKS keys for validation (seconds)
    #[serde(default = "default_jwks_cache_ttl")]
    pub jwks_cache_ttl: u64,

    /// Required audience claim (REQUIRED for security)
    /// Tokens MUST be issued for this audience to be accepted
    pub audience: String,

    /// Required token scopes (e.g., ["xzepr:read", "xzepr:write"])
    #[serde(default)]
    pub required_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Global requests per second per user
    #[serde(default = "default_rate_limit_rps")]
    pub requests_per_second: u32,

    /// Burst size for rate limiter
    #[serde(default = "default_rate_limit_burst")]
    pub burst_size: u32,

    /// Per-tool rate limits (requests per minute)
    #[serde(default)]
    pub per_tool_limits: HashMap<String, u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    #[serde(default = "default_host")]
    pub host: String,

    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// MCP endpoint path
    #[serde(default = "default_mcp_path")]
    pub mcp_path: String,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Enable JSON formatting
    #[serde(default)]
    pub json: bool,
}

fn default_timeout() -> u64 { 30 }
fn default_host() -> String { "127.0.0.1".to_string() }
fn default_port() -> u16 { 8000 }
fn default_mcp_path() -> String { "/mcp".to_string() }
fn default_log_level() -> String { "info".to_string() }
fn default_jwks_cache_ttl() -> u64 { 3600 }
fn default_rate_limit_rps() -> u32 { 10 }
fn default_rate_limit_burst() -> u32 { 20 }
```

#### Configuration Files

`config/default.yaml`:

```yaml
xzepr:
  url: "http://localhost:8042"
  timeout: 30

server:
  host: "127.0.0.1"
  port: 8000
  mcp_path: "/mcp"

logging:
  level: "info"
  json: false

# OIDC configuration for Keycloak integration (REQUIRED for production)
oidc:
  issuer_url: "https://keycloak.example.com/realms/xzepr"
  client_id: "xzepr-mcp"
  jwks_cache_ttl: 3600
  audience: "xzepr-api"  # REQUIRED - tokens must be issued for this audience
  required_scopes:
    - "xzepr:read"
    - "xzepr:write"

# Rate limiting configuration
server:
  rate_limit:
    requests_per_second: 10
    burst_size: 20
    per_tool_limits:
      fetch_event: 100
      search_events: 20
      create_event: 10
```

#### Environment Variables

- `XZEPR_URL` - XZepr server URL
- `XZEPR_TOKEN` - XZepr JWT token
- `XZEPR_API_KEY` - XZepr API key
- `XZEPR_TIMEOUT` - Request timeout in seconds
- `SERVER_HOST` - MCP server host
- `SERVER_PORT` - MCP server port
- `LOG_LEVEL` - Logging level
- `LOG_JSON` - Enable JSON logging

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("XZepr API error: {0}")]
    XZeprApi(#[from] XZeprError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },

    #[error("MCP protocol error: {0}")]
    Protocol(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum XZeprError {
    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("Request timeout: {0}")]
    Timeout(String),

    #[error("HTTP error {status}: {message}")]
    Http { status: u16, message: String },

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Resource not found: {resource_type} with id {id}")]
    NotFound { resource_type: String, id: String },

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("HTTP client error: {0}")]
    Reqwest(#[from] reqwest::Error),
}
```

## Implementation Phases

### Phase 1: Foundation & Core Client

**Duration**: 1-2 weeks

#### Task 1.1: Project Setup

- Initialize Cargo workspace with proper structure
- Configure dependencies in `Cargo.toml`
- Set up CI/CD configuration (.github/workflows)
- Create Makefile with standard targets (build, test, lint, format)
- Configure linting (.yamllint.yaml, rustfmt.toml, clippy settings)
- Create initial documentation structure

**Deliverables**:

- Compilable project skeleton
- CI/CD pipeline passing
- README.md with setup instructions

**Success Criteria**:

- `cargo build` succeeds
- `cargo test` runs (even with no tests)
- `cargo clippy` passes with no warnings
- `cargo fmt --check` passes

#### Task 1.2: Configuration System

**Implementation**: `src/config/`

- Implement `Settings` struct with environment variables
- Add YAML configuration file loading
- Implement configuration validation
- Add configuration loading tests
- Document configuration options

**Deliverables**:

- `config/settings.rs` with complete configuration structures
- Environment variable parsing
- YAML file loading
- Unit tests with >80% coverage

**Success Criteria**:

- All configuration sources work (env, file, defaults)
- Validation catches invalid configurations
- Tests cover all configuration paths

#### Task 1.3: XZepr HTTP Client

**Implementation**: `src/xzepr/`

- Create `XZeprClient` struct with reqwest
- Implement authentication (JWT, API key)
- Add retry logic with exponential backoff (3 attempts, 100ms → 400ms → 1600ms)
- Implement circuit breaker (5 failures → open for 30s)
- Configure connection pooling (10-50 connections)
- Implement request/response types
- Add comprehensive error handling
- Create client tests with mock responses

**Resilience Patterns**:

```rust
pub struct XZeprClient {
    client: reqwest::Client,
    base_url: String,
    retry_policy: RetryPolicy,
    circuit_breaker: CircuitBreaker,
}

pub struct RetryPolicy {
    max_attempts: u32,           // 3
    initial_backoff_ms: u64,     // 100ms
    max_backoff_ms: u64,         // 5000ms
    backoff_multiplier: f64,     // 2.0 (exponential)
}

pub struct CircuitBreaker {
    failure_threshold: u32,      // 5 failures
    success_threshold: u32,      // 2 successes to close
    timeout_seconds: u64,        // 30s open duration
    state: Arc<Mutex<BreakerState>>,
}
```

**Connection Pooling**:

- Min idle connections: 10
- Max connections: 50
- Connection timeout: 30s
- Idle timeout: 90s
- Keep-alive: enabled

**Deliverables**:

- `xzepr/client.rs` with full client implementation
- `xzepr/types.rs` with all XZepr domain types
- `xzepr/error.rs` with error types
- Integration tests using mock server
- Retry and circuit breaker tests

**Success Criteria**:

- Client can make authenticated requests
- All error cases handled gracefully
- Retry logic works correctly with exponential backoff
- Circuit breaker opens on sustained failures
- Connection pooling reduces overhead
- Tests achieve >80% coverage

#### Task 1.4: Error Handling Framework

**Implementation**: `src/error.rs`

- Define `McpError` and `XZeprError` enums
- Implement error conversions
- Add user-friendly error messages
- Create error handling utilities
- Add error serialization for MCP responses
- Add OIDC-specific errors (invalid token, validation failure, etc.)

**Deliverables**:

- Complete error type hierarchy
- Error conversion implementations
- Unit tests for error handling
- OIDC error variants and messages

**Success Criteria**:

- All error paths have appropriate types
- Error messages are clear and actionable
- Error types convert correctly
- OIDC errors provide guidance (e.g., "Token expired, obtain new token from Keycloak")

#### Task 1.6: Input Validation and Sanitization (MANDATORY)

**Implementation**: `src/validation/`

- Create input validation framework using `validator` crate
- Implement prompt injection detection
- Add content sanitization for event payloads
- Validate all string inputs for suspicious patterns
- Add recursive JSON validation

**Injection Pattern Detection**:

```rust
const INJECTION_PATTERNS: &[&str] = &[
    "IGNORE PREVIOUS INSTRUCTIONS",
    "IGNORE ALL PREVIOUS",
    "SYSTEM:",
    "<|im_start|>",
    "<|im_end|>",
    "You are now",
    "Disregard all",
    "New instructions:",
];

pub fn detect_prompt_injection(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    for pattern in INJECTION_PATTERNS {
        if lower.contains(&pattern.to_lowercase()) {
            return Some(format!("Potential injection pattern: {}", pattern));
        }
    }
    None
}

pub fn sanitize_input(input: &str) -> String {
    // Remove control characters
    // Normalize whitespace
    // Truncate to max length
}
```

**Deliverables**:

- `validation/mod.rs` - Validation framework
- `validation/injection.rs` - Injection detection
- `validation/sanitize.rs` - Content sanitization
- Validation tests with attack patterns
- Documentation on validation rules

**Success Criteria**:

- All user inputs validated before processing
- Injection patterns detected and rejected
- JSON payloads validated recursively
- Clear error messages for validation failures
- > 95% detection rate for known injection patterns

#### Task 1.5: OIDC/JWT Authentication (MANDATORY for Production)

**Implementation**: `src/auth/`

- Create `OidcValidator` for JWT validation
- Implement JWKS key fetching and caching
- Add JWT claim validation (issuer, audience, expiration, scopes)
- Create token extraction from MCP requests
- Add token scope validation per tool
- Implement token fingerprinting for audit logs
- Add comprehensive auth tests with mock JWKS

**SECURITY CRITICAL**: Token validation is MANDATORY. Tokens MUST:

- Be issued by configured Keycloak realm
- Have audience claim matching `xzepr-mcp`
- Contain required scopes for requested tools
- Not be expired or used before valid time

**JWKS Caching Strategy**:

```rust
pub struct JwksCache {
    keys: Arc<RwLock<HashMap<String, Jwk>>>,
    last_refresh: Arc<RwLock<Instant>>,
    ttl: Duration,
}

impl JwksCache {
    async fn get_key(&self, kid: &str) -> Result<Jwk, AuthError> {
        // Check cache, refresh if expired
    }
}
```

**Token Scope Validation**:

```rust
pub struct TokenScopes {
    pub read: bool,   // xzepr:read scope
    pub write: bool,  // xzepr:write scope
}

pub fn validate_tool_permission(
    tool: &str,
    scopes: &TokenScopes,
) -> Result<(), AuthError> {
    match tool {
        "fetch_event" | "fetch_receiver" | "fetch_group"
        | "search_events" | "search_receivers" | "search_groups" => {
            if !scopes.read {
                return Err(AuthError::InsufficientScope {
                    tool: tool.to_string(),
                    required: "xzepr:read".to_string(),
                });
            }
        }
        "create_event" | "create_receiver" | "create_group" => {
            if !scopes.write {
                return Err(AuthError::InsufficientScope {
                    tool: tool.to_string(),
                    required: "xzepr:write".to_string(),
                });
            }
        }
        _ => {}
    }
    Ok(())
}
```

**Deliverables**:

- `auth/oidc.rs` - OIDC validator implementation
- `auth/jwt.rs` - JWT parsing and validation with scope extraction
- `auth/jwks.rs` - JWKS fetching and caching
- `auth/scopes.rs` - Token scope validation per tool
- Integration tests with mock Keycloak responses
- Documentation on obtaining tokens from Keycloak
- Security test suite for token validation

**Success Criteria**:

- Valid tokens pass validation
- Invalid/expired tokens rejected with clear errors
- Wrong audience tokens rejected (CRITICAL)
- Insufficient scope tokens rejected per tool
- JWKS keys cached to reduce latency (<5ms validation time)
- Token fingerprints generated for audit logging
- Tests cover all JWT validation scenarios (expired, wrong issuer, wrong audience, missing scopes)

**Note**: This task is MANDATORY for production. Token validation cannot be bypassed.

### Phase 2: MCP Protocol Implementation

**Duration**: 2-3 weeks

**Scope Note**: This phase implements 10 MCP tools with handlers. Consider splitting into sub-phases:

- **Phase 2A** (1-1.5 weeks): MCP server setup + 5 core tools (fetch/create for events, receivers, groups)
- **Phase 2B** (1-1.5 weeks): 5 search/utility tools (search operations, health check)

**Rationale**: Splitting reduces risk, enables earlier testing, and provides natural checkpoint. Sub-phases are optional if team prefers single implementation cycle.

#### Task 2.1: MCP Server Setup

**Implementation**: `src/mcp/server.rs`

- Integrate `mcp-rust-sdk`
- Set up HTTP server with axum
- Implement MCP endpoint handler
- Add health check endpoint
- Configure CORS for web clients
- Add request logging middleware

**Deliverables**:

- Running MCP server
- HTTP endpoint at `/mcp`
- Health check at `/health`
- Request/response logging

**Success Criteria**:

- Server starts without errors
- MCP clients can connect
- Health check responds correctly
- Logs capture all requests

#### Task 2.2: MCP Tool Definitions

**Implementation**: `src/mcp/tools.rs`, `src/mcp/schemas.rs`

- Define all 10 MCP tools matching Python implementation
- Create JSON schemas for tool inputs
- Implement tool metadata (descriptions, parameters)
- Add input validation schemas
- Document each tool with examples

**Tools to implement**:

1. `fetch_event` - Fetch event by ID
2. `fetch_receiver` - Fetch event receiver by ID
3. `fetch_group` - Fetch event receiver group by ID
4. `create_event` - Create new event
5. `create_receiver` - Create new event receiver
6. `create_group` - Create new event receiver group
7. `search_events` - Search events with filters
8. `search_receivers` - Search receivers with filters
9. `search_groups` - Search groups with filters
10. `health_check` - Server health status

**Deliverables**:

- All tool definitions with proper schemas
- Input/output type definitions
- Comprehensive tool documentation

**Success Criteria**:

- All tools appear in MCP tool list
- Schemas validate correctly
- Tool metadata is complete and accurate

#### Task 2.3: Tool Handler Implementation with Security Controls

**Implementation**: `src/mcp/handlers.rs`

- Implement handlers for all 10 tools
- Connect handlers to XZepr client
- Add input validation for each tool
- **Add token scope validation before execution**
- **Add input sanitization for all text fields**
- **Add output sanitization before returning to AI**
- Implement error handling and recovery
- Add request/response logging with security context
- **Track read/write operations for correlation analysis**
- Create handler tests with mocks

**Security Controls Per Handler**:

```rust
pub async fn handle_tool_request(
    tool: &str,
    input: serde_json::Value,
    auth: &AuthContext,
) -> Result<ToolResponse, McpError> {
    // 1. Validate token scope for this tool
    validate_tool_permission(tool, &auth.scopes)?;

    // 2. Validate and sanitize input
    let validated_input = validate_input(tool, input)?;

    // 3. Check rate limit
    check_rate_limit(&auth.user_id, tool).await?;

    // 4. Execute tool
    let result = execute_tool(tool, validated_input).await?;

    // 5. Sanitize output
    let sanitized = sanitize_output(result)?;

    // 6. Audit log
    audit_log(tool, &auth, "success").await;

    Ok(sanitized)
}
```

**Deliverables**:

- Complete handler implementations for all 10 tools
- Input validation and sanitization for all parameters
- Token scope validation per tool
- Output sanitization to prevent data leakage
- Request correlation tracking (read→write patterns)
- Proper error responses with security context
- Unit and integration tests including security scenarios

**Success Criteria**:

- All tools execute successfully
- Invalid inputs are rejected with clear messages
- Insufficient scope tokens rejected before execution
- XZepr API errors are properly propagated
- Suspicious patterns detected and logged
- Tests achieve >80% coverage including security tests

#### Task 2.4: Response Formatting

**Implementation**: `src/mcp/handlers.rs`

- Format XZepr responses for MCP clients
- Add JSON pretty-printing
- Include helpful context in responses
- Handle pagination for search results
- Add response streaming for large results

**Deliverables**:

- Consistent response formatting
- Human-readable JSON output
- Pagination support

**Success Criteria**:

- Responses are well-formatted
- Large result sets are handled efficiently
- Pagination works correctly

### Phase 3: Testing & Documentation

**Duration**: 1-2 weeks

#### Task 3.1: Unit Testing

**Implementation**: `src/**/*.rs` with test modules

- Write unit tests for all modules
- Achieve >80% code coverage
- Test error conditions
- Test edge cases
- Mock external dependencies

**Deliverables**:

- Unit tests in all modules
- Test coverage report
- Mock fixtures for XZepr responses

**Success Criteria**:

- `cargo test` passes all tests
- Code coverage >80%
- All public APIs tested

#### Task 3.2: Integration Testing

**Implementation**: `tests/integration_tests.rs`

- Set up test environment with mock XZepr server
- Test complete request/response cycles
- Test authentication flows
- Test error scenarios
- Test concurrent requests

**Deliverables**:

- Integration test suite
- Mock server implementation
- Test fixtures and data

**Success Criteria**:

- All integration tests pass
- Real-world scenarios covered
- Performance benchmarks established

#### Task 3.3: Documentation

**Implementation**: `docs/` following Diataxis framework

Create documentation in four categories:

**Tutorials** (`docs/tutorials/`):

- `quickstart.md` - Getting started guide
- `first_query.md` - Making your first query

**How-To Guides** (`docs/how_to/`):

- `configure_server.md` - Server configuration
- `deploy_production.md` - Production deployment
- `use_with_claude.md` - Claude Desktop integration
- `use_with_vscode.md` - VSCode integration

**Reference** (`docs/reference/`):

- `api.md` - MCP API reference
- `configuration.md` - Configuration options
- `mcp_tools.md` - Tool reference
- `cli.md` - Command-line interface

**Explanation** (`docs/explanation/`):

- `xzepr_mcp_rust_architecture.md` - This document
- `design_decisions.md` - Architecture decisions
- `mcp_protocol.md` - MCP protocol overview

**Deliverables**:

- Complete documentation set
- Code examples in documentation
- API documentation (rustdoc)

**Success Criteria**:

- All documentation categories populated
- Examples are tested and working
- rustdoc generates without warnings

#### Task 3.4: Performance Testing

**Implementation**: `benches/`

- Create benchmark suite with criterion
- Benchmark tool handlers
- Benchmark XZepr client
- Profile memory usage
- Test under load

**Deliverables**:

- Benchmark suite
- Performance baseline
- Optimization recommendations

**Success Criteria**:

- Benchmarks run successfully
- Performance targets met
- No memory leaks detected

### Phase 4: Production Readiness

**Duration**: 1 week

#### Task 4.1: Observability

**Implementation**: `src/` with tracing integration

- Add structured logging with tracing
- Implement request tracing
- Add metrics collection
- Create health check endpoints
- Add readiness/liveness probes

**Deliverables**:

- Structured JSON logging with tracing
- Prometheus metrics at `/metrics` endpoint
- Health check at `/api/v1/health`
- Liveness probe at `/api/v1/health/live`
- Readiness probe at `/api/v1/health/ready`
- Metrics collection for all tool calls
- Span creation for distributed tracing

**Success Criteria**:

- All requests logged with context
- Metrics available for monitoring
- Health checks work correctly

#### Task 4.2: CLI Implementation

**Implementation**: `src/main.rs`

- Create CLI with clap
- Add start/stop commands
- Add configuration validation command
- Add version command
- Support for multiple profiles

**Deliverables**:

- Full-featured CLI
- Help documentation
- Shell completion scripts

**Success Criteria**:

- CLI provides all needed functionality
- Help text is comprehensive
- Command structure is intuitive

#### Task 4.3: Docker & Deployment

**Implementation**: Root directory

- Create optimized Dockerfile (multi-stage build)
- Create docker-compose.yaml
- Add Kubernetes manifests
- Create deployment documentation
- Add example configurations

**Deliverables**:

- Optimized Dockerfile with <50MB final image (multi-stage, Alpine-based)
- docker-compose.yaml for local development
- Kubernetes deployment manifests (deployment, service, ingress)
- Kubernetes ConfigMap and Secret templates
- Helm chart (optional)
- Deployment guide with security best practices
- Container security scanning results

**Success Criteria**:

- Docker image builds successfully
- Container runs in orchestration
- Deployment guide is complete

#### Task 4.4: Security Hardening

**Implementation**: Throughout codebase

- Audit dependencies with cargo-audit
- Implement input sanitization
- Add rate limiting
- Secure token handling
- Add security documentation

**Deliverables**:

- Rate limiting implementation with configurable thresholds
- Security monitoring system with pattern detection
- Enhanced audit logging with security events
- Alerting integration (Prometheus, webhooks)
- Security test suite (injection, token abuse, rate limits)
- Incident response runbook
- Security hardening documentation
- Penetration test results
- OWASP API Security Top 10 compliance report

**Success Criteria**:

- Rate limiting enforced on all endpoints
- All injection patterns detected (>95% detection rate)
- Security monitoring operational with alerting
- All security tests passing
- Zero critical vulnerabilities in security audit
- Token validation cannot be bypassed
- Audit logs capture all security events
- Incident response procedures documented and tested
- OWASP API Security compliance achieved
- Penetration test findings remediated

## Key Design Decisions

### 0. Simple Modular Architecture (Not DDD Layers)

**Decision**: Use simple module structure instead of DDD layered architecture

**Rationale**:

- XZepr-MCP is a protocol adapter, not a business application
- No complex domain logic to encapsulate
- No persistence layer needed
- Layered architecture (domain/application/infrastructure) would be over-engineering
- Simpler structure is easier to understand and maintain

**Implementation**:

- Organize by technical concerns: `config/`, `client/`, `mcp/`
- Keep clear boundaries: MCP layer calls client layer, never reverse
- This differs from XZepr main server which DOES need DDD layers

### 1. StreamableHTTP Transport with rmcp

**Decision**: Use StreamableHTTP transport from `rmcp` crate for MCP protocol

**Rationale**:

- Enables remote client access (not just local stdio)
- Supports streaming responses for large datasets
- HTTP-based allows standard observability tools
- Compatible with container orchestration health checks
- Session management via `LocalSessionManager`

**Implementation**:

```rust
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpService, session::local::LocalSessionManager,
    },
};
```

**Alternatives Considered**:

- stdio transport (rejected: doesn't support remote clients)
- WebSocket transport (rejected: rmcp StreamableHTTP is standard)

### 2. ULID for Identifiers

**Decision**: Use ULID for all identifiers, matching XZepr

**Rationale**:

- Consistency with XZepr backend
- Sortable by creation time
- More compact than UUID v7
- Better distributed system properties

### 3. Error Handling Strategy

**Decision**: Use thiserror for error types with detailed variants

**Rationale**:

- Clear error messages for AI clients
- Structured error information
- Easy error propagation with `?` operator
- Type-safe error handling

### 4. Configuration Precedence

**Decision**: Environment variables > Config file > Defaults

**Rationale**:

- Container-friendly
- Matches XZepr patterns
- Standard Rust practice
- Flexible deployment

### 5. API Versioning Strategy

**Decision**: XZepr API endpoints are versioned (e.g., `/api/v1/events`), MCP tools use protocol-level versioning

**Rationale**:

- XZepr backend provides versioned REST API
- MCP protocol has its own versioning mechanism
- Health check endpoints exposed at `/api/v1/health` for consistency
- OpenAPI documentation includes version in info section

**Implementation**:

- XZepr client calls: `GET /api/v1/events/{id}`
- MCP protocol: Uses tool schema versioning
- Health checks: `GET /api/v1/health` with OpenAPI spec

### 6. OpenAPI Documentation

**Decision**: Generate OpenAPI 3.0 specification for health check and monitoring endpoints

**Rationale**:

- Required by PLAN.md for services with endpoints
- Enables automated API testing
- Provides clear contract for monitoring systems
- Supports API client generation

**Implementation**:

- Use `utoipa` crate for compile-time OpenAPI generation
- Expose spec at `/api/v1/openapi.json`
- Swagger UI at `/api/v1/docs` for development
- RapiDoc UI at `/api/v1/rapidoc` for alternative view

### 7. OIDC/Keycloak Authentication Support

**Decision**: Support OIDC tokens from Keycloak with optional local validation

**Implementation Approach**:

Since MCP clients (Claude Desktop, VSCode) are not web browsers, interactive OAuth flows are impractical. Instead:

1. **Token Acquisition** (external to MCP server):

   - Users obtain JWT tokens via Keycloak (web portal, CLI tool, or script)
   - Tokens provided to MCP client via environment variables or config
   - Tokens MUST be issued for audience `xzepr-mcp`
   - Tokens MUST include required scopes: `xzepr:read` and/or `xzepr:write`

2. **Token Validation** (MANDATORY in MCP server):

   - Download JWKS (JSON Web Key Set) from Keycloak
   - Validate JWT signature locally - **CANNOT BE BYPASSED**
   - Cache JWKS keys (default 1 hour) to reduce latency
   - Verify ALL claims: issuer, audience, expiration, scopes
   - Extract token scopes for per-tool authorization
   - Generate token fingerprint for audit logging

3. **Token Authorization**:
   - Validate token scopes before each tool execution
   - Reject requests with insufficient scopes
   - Pass validated token to XZepr API for additional checks (defense in depth)

**SECURITY CRITICAL - Token Validation is MANDATORY**:

Tokens MUST meet ALL criteria:

- Issued by configured Keycloak realm (issuer validation)
- Issued FOR xzepr-mcp audience (prevents confused deputy)
- Not expired (expiration validation)
- Contains required scopes for requested tool (authorization)

**Configuration** (validation always enabled):

```rust
// Production configuration (REQUIRED)
oidc:
  issuer_url: "https://keycloak.example.com/realms/xzepr"
  client_id: "xzepr-mcp"
  audience: "xzepr-mcp"  # REQUIRED - tokens MUST be for this audience
  required_scopes:
    - "xzepr:read"   # Required for fetch/search tools
    - "xzepr:write"  # Required for create tools
```

**Rationale**:

- **CRITICAL**: Prevents token passthrough vulnerabilities
- Supports XZepr's Keycloak integration
- Local validation catches auth errors early (better UX)
- Scope-based authorization enables least privilege
- No credential storage in MCP server (tokens are ephemeral)
- Defense in depth: MCP validates, XZepr validates again
- Compatible with MCP protocol limitations (no interactive flows)

**Token Refresh**: Users responsible for obtaining fresh tokens when expired. MCP server returns clear error messages with guidance when tokens expire or have insufficient scopes.

### 8. API Key Support (Alternative to OIDC)

**Decision**: Support both OIDC tokens and API keys for authentication

**Rationale**:

- API keys simpler for development/testing
- OIDC tokens for production environments
- Both passed through to XZepr API
- XZepr determines which authentication method to accept

### 9. Response Format

**Decision**: Return formatted JSON strings from tools

**Rationale**:

- Human-readable for AI interpretation
- Matches reference implementation patterns
- Easy debugging

### 10. Async-First Architecture

**Decision**: Use async/await throughout with tokio runtime

**Rationale**:

- Non-blocking I/O for HTTP calls
- Better resource utilization
- Consistent with XZepr
- Required by rmcp StreamableHTTP
- Scalable for concurrent requests

## Testing Strategy

### Test Coverage Requirements

- **Unit Tests**: >80% coverage for all modules
- **Integration Tests**: All API endpoints covered
- **Error Path Testing**: All error conditions tested
- **Performance Tests**: Benchmark all critical paths

### Test Structure

```
tests/
├── integration_tests.rs      # Full request/response tests
├── unit/
│   ├── client_tests.rs       # XZepr client tests
│   ├── config_tests.rs       # Configuration tests
│   └── handler_tests.rs      # Tool handler tests
├── fixtures/
│   ├── events.json           # Mock event data
│   ├── receivers.json        # Mock receiver data
│   └── groups.json           # Mock group data
└── common/
    ├── mod.rs
    └── mock_server.rs        # Mock XZepr server
```

### Test Patterns

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_fetch_event_success() {
        // Arrange
        let mut mock_client = MockXZeprClient::new();
        mock_client
            .expect_get_event()
            .with(eq("01ARZ3NDEKTSV4RRFFQ69G5FAV"))
            .times(1)
            .returning(|_| Ok(Event { /* ... */ }));

        // Act
        let result = fetch_event_handler(mock_client, "01ARZ3NDEKTSV4RRFFQ69G5FAV").await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_event_not_found() {
        // Test error case
    }

    #[tokio::test]
    async fn test_fetch_event_invalid_id() {
        // Test validation
    }
}
```

## Deployment Architecture

### Container Deployment

```yaml
# docker-compose.yaml
version: "3.8"

services:
  xzepr-mcp:
    build: .
    container_name: xzepr-mcp-server
    ports:
      - "8000:8000"
    environment:
      - XZEPR_URL=http://xzepr-server:8042
      - XZEPR_TOKEN=${XZEPR_TOKEN}
      - LOG_LEVEL=info
      - LOG_JSON=true
    depends_on:
      - xzepr-server
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  xzepr-server:
    image: xzepr/xzepr:latest
    # XZepr configuration...
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: xzepr-mcp
spec:
  replicas: 3
  selector:
    matchLabels:
      app: xzepr-mcp
  template:
    metadata:
      labels:
        app: xzepr-mcp
    spec:
      containers:
        - name: xzepr-mcp
          image: xzepr/xzepr-mcp:latest
          ports:
            - containerPort: 8000
          env:
            - name: XZEPR_URL
              value: "http://xzepr-service:8042"
            - name: XZEPR_TOKEN
              valueFrom:
                secretKeyRef:
                  name: xzepr-secrets
                  key: api-token
          livenessProbe:
            httpGet:
              path: /health
              port: 8000
            initialDelaySeconds: 30
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health
              port: 8000
            initialDelaySeconds: 5
            periodSeconds: 5
          resources:
            requests:
              memory: "128Mi"
              cpu: "100m"
            limits:
              memory: "512Mi"
              cpu: "500m"
```

## Client Integration Examples

### Claude Desktop Configuration

```json
{
  "mcpServers": {
    "xzepr": {
      "type": "http",
      "url": "http://localhost:8000/mcp",
      "env": {
        "XZEPR_TOKEN": "your-jwt-token-here"
      }
    }
  }
}
```

### VSCode Configuration

```json
{
  "mcp.servers": {
    "xzepr": {
      "type": "http",
      "url": "http://localhost:8000/mcp"
    }
  },
  "mcp.inputs": [
    {
      "type": "promptString",
      "id": "xzepr_token",
      "description": "XZepr API Token",
      "password": true
    }
  ]
}
```

### MCP Inspector

```bash
# Set environment variable
export XZEPR_TOKEN="your-jwt-token"

# Start server
cargo run

# In another terminal
npx @modelcontextprotocol/inspector
```

## Performance Targets

### Throughput

**Baseline (Without Caching)**:

- **Target**: 100 requests/second per instance
- **Tool Execution**: <200ms p95 latency for simple operations
- **Search Operations**: <500ms p95 latency
- **Limitation**: Bound by XZepr API response times

**Peak (With Phase 5 Caching)**:

- **Target**: 500+ requests/second per instance
- **Tool Execution**: <100ms p95 latency for cached operations
- **Search Operations**: <200ms p95 latency for cached searches
- **Cache Hit Rate**: 70%+ expected for typical workloads

**Note**: Peak performance targets require Phase 5 caching implementation. Baseline performance is sufficient for most use cases (Claude Desktop, VSCode extensions with single users).

**Performance Dependency**: All latency targets are bounded by XZepr API response times. The MCP server adds minimal overhead (<10ms). If XZepr API experiences high latency, MCP server latency will increase proportionally. Caching (Phase 5) mitigates this dependency.

### Resource Usage

- **CPU**: <100m baseline, burst to 500m under load
- **Memory**: <128MB base, <512MB under load
- **Network**: Minimal overhead beyond XZepr API calls (adds ~1-2KB per request)
- **Startup Time**: <5 seconds
- **Connection Pool**: 10-50 concurrent connections to XZepr API

## Security Considerations

### Authentication

**OIDC/Keycloak Integration**:

- JWT tokens issued by Keycloak validated before use
- **MANDATORY** local validation (cannot be bypassed)
- JWKS keys cached from Keycloak for signature verification
- Token validation includes: signature, issuer, **audience**, expiration, **scopes**
- **Audience validation prevents confused deputy attacks**
- **Scope validation enables least privilege access**
- Clear error messages when tokens are invalid, expired, or have insufficient scopes

**Alternative Authentication**:

- API key authentication for development/testing
- Both methods pass through to XZepr for final authorization

**Token Lifecycle**:

- Users obtain tokens externally (Keycloak web UI, CLI tool)
- Tokens provided via environment variable or config
- MCP server does not store credentials
- Token refresh is user's responsibility (not handled by MCP server)

**Security Features**:

- **Token audience validation** - prevents token passthrough vulnerabilities
- **Token scope validation** - enforces least privilege per tool
- No credential storage in MCP server (tokens are ephemeral)
- Local validation reduces attack surface and catches errors early
- Defense in depth (MCP validates, XZepr validates again)
- Token fingerprinting for audit trail correlation
- Comprehensive audit logging for all auth events
- Rate limiting prevents token abuse
- Injection detection protects against prompt attacks

### Input Validation

**Multi-Layer Validation**:

1. **Schema Validation**: All inputs validated against defined schemas
2. **ULID Format**: 26-character Crockford Base32 validation
3. **Injection Detection**: Pattern-based detection of prompt injection attempts
4. **Content Sanitization**: Remove/escape suspicious patterns
5. **JSON Payload**: Recursive validation, size limits (64KB max)
6. **Query Parameters**: Sanitization and type validation

**Injection Pattern Detection**:

The server actively detects and blocks prompt injection attempts:

- Suspicious instruction patterns ("IGNORE PREVIOUS INSTRUCTIONS")
- System message injection attempts
- Control character sequences
- Excessive repetition or length
- Recursive validation of nested JSON

**Validation Enforcement**:

- Validation occurs BEFORE any processing
- Failed validation returns 400 Bad Request with details
- Validation failures are logged for security monitoring
- Repeated validation failures trigger alerts

### Rate Limiting

**Implementation Details**:

- **Global Limit**: 10 requests/second per user (configurable)
- **Burst Allowance**: 20 requests (token bucket algorithm)
- **Per-Tool Limits**: Different limits for different operations
  - Fetch operations: 100 requests/minute
  - Search operations: 20 requests/minute (more expensive)
  - Create operations: 10 requests/minute (write operations)

**Rate Limit Headers**:

```
X-RateLimit-Limit: 600
X-RateLimit-Remaining: 543
X-RateLimit-Reset: 1699564800
Retry-After: 30
```

**Enforcement**:

- Rate limits checked before tool execution
- 429 Too Many Requests response when exceeded
- User-specific limits (not global)
- Metrics exposed for monitoring
- Aligned with XZepr API limits to prevent downstream overload

### Network Security

- HTTPS support for production
- CORS configuration for web clients
- Request size limits
- Timeout configuration

## Monitoring & Observability

### Metrics (Prometheus Format)

**Counter Metrics**:

- `xzepr_mcp_requests_total{tool, status}` - Total requests by tool and status
- `xzepr_mcp_errors_total{tool, error_type}` - Total errors by tool and type
- `xzepr_api_calls_total{endpoint, method, status}` - XZepr API calls

**Histogram Metrics**:

- `xzepr_mcp_request_duration_seconds{tool}` - Request duration by tool
- `xzepr_api_call_duration_seconds{endpoint}` - XZepr API call latency
- `xzepr_mcp_payload_size_bytes{tool, direction}` - Request/response sizes

**Gauge Metrics**:

- `xzepr_mcp_active_connections` - Current active MCP connections
- `xzepr_api_connection_pool_size` - XZepr client connection pool size
- `xzepr_api_connection_pool_idle` - Idle connections in pool
- `xzepr_mcp_circuit_breaker_state{state}` - Circuit breaker state (0=closed, 1=open, 2=half-open)

**Metrics Endpoint**: `/metrics` (Prometheus text format)

**Example Prometheus Queries**:

```promql
# Request rate by tool
rate(xzepr_mcp_requests_total[5m])

# Error rate
rate(xzepr_mcp_errors_total[5m]) / rate(xzepr_mcp_requests_total[5m])

# P95 latency by tool
histogram_quantile(0.95, rate(xzepr_mcp_request_duration_seconds_bucket[5m]))

# Circuit breaker opens
increase(xzepr_mcp_circuit_breaker_state{state="open"}[1h])
```

### Logging (Structured JSON)

**Log Format** (using `tracing` with JSON formatter):

```json
{
  "timestamp": "2025-11-07T18:12:07.982682Z",
  "level": "INFO",
  "target": "xzepr_mcp::mcp::handlers",
  "fields": {
    "message": "Handling tool request",
    "tool": "fetch_event",
    "request_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "user_id": "user_123",
    "duration_ms": 45
  },
  "span": {
    "name": "tool_request",
    "tool": "fetch_event"
  }
}
```

**Log Levels**:

- `ERROR` - Unrecoverable errors, failed requests
- `WARN` - Recoverable errors, retries, circuit breaker opens
- `INFO` - Tool requests, XZepr API calls, lifecycle events
- `DEBUG` - Request/response details, validation steps
- `TRACE` - Low-level details, connection pool state

**Key Logged Events**:

1. **Server Lifecycle**:

   - Server start/shutdown
   - Configuration loaded
   - Connection established/lost

2. **Tool Requests**:

   - Tool invocation (with params)
   - Validation failures
   - XZepr API call initiated
   - Response received
   - Request completed (with duration)

3. **Error Conditions**:
   - Authentication failures
   - XZepr API errors
   - Timeout events
   - Circuit breaker state changes
   - Retry attempts

**Contextual Fields** (added to all logs within span):

- `request_id` - ULID for request correlation
- `tool` - MCP tool name
- `user_id` - From JWT token (if available)
- `duration_ms` - Operation duration

### Logging

- Structured JSON logging
- Request/response logging with correlation IDs
- Error logging with full context
- Performance logging for slow requests

### Health Checks

- Liveness: Server process running
- Readiness: Can connect to XZepr API
- Dependency status in health response

### Tracing

- Distributed tracing with OpenTelemetry
- Request flow from MCP to XZepr
- Performance bottleneck identification

**Configuration**:

```yaml
tracing:
  enabled: true
  endpoint: "http://jaeger:14268/api/traces"
  service_name: "xzepr-mcp"
  sample_rate: 0.1
```

### Authentication Flow

**OIDC Token Acquisition** (external to MCP server):

```bash
# Option 1: Using Keycloak Direct Grant (for CLI)
TOKEN=$(curl -X POST "https://keycloak.example.com/realms/xzepr/protocol/openid-connect/token" \
  -d "client_id=xzepr-mcp" \
  -d "client_secret=<secret>" \
  -d "grant_type=password" \
  -d "username=user@example.com" \
  -d "password=<password>" \
  | jq -r '.access_token')

# Option 2: Using Keycloak web UI (copy token from browser)

# Option 3: Using custom CLI tool
xzepr-auth login --realm=xzepr
```

**MCP Client Configuration** (with token):

```json
{
  "mcpServers": {
    "xzepr": {
      "type": "http",
      "url": "http://localhost:8000/mcp",
      "env": {
        "XZEPR_TOKEN": "${TOKEN}"
      }
    }
  }
}
```

**MCP Server Validation Flow**:

```
1. Client sends MCP request with Authorization: Bearer <token>
2. MCP server extracts token
3. IF oidc.validate_locally = true:
   a. Fetch JWKS from Keycloak (cached)
   b. Verify token signature
   c. Validate claims (issuer, audience, exp)
   d. Reject if invalid with clear error
4. Forward request to XZepr API with token
5. XZepr validates token (again, defense in depth)
6. Return response to client
```

## Future Enhancements

### Phase 5: Advanced Features (Optional)

1. **Response Caching Layer** (Required for Peak Performance Targets)

   - Redis integration for response caching
   - Configurable TTL per resource type (events: 60s, receivers: 300s, groups: 300s)
   - Cache invalidation on mutations (create operations)
   - Cache-Control headers respected from XZepr
   - Metrics for cache hit/miss rates
   - **Enables**: Peak throughput of 500 req/s by reducing XZepr API load

2. **Batch Operations**

   - Bulk event creation
   - Batch queries
   - Transaction support

3. **Streaming**

   - Real-time event streaming
   - WebSocket support for updates
   - Server-sent events for notifications

4. **Advanced Search**

   - Full-text search integration
   - Aggregation queries
   - Time-range queries

5. **Client SDK**
   - Rust client library
   - Python client bindings
   - TypeScript/JavaScript client

### Graceful Shutdown

**Implementation** (Priority 3 - Phase 4):

```rust
use tokio::signal;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Signal received, starting graceful shutdown");
}
```

**Shutdown Process**:

1. Stop accepting new connections
2. Wait for active requests to complete (max 30s)
3. Close XZepr client connection pool
4. Flush metrics and logs
5. Exit process

**Kubernetes Integration**:

- Respect `SIGTERM` signal
- Honor `terminationGracePeriodSeconds: 30`
- Update readiness probe to fail during shutdown

## Architecture Note

**Important**: This architecture document describes a simple modular design appropriate for MCP servers, which are protocol adapters. This differs from the XZepr main server architecture which uses Domain-Driven Design (DDD) with layered architecture.

**Why the difference?**

- **XZepr Server**: Complex business logic, persistence, auth → needs DDD layers
- **XZepr-MCP**: Protocol translation, stateless proxy → needs simple modules

Each project type should use architecture patterns appropriate to its purpose and complexity.

## Design Decision Summary

Based on architecture review, the following key decisions were confirmed:

1. **Transport**: StreamableHTTP via `rmcp` crate for remote client support
2. **OpenAPI**: YES - health check endpoints documented with OpenAPI 3.0
3. **API Versioning**: XZepr endpoints are versioned (`/api/v1/*`), MCP uses protocol versioning
4. **Caching**: Future feature (Phase 5) - required for peak performance targets
5. **Architecture**: Simple modular design (not DDD layers) appropriate for protocol adapters

---

## References

### External Resources

- [MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [MCP Protocol Specification](https://modelcontextprotocol.io)
- [XZepr Repository](https://github.com/xbcsmith/xzepr)
- [EPR MCP Python](https://github.com/xbcsmith/epr-mcp-python)

### Internal Documentation

- [PLAN.md](../../PLAN.md) - Planning guidelines
- [AGENTS.md](../../AGENTS.md) - Development rules
- [XZepr README](../../../xzepr/README.md) - XZepr overview

## Appendices

### Appendix A: MCP Tool Reference

| Tool Name          | Operation | Input Parameters | Output                  |
| ------------------ | --------- | ---------------- | ----------------------- |
| `fetch_event`      | GET       | `id: String`     | Event JSON              |
| `fetch_receiver`   | GET       | `id: String`     | EventReceiver JSON      |
| `fetch_group`      | GET       | `id: String`     | EventReceiverGroup JSON |
| `create_event`     | POST      | Event fields     | Created event           |
| `create_receiver`  | POST      | Receiver fields  | Created receiver        |
| `create_group`     | POST      | Group fields     | Created group           |
| `search_events`    | POST      | Search filters   | Event list              |
| `search_receivers` | POST      | Search filters   | Receiver list           |
| `search_groups`    | POST      | Search filters   | Group list              |
| `health_check`     | GET       | None             | Health status           |

### Appendix B: Error Code Mapping

| XZepr Error      | HTTP Status | MCP Error             |
| ---------------- | ----------- | --------------------- |
| Not Found        | 404         | Resource not found    |
| Validation Error | 400         | Invalid input         |
| Unauthorized     | 401         | Authentication failed |
| Forbidden        | 403         | Permission denied     |
| Server Error     | 500         | Internal error        |
| Timeout          | 408         | Request timeout       |

### Appendix C: Environment Variable Reference

| Variable               | Type    | Default                  | Description                                |
| ---------------------- | ------- | ------------------------ | ------------------------------------------ |
| `XZEPR_URL`            | URL     | `http://localhost:8042`  | XZepr API base URL                         |
| `XZEPR_TOKEN`          | String  | None                     | JWT authentication token (from Keycloak)   |
| `XZEPR_API_KEY`        | String  | None                     | API key for authentication (alternative)   |
| `XZEPR_TIMEOUT`        | Integer | 30                       | Request timeout (seconds)                  |
| `SERVER_HOST`          | String  | `127.0.0.1`              | MCP server host                            |
| `SERVER_PORT`          | Integer | 8000                     | MCP server port                            |
| `LOG_LEVEL`            | String  | `info`                   | Log level (trace/debug/info/warn/error)    |
| `LOG_JSON`             | Boolean | false                    | Enable JSON logging                        |
| `OIDC_ISSUER_URL`      | URL     | None                     | Keycloak issuer URL (e.g., realm URL)      |
| `OIDC_CLIENT_ID`       | String  | None                     | OIDC client ID for token validation        |
| `OIDC_CLIENT_SECRET`   | String  | None                     | OIDC client secret (optional)              |
| `OIDC_AUDIENCE`        | String  | **REQUIRED**             | Required audience claim in JWT (MANDATORY) |
| `OIDC_REQUIRED_SCOPES` | String  | "xzepr:read,xzepr:write" | Required token scopes (comma-separated)    |
| `OIDC_JWKS_CACHE_TTL`  | Integer | 3600                     | JWKS cache TTL (seconds)                   |
| `RATE_LIMIT_RPS`       | Integer | 10                       | Global requests per second per user        |
| `RATE_LIMIT_BURST`     | Integer | 20                       | Rate limit burst size                      |

---

**Document Status**: Draft v1.0
**Last Updated**: 2025-01-07
**Author**: AI Development Team
**Review Status**: Pending technical review
