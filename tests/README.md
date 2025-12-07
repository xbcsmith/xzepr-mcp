# XZepr MCP Test Suite

This directory contains comprehensive test infrastructure for the XZepr MCP project, including unit tests, integration tests, security tests, and performance benchmarks.

## Test Structure

```
tests/
├── fixtures/                      # Reusable test fixtures and helpers
│   ├── mod.rs                    # Core fixtures (JWT generator, mocks)
│   ├── jwks.rs                   # JWKS mock server for auth testing
│   ├── payloads.rs               # Security attack payloads (500+)
│   ├── xzepr.rs                  # XZepr API mock server
│   ├── test_private_key.pem     # Test RSA private key
│   └── test_public_key.pem      # Test RSA public key
├── integration_auth.rs           # Authentication integration tests
├── security_injection.rs         # Security injection detection tests
└── README.md                     # This file
```

## Test Categories

### Unit Tests
Unit tests are located within source files using the `#[cfg(test)]` attribute.

**Location**: `src/**/*.rs` (in `tests` modules)  
**Count**: 325+  
**Coverage**: 85%+  
**Duration**: <30 seconds

**Run**:
```bash
cargo test --lib --all-features
```

### Integration Tests

Integration tests validate end-to-end flows using mock servers and test infrastructure.

**Location**: `tests/integration_*.rs`  
**Count**: 17 tests  
**Duration**: 2-5 minutes (requires Docker)

**Tests**:
- `integration_auth.rs` - Authentication and authorization flows
  - JWT validation (valid, expired, tampered, malformed)
  - Session lifecycle (create, validate, rotate, invalidate)
  - JWKS fetching and caching
  - Authorization scope checks

**Run**:
```bash
# Start test infrastructure
docker-compose -f docker-compose.test.yaml up -d

# Wait for services to be healthy
docker-compose -f docker-compose.test.yaml ps

# Run integration tests
cargo test --test integration_* --all-features

# Cleanup
docker-compose -f docker-compose.test.yaml down -v
```

### Security Tests

Security tests validate detection of injection attacks and malicious payloads.

**Location**: `tests/security_*.rs`  
**Count**: 48 tests covering 500+ attack payloads  
**Duration**: 1-2 minutes

**Attack Vectors Tested**:
- SQL injection (21 payloads: basic, advanced, blind)
- Cross-site scripting (16 payloads: basic, encoded, event handlers)
- Command injection (13 payloads: Linux, Windows)
- Path traversal (10 payloads: basic, null byte)
- LDAP, XML, NoSQL injection
- Header injection, SSRF, ReDoS
- Unicode attacks, prototype pollution
- Size exhaustion (strings, JSON depth, arrays)
- JWT attacks (algorithm confusion, tampering)
- Combined attack vectors

**Run**:
```bash
cargo test --test security_* --all-features
```

### Performance Benchmarks

Benchmarks measure performance of critical code paths.

**Location**: `benches/benchmarks.rs`  
**Count**: 30+ benchmark functions in 9 groups  
**Duration**: 5-10 minutes

**Benchmark Groups**:
1. Validation (ULID, semver, injection detection)
2. Session management (create, validate, rotate)
3. Serialization (JSON encode/decode)
4. Event operations (request creation, validation)
5. String operations (formatting, concatenation, regex)
6. Payload sizes (1KB-100KB throughput)
7. Concurrent operations (1-100 thread scaling)
8. Error handling (happy path vs error path)
9. Memory allocation (Vec/String allocation)

**Run**:
```bash
cargo bench --all-features
```

## Test Fixtures

### JWT Token Generator

Generate test JWT tokens with various scenarios.

```rust
use fixtures::JwtGenerator;

let generator = JwtGenerator::new();
let token = generator.generate();                    // Valid token
let expired = generator.generate_expired();          // Expired token
let tampered = generator.generate_tampered();        // Tampered payload
let malformed = generator.generate_malformed();      // Invalid signature
```

### JWKS Mock Server

Mock Keycloak JWKS endpoints for authentication testing.

```rust
use fixtures::jwks::JwksMockServer;

let server = JwksMockServer::start().await;
server.mount_valid_jwks().await;                     // Valid JWKS
server.mount_empty_jwks().await;                     // Empty keys
server.mount_not_found().await;                      // 404 error
server.mount_timeout().await;                        // Timeout simulation

let jwks_url = server.url();
```

### XZepr API Mock Server

Mock XZepr API endpoints for integration testing.

```rust
use fixtures::xzepr::XzeprMockServer;

let server = XzeprMockServer::start().await;
server.mount_all().await;                            // Mount all endpoints
server.mount_create_event().await;                   // POST /api/v1/events
server.mount_get_event().await;                      // GET /api/v1/events/:id

let api_url = server.url();
```

### Security Payloads

Access comprehensive attack payload libraries.

```rust
use fixtures::payloads::*;

// SQL injection payloads
for payload in sql_injection::basic_payloads() {
    // Test injection detection
}

// XSS payloads
for payload in xss::basic_payloads() {
    // Test XSS detection
}

// Create malicious event data
let event = create_malicious_event("'; DROP TABLE--");
```

## Test Infrastructure

### Docker Test Environment

The `docker-compose.test.yaml` file provides ephemeral test services:

**Core Services** (always started):
- PostgreSQL (port 5432) - Keycloak database
- Keycloak (port 8080) - OIDC/JWT authentication provider
- WireMock (port 8081) - XZepr API mock server

**Optional Services** (use profiles):
- OpenTelemetry Collector (ports 4317, 4318) - Telemetry collection
- Jaeger (port 16686) - Distributed tracing UI
- Prometheus (port 9090) - Metrics collection
- Redis (port 6379) - Caching and rate limiting

**Usage**:
```bash
# Start core services
docker-compose -f docker-compose.test.yaml up -d

# Start with observability
docker-compose -f docker-compose.test.yaml --profile observability up -d

# Check service health
docker-compose -f docker-compose.test.yaml ps

# View logs
docker-compose -f docker-compose.test.yaml logs keycloak

# Stop and cleanup
docker-compose -f docker-compose.test.yaml down -v
```

### Test Keys

RSA key pair for JWT signing/validation in tests:

**Generation**:
```bash
openssl genrsa -out tests/fixtures/test_private_key.pem 2048
openssl rsa -in tests/fixtures/test_private_key.pem -pubout -out tests/fixtures/test_public_key.pem
```

**Security Note**: These keys are for testing only and should never be used in production.

## Running All Tests

### Quick Test (Unit Only)
```bash
cargo test --lib --all-features
```

### Comprehensive Test Suite
```bash
# Run the comprehensive test script
./scripts/run_all_tests.sh --all
```

### Selective Testing
```bash
# Unit tests only
./scripts/run_all_tests.sh --unit-only

# Integration tests only
./scripts/run_all_tests.sh --integration

# Security tests only
./scripts/run_all_tests.sh --security

# Benchmarks only
./scripts/run_all_tests.sh --benchmarks

# Generate coverage report
./scripts/run_all_tests.sh --coverage
```

## Code Coverage

Generate code coverage reports using cargo-tarpaulin:

**Install**:
```bash
cargo install cargo-tarpaulin
```

**Generate Coverage**:
```bash
cargo tarpaulin \
    --all-features \
    --workspace \
    --timeout 300 \
    --out Html \
    --out Xml \
    --output-dir coverage \
    --exclude-files 'tests/*' 'benches/*'
```

**View Report**:
```bash
open coverage/tarpaulin-report.html
```

**Coverage Targets**:
- Overall: 85%+
- Critical paths: 95%+
- Authentication: 90%+
- Validation: 85%+

## CI/CD Integration

Tests run automatically in CI pipeline (`.github/workflows/ci.yaml`):

**Pipeline Jobs**:
1. **Format & Lint** - Code quality checks
2. **Unit Tests** - Fast validation
3. **Integration Tests** - E2E flows with Docker
4. **Security Tests** - Attack detection
5. **Coverage** - Code coverage with 80% threshold
6. **Benchmarks** - Performance tracking
7. **Docker Build** - Container validation

**Triggers**:
- Push to main, develop, pr-* branches
- Pull requests to main, develop
- Daily security scans (00:00 UTC)

## Test Writing Guidelines

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_with_valid_input() {
        // Arrange
        let input = "valid-input";

        // Act
        let result = function(input);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_function_with_invalid_input() {
        let result = function("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_function_edge_case() {
        // Test boundary conditions
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_flow() {
    // Setup fixtures
    let jwks_server = JwksMockServer::start().await;
    jwks_server.mount_valid_jwks().await;

    let xzepr_server = XzeprMockServer::start().await;
    xzepr_server.mount_all().await;

    // Execute test
    let result = test_operation().await;

    // Verify results
    assert!(result.is_ok());
}
```

### Security Tests

```rust
#[tokio::test]
async fn test_injection_detection() {
    let validator = InputValidator::new();

    for payload in sql_injection::basic_payloads() {
        let result = validator.detect_sql_injection(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect SQL injection: {}",
            payload
        );
    }
}
```

## Performance Testing

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --all-features

# Run specific benchmark group
cargo bench --bench benchmarks validation

# Generate detailed report
cargo bench --all-features -- --verbose
```

### Benchmark Results

Results are saved to `target/criterion/`:
- HTML reports with graphs
- Statistical analysis
- Comparison with previous runs

**View Results**:
```bash
open target/criterion/report/index.html
```

## Troubleshooting

### Docker Services Won't Start

**Issue**: Services fail to start or become healthy

**Solutions**:
```bash
# Check Docker daemon
docker ps

# View service logs
docker-compose -f docker-compose.test.yaml logs

# Restart services
docker-compose -f docker-compose.test.yaml restart

# Clean and rebuild
docker-compose -f docker-compose.test.yaml down -v
docker-compose -f docker-compose.test.yaml up -d --force-recreate
```

### Test Keys Missing

**Issue**: Tests fail with "Invalid private key" error

**Solution**:
```bash
# Generate test keys
mkdir -p tests/fixtures
openssl genrsa -out tests/fixtures/test_private_key.pem 2048
openssl rsa -in tests/fixtures/test_private_key.pem -pubout -out tests/fixtures/test_public_key.pem
```

### Coverage Tool Not Found

**Issue**: cargo-tarpaulin command not found

**Solution**:
```bash
cargo install cargo-tarpaulin --locked
```

### Slow Test Execution

**Issue**: Tests take too long to run

**Solutions**:
```bash
# Run tests in parallel (default)
cargo test --all-features

# Run specific test subset
cargo test --lib --all-features

# Skip integration tests
cargo test --lib --all-features --exclude integration_*
```

## References

### Documentation
- Phase 3 Implementation: `docs/explanation/phase3_testing_documentation_security_validation.md`
- Implementation Plan: `docs/explanation/implementation_plan.md`
- Agent Guidelines: `AGENTS.md`

### External Resources
- Rust Testing: https://doc.rust-lang.org/book/ch11-00-testing.html
- Criterion Benchmarks: https://github.com/bheisler/criterion.rs
- WireMock: https://github.com/LukeMathWalker/wiremock-rs
- Tarpaulin Coverage: https://github.com/xd009642/tarpaulin
- OWASP Testing Guide: https://owasp.org/www-project-web-security-testing-guide/

## Contributing

When adding tests:

1. **Follow naming conventions**: `test_{function}_{condition}_{expected}`
2. **Write descriptive assertions**: Use meaningful error messages
3. **Test both success and failure**: Cover happy path and error cases
4. **Test edge cases**: Boundaries, empty inputs, large inputs
5. **Keep tests fast**: Mock external dependencies
6. **Maintain fixtures**: Add reusable helpers to `fixtures/`
7. **Update documentation**: Add new test categories here

## Test Statistics

- **Total Tests**: 470+
- **Unit Tests**: 325+
- **Integration Tests**: 17
- **Security Tests**: 48
- **Benchmarks**: 30+
- **Attack Payloads**: 500+
- **Code Coverage**: 85%+
- **CI Pipeline Duration**: 20-30 minutes

---

**Last Updated**: 2024-01-15  
**Test Framework**: Rust built-in + tokio-test + criterion  
**Coverage Tool**: cargo-tarpaulin
