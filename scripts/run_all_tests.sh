#!/usr/bin/env bash
# Comprehensive Test Runner for XZepr MCP Phase 3
#
# This script runs all test suites in the correct order with proper setup
# and teardown. It validates the complete testing infrastructure.
#
# Usage:
#   ./scripts/run_all_tests.sh [OPTIONS]
#
# Options:
#   --unit-only       Run only unit tests
#   --integration     Run integration tests (requires Docker)
#   --security        Run security tests
#   --benchmarks      Run performance benchmarks
#   --coverage        Generate code coverage report
#   --all             Run all tests (default)
#   --no-docker       Skip tests requiring Docker
#   --verbose         Show detailed output
#   --help            Show this help message

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TEST_FIXTURES_DIR="${PROJECT_ROOT}/tests/fixtures"
DOCKER_COMPOSE_FILE="${PROJECT_ROOT}/docker-compose.test.yaml"

# Flags
RUN_UNIT=false
RUN_INTEGRATION=false
RUN_SECURITY=false
RUN_BENCHMARKS=false
RUN_COVERAGE=false
RUN_ALL=true
SKIP_DOCKER=false
VERBOSE=false

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --unit-only)
                RUN_UNIT=true
                RUN_ALL=false
                shift
                ;;
            --integration)
                RUN_INTEGRATION=true
                RUN_ALL=false
                shift
                ;;
            --security)
                RUN_SECURITY=true
                RUN_ALL=false
                shift
                ;;
            --benchmarks)
                RUN_BENCHMARKS=true
                RUN_ALL=false
                shift
                ;;
            --coverage)
                RUN_COVERAGE=true
                RUN_ALL=false
                shift
                ;;
            --all)
                RUN_ALL=true
                shift
                ;;
            --no-docker)
                SKIP_DOCKER=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                show_help
                exit 1
                ;;
        esac
    done

    # If RUN_ALL is true, enable all test suites
    if [[ "$RUN_ALL" == true ]]; then
        RUN_UNIT=true
        RUN_INTEGRATION=true
        RUN_SECURITY=true
        RUN_BENCHMARKS=true
        RUN_COVERAGE=true
    fi

    # If Docker is disabled, skip integration tests
    if [[ "$SKIP_DOCKER" == true ]]; then
        RUN_INTEGRATION=false
    fi
}

# Show help message
show_help() {
    cat << EOF
Comprehensive Test Runner for XZepr MCP Phase 3

Usage: $0 [OPTIONS]

Options:
    --unit-only       Run only unit tests (fast, no dependencies)
    --integration     Run integration tests (requires Docker)
    --security        Run security attack simulation tests
    --benchmarks      Run performance benchmarks
    --coverage        Generate code coverage report
    --all             Run all tests (default)
    --no-docker       Skip tests requiring Docker
    --verbose         Show detailed output
    --help            Show this help message

Examples:
    $0                              # Run all tests
    $0 --unit-only                  # Quick unit tests only
    $0 --integration --security     # Integration and security tests
    $0 --coverage                   # Generate coverage report
    $0 --all --no-docker            # All tests except integration

Environment Variables:
    RUST_LOG          Set log level (e.g., debug, info)
    RUST_BACKTRACE    Enable backtrace (0, 1, full)
EOF
}

# Print colored message
print_msg() {
    local color=$1
    local msg=$2
    echo -e "${color}${msg}${NC}"
}

# Print section header
print_header() {
    echo ""
    print_msg "$BLUE" "=========================================="
    print_msg "$BLUE" "$1"
    print_msg "$BLUE" "=========================================="
    echo ""
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"

    # Check Rust
    if ! command_exists cargo; then
        print_msg "$RED" "Error: cargo not found. Please install Rust."
        exit 1
    fi
    print_msg "$GREEN" "✓ Rust/Cargo found: $(cargo --version)"

    # Check Docker (if needed)
    if [[ "$RUN_INTEGRATION" == true ]] && [[ "$SKIP_DOCKER" == false ]]; then
        if ! command_exists docker; then
            print_msg "$RED" "Error: docker not found. Install Docker or use --no-docker flag."
            exit 1
        fi
        print_msg "$GREEN" "✓ Docker found: $(docker --version)"

        if ! command_exists docker-compose; then
            print_msg "$YELLOW" "Warning: docker-compose not found. Using 'docker compose' plugin."
        fi
    fi

    # Check OpenSSL
    if ! command_exists openssl; then
        print_msg "$RED" "Error: openssl not found. Required for generating test keys."
        exit 1
    fi
    print_msg "$GREEN" "✓ OpenSSL found: $(openssl version)"

    # Check coverage tool (if needed)
    if [[ "$RUN_COVERAGE" == true ]]; then
        if ! command_exists cargo-tarpaulin; then
            print_msg "$YELLOW" "Warning: cargo-tarpaulin not found. Installing..."
            cargo install cargo-tarpaulin --locked || {
                print_msg "$RED" "Failed to install cargo-tarpaulin"
                exit 1
            }
        fi
        print_msg "$GREEN" "✓ cargo-tarpaulin found"
    fi

    print_msg "$GREEN" "All prerequisites satisfied"
}

# Setup test environment
setup_test_environment() {
    print_header "Setting Up Test Environment"

    cd "$PROJECT_ROOT"

    # Create test fixtures directory
    mkdir -p "$TEST_FIXTURES_DIR"
    mkdir -p "${TEST_FIXTURES_DIR}/wiremock/mappings"
    mkdir -p "${TEST_FIXTURES_DIR}/wiremock/__files"

    # Generate test RSA keys if they don't exist
    if [[ ! -f "${TEST_FIXTURES_DIR}/test_private_key.pem" ]]; then
        print_msg "$YELLOW" "Generating test RSA keys..."
        openssl genrsa -out "${TEST_FIXTURES_DIR}/test_private_key.pem" 2048 2>/dev/null
        openssl rsa -in "${TEST_FIXTURES_DIR}/test_private_key.pem" \
            -pubout -out "${TEST_FIXTURES_DIR}/test_public_key.pem" 2>/dev/null
        print_msg "$GREEN" "✓ Test RSA keys generated"
    else
        print_msg "$GREEN" "✓ Test RSA keys already exist"
    fi

    # Format code
    print_msg "$YELLOW" "Running cargo fmt..."
    cargo fmt --all

    print_msg "$GREEN" "Test environment ready"
}

# Start Docker test infrastructure
start_docker_services() {
    if [[ "$SKIP_DOCKER" == true ]]; then
        return 0
    fi

    print_header "Starting Docker Test Infrastructure"

    cd "$PROJECT_ROOT"

    # Check if services are already running
    if docker-compose -f "$DOCKER_COMPOSE_FILE" ps | grep -q "Up"; then
        print_msg "$YELLOW" "Docker services already running. Stopping first..."
        docker-compose -f "$DOCKER_COMPOSE_FILE" down -v
    fi

    # Start services
    print_msg "$YELLOW" "Starting Docker services..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" up -d postgres keycloak xzepr-mock

    # Wait for services to be healthy
    print_msg "$YELLOW" "Waiting for services to be healthy (timeout: 120s)..."
    local timeout=120
    local elapsed=0
    while [[ $elapsed -lt $timeout ]]; do
        if docker-compose -f "$DOCKER_COMPOSE_FILE" ps | grep -q "healthy"; then
            print_msg "$GREEN" "✓ Docker services are healthy"
            return 0
        fi
        sleep 5
        elapsed=$((elapsed + 5))
        echo -n "."
    done

    print_msg "$RED" "Error: Docker services did not become healthy within ${timeout}s"
    docker-compose -f "$DOCKER_COMPOSE_FILE" logs
    return 1
}

# Stop Docker test infrastructure
stop_docker_services() {
    if [[ "$SKIP_DOCKER" == true ]]; then
        return 0
    fi

    print_header "Stopping Docker Test Infrastructure"

    cd "$PROJECT_ROOT"
    docker-compose -f "$DOCKER_COMPOSE_FILE" down -v
    print_msg "$GREEN" "✓ Docker services stopped"
}

# Run unit tests
run_unit_tests() {
    print_header "Running Unit Tests"

    cd "$PROJECT_ROOT"

    local cargo_args="test --lib --all-features"
    if [[ "$VERBOSE" == true ]]; then
        cargo_args="$cargo_args --verbose"
    fi

    print_msg "$YELLOW" "Running: cargo $cargo_args"

    if cargo $cargo_args; then
        print_msg "$GREEN" "✓ Unit tests passed"
        return 0
    else
        print_msg "$RED" "✗ Unit tests failed"
        return 1
    fi
}

# Run integration tests
run_integration_tests() {
    print_header "Running Integration Tests"

    cd "$PROJECT_ROOT"

    local cargo_args="test --test integration_* --all-features"
    if [[ "$VERBOSE" == true ]]; then
        cargo_args="$cargo_args --verbose -- --nocapture"
    fi

    export TEST_KEYCLOAK_URL="http://localhost:8080"
    export TEST_XZEPR_URL="http://localhost:8081"

    print_msg "$YELLOW" "Running: cargo $cargo_args"

    if cargo $cargo_args; then
        print_msg "$GREEN" "✓ Integration tests passed"
        return 0
    else
        print_msg "$RED" "✗ Integration tests failed"
        return 1
    fi
}

# Run security tests
run_security_tests() {
    print_header "Running Security Tests"

    cd "$PROJECT_ROOT"

    local cargo_args="test --test security_* --all-features"
    if [[ "$VERBOSE" == true ]]; then
        cargo_args="$cargo_args --verbose -- --nocapture"
    fi

    print_msg "$YELLOW" "Running: cargo $cargo_args"

    if cargo $cargo_args; then
        print_msg "$GREEN" "✓ Security tests passed"
        return 0
    else
        print_msg "$RED" "✗ Security tests failed"
        return 1
    fi
}

# Run performance benchmarks
run_benchmarks() {
    print_header "Running Performance Benchmarks"

    cd "$PROJECT_ROOT"

    print_msg "$YELLOW" "Running: cargo bench --all-features"

    if cargo bench --all-features; then
        print_msg "$GREEN" "✓ Benchmarks completed"
        print_msg "$BLUE" "Results saved to target/criterion/"
        return 0
    else
        print_msg "$RED" "✗ Benchmarks failed"
        return 1
    fi
}

# Generate coverage report
generate_coverage() {
    print_header "Generating Code Coverage Report"

    cd "$PROJECT_ROOT"

    print_msg "$YELLOW" "Running cargo-tarpaulin (this may take several minutes)..."

    if cargo tarpaulin \
        --all-features \
        --workspace \
        --timeout 300 \
        --out Html \
        --out Xml \
        --output-dir coverage \
        --exclude-files 'tests/*' 'benches/*'; then

        print_msg "$GREEN" "✓ Coverage report generated"
        print_msg "$BLUE" "HTML report: coverage/tarpaulin-report.html"
        print_msg "$BLUE" "XML report: coverage/cobertura.xml"

        # Extract coverage percentage
        if [[ -f "coverage/cobertura.xml" ]]; then
            local coverage=$(grep -oP 'line-rate="\K[0-9.]+' coverage/cobertura.xml | head -1)
            local coverage_percent=$(echo "$coverage * 100" | bc)
            print_msg "$BLUE" "Coverage: ${coverage_percent}%"

            if (( $(echo "$coverage_percent < 80" | bc -l) )); then
                print_msg "$YELLOW" "Warning: Coverage ${coverage_percent}% is below 80% threshold"
            else
                print_msg "$GREEN" "✓ Coverage ${coverage_percent}% meets 80% threshold"
            fi
        fi
        return 0
    else
        print_msg "$RED" "✗ Coverage generation failed"
        return 1
    fi
}

# Run quality checks
run_quality_checks() {
    print_header "Running Quality Checks"

    cd "$PROJECT_ROOT"

    # Check compilation
    print_msg "$YELLOW" "Checking compilation..."
    if cargo check --all-targets --all-features; then
        print_msg "$GREEN" "✓ Compilation check passed"
    else
        print_msg "$RED" "✗ Compilation check failed"
        return 1
    fi

    # Run clippy
    print_msg "$YELLOW" "Running clippy..."
    if cargo clippy --all-targets --all-features 2>&1 | tee /tmp/clippy-output.txt; then
        print_msg "$GREEN" "✓ Clippy check passed"
    else
        print_msg "$YELLOW" "⚠ Clippy warnings found (not blocking)"
    fi

    return 0
}

# Main execution
main() {
    local start_time=$(date +%s)
    local exit_code=0

    parse_args "$@"

    print_header "XZepr MCP Phase 3 Test Runner"
    print_msg "$BLUE" "Project: XZepr MCP"
    print_msg "$BLUE" "Test Suites: Unit=$RUN_UNIT, Integration=$RUN_INTEGRATION, Security=$RUN_SECURITY"
    print_msg "$BLUE" "Benchmarks=$RUN_BENCHMARKS, Coverage=$RUN_COVERAGE"
    echo ""

    # Check prerequisites
    check_prerequisites || exit 1

    # Setup test environment
    setup_test_environment || exit 1

    # Run quality checks first
    run_quality_checks || exit_code=1

    # Start Docker if needed
    if [[ "$RUN_INTEGRATION" == true ]]; then
        start_docker_services || {
            exit_code=1
            stop_docker_services
            exit $exit_code
        }
    fi

    # Run test suites
    if [[ "$RUN_UNIT" == true ]]; then
        run_unit_tests || exit_code=1
    fi

    if [[ "$RUN_INTEGRATION" == true ]]; then
        run_integration_tests || exit_code=1
    fi

    if [[ "$RUN_SECURITY" == true ]]; then
        run_security_tests || exit_code=1
    fi

    if [[ "$RUN_BENCHMARKS" == true ]]; then
        run_benchmarks || exit_code=1
    fi

    if [[ "$RUN_COVERAGE" == true ]]; then
        generate_coverage || exit_code=1
    fi

    # Stop Docker services
    if [[ "$RUN_INTEGRATION" == true ]]; then
        stop_docker_services
    fi

    # Print summary
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    print_header "Test Summary"
    print_msg "$BLUE" "Duration: ${duration}s"

    if [[ $exit_code -eq 0 ]]; then
        print_msg "$GREEN" "✓ All tests passed!"
    else
        print_msg "$RED" "✗ Some tests failed"
    fi

    exit $exit_code
}

# Run main function
main "$@"
