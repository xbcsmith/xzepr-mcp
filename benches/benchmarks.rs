//! Performance Benchmarks for XZepr MCP
//!
//! Benchmark suite covering validation operations and serialization performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::json;
use std::time::Duration;
use xzepr_mcp::middleware::InputValidator;

/// Benchmark input validation performance (synchronous)
fn benchmark_validation(c: &mut Criterion) {
    let validator = InputValidator::new();

    let mut group = c.benchmark_group("validation");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark ULID validation (blocking)
    group.bench_function("ulid_valid", |b| {
        b.iter(|| {
            let ulid = black_box("01HQKZ5VJ8QXR9BZVW3TN8C6XE");
            validator.validate_ulid_blocking(ulid).ok()
        })
    });

    group.bench_function("ulid_invalid", |b| {
        b.iter(|| {
            let ulid = black_box("invalid-ulid");
            validator.validate_ulid_blocking(ulid).ok()
        })
    });

    // Benchmark semver validation (blocking)
    group.bench_function("semver_valid", |b| {
        b.iter(|| {
            let version = black_box("1.2.3");
            validator.validate_semver_blocking(version).ok()
        })
    });

    group.bench_function("semver_invalid", |b| {
        b.iter(|| {
            let version = black_box("not-a-version");
            validator.validate_semver_blocking(version).ok()
        })
    });

    // Benchmark SQL injection detection (blocking)
    group.bench_function("sql_injection_clean", |b| {
        b.iter(|| {
            let input = black_box("valid-event-name");
            validator.detect_sql_injection_blocking(input).ok()
        })
    });

    group.bench_function("sql_injection_malicious", |b| {
        b.iter(|| {
            let input = black_box("'; DROP TABLE events; --");
            validator.detect_sql_injection_blocking(input).ok()
        })
    });

    // Benchmark XSS detection (blocking)
    group.bench_function("xss_clean", |b| {
        b.iter(|| {
            let input = black_box("Clean text input");
            validator.detect_xss_blocking(input).ok()
        })
    });

    group.bench_function("xss_malicious", |b| {
        b.iter(|| {
            let input = black_box("<script>alert('XSS')</script>");
            validator.detect_xss_blocking(input).ok()
        })
    });

    // Benchmark full input validation (blocking)
    let valid_event = json!({
        "name": "test-event",
        "version": "1.0.0",
        "release": "production",
        "platform": "linux",
        "arch": "x86_64",
        "commit": "abc123"
    });

    group.bench_function("validate_input_simple", |b| {
        b.iter(|| {
            validator
                .validate_input_blocking(black_box(&valid_event))
                .ok()
        })
    });

    let nested_event = json!({
        "name": "test-event",
        "version": "1.0.0",
        "metadata": {
            "description": "Test description",
            "tags": ["tag1", "tag2", "tag3"],
            "nested": {
                "level2": {
                    "level3": "value"
                }
            }
        }
    });

    group.bench_function("validate_input_nested", |b| {
        b.iter(|| {
            validator
                .validate_input_blocking(black_box(&nested_event))
                .ok()
        })
    });

    group.finish();
}

/// Benchmark JSON serialization/deserialization
fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    group.measurement_time(Duration::from_secs(10));

    let event_data = json!({
        "id": "01HQKZ5VJ8QXR9BZVW3TN8C6XE",
        "name": "test-event",
        "version": "1.0.0",
        "release": "production",
        "platform": "linux",
        "arch": "x86_64",
        "commit": "abc123def456",
        "timestamp": "2024-01-15T10:30:00Z"
    });

    // Benchmark serialization
    group.bench_function("serialize_event", |b| {
        b.iter(|| serde_json::to_string(black_box(&event_data)).ok())
    });

    // Benchmark deserialization
    let json_string = serde_json::to_string(&event_data).unwrap();
    group.bench_function("deserialize_event", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(&json_string)).ok())
    });

    // Benchmark with nested structure
    let nested_event = json!({
        "id": "01HQKZ5VJ8QXR9BZVW3TN8C6XE",
        "name": "test-event",
        "version": "1.0.0",
        "metadata": {
            "tags": ["tag1", "tag2", "tag3", "tag4", "tag5"],
            "properties": {
                "prop1": "value1",
                "prop2": "value2",
                "prop3": "value3"
            },
            "nested": {
                "level2": {
                    "level3": {
                        "data": "deep value"
                    }
                }
            }
        }
    });

    group.bench_function("serialize_nested", |b| {
        b.iter(|| serde_json::to_string(black_box(&nested_event)).ok())
    });

    let nested_json_string = serde_json::to_string(&nested_event).unwrap();
    group.bench_function("deserialize_nested", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(&nested_json_string)).ok())
    });

    group.finish();
}

/// Benchmark event data creation
fn benchmark_event_operations(c: &mut Criterion) {
    let validator = InputValidator::new();

    let mut group = c.benchmark_group("events");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark creating event request as JSON
    group.bench_function("create_request", |b| {
        b.iter(|| {
            json!({
                "name": black_box("test-event"),
                "version": black_box("1.0.0"),
                "release": black_box("production"),
                "platform": black_box("linux"),
                "arch": black_box("x86_64"),
                "commit": black_box("abc123"),
            })
        })
    });

    // Benchmark end-to-end event validation
    let event_json = json!({
        "name": "benchmark-event",
        "version": "1.0.0",
        "release": "production",
        "platform": "linux",
        "arch": "x86_64",
        "commit": "abc123"
    });

    group.bench_function("validate_event_end_to_end", |b| {
        b.iter(|| {
            validator
                .validate_input_blocking(black_box(&event_json))
                .ok()
        })
    });

    group.finish();
}

/// Benchmark string operations
fn benchmark_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("strings");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark string allocation and formatting
    group.bench_function("format_event_id", |b| {
        b.iter(|| {
            format!(
                "Event: {} - Version: {} - Platform: {}",
                black_box("test-event"),
                black_box("1.0.0"),
                black_box("linux")
            )
        })
    });

    // Benchmark string concatenation
    group.bench_function("concat_strings", |b| {
        b.iter(|| {
            let mut result = String::new();
            result.push_str(black_box("name="));
            result.push_str(black_box("test"));
            result.push_str(black_box("&version="));
            result.push_str(black_box("1.0.0"));
            result
        })
    });

    // Benchmark regex matching (for validation)
    let ulid_pattern = regex::Regex::new(r"^[0-9A-Z]{26}$").unwrap();
    group.bench_function("regex_ulid_match", |b| {
        b.iter(|| ulid_pattern.is_match(black_box("01HQKZ5VJ8QXR9BZVW3TN8C6XE")))
    });

    let semver_pattern = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
    group.bench_function("regex_semver_match", |b| {
        b.iter(|| semver_pattern.is_match(black_box("1.2.3")))
    });

    group.finish();
}

/// Benchmark payload sizes
fn benchmark_payload_sizes(c: &mut Criterion) {
    let validator = InputValidator::new();

    let mut group = c.benchmark_group("payload_sizes");
    group.measurement_time(Duration::from_secs(10));

    // Test different payload sizes
    for size_kb in [1, 10, 50].iter() {
        let payload_size = size_kb * 1024;
        group.throughput(Throughput::Bytes(payload_size as u64));

        let large_string = "a".repeat(payload_size);
        let event = json!({
            "name": "test",
            "version": "1.0.0",
            "data": large_string
        });

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}KB", size_kb)),
            &event,
            |b, event| b.iter(|| validator.validate_input_blocking(black_box(event)).ok()),
        );
    }

    group.finish();
}

/// Benchmark error handling paths
fn benchmark_error_handling(c: &mut Criterion) {
    let validator = InputValidator::new();

    let mut group = c.benchmark_group("error_handling");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark successful validation (happy path)
    let valid_event = json!({"name": "test", "version": "1.0.0"});
    group.bench_function("happy_path", |b| {
        b.iter(|| {
            validator
                .validate_input_blocking(black_box(&valid_event))
                .ok()
        })
    });

    // Benchmark validation errors (error path)
    let invalid_event = json!({"name": "'; DROP TABLE--", "version": "invalid"});
    group.bench_function("error_path", |b| {
        b.iter(|| {
            validator
                .validate_input_blocking(black_box(&invalid_event))
                .ok()
        })
    });

    group.finish();
}

/// Benchmark memory allocations
fn benchmark_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocations");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark Vec allocations
    group.bench_function("vec_with_capacity", |b| {
        b.iter(|| {
            let mut vec = Vec::with_capacity(black_box(100));
            for i in 0..100 {
                vec.push(i);
            }
            vec
        })
    });

    group.bench_function("vec_without_capacity", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..black_box(100) {
                vec.push(i);
            }
            vec
        })
    });

    // Benchmark String allocations
    group.bench_function("string_with_capacity", |b| {
        b.iter(|| {
            let mut s = String::with_capacity(black_box(1000));
            s.push_str("test data");
            s
        })
    });

    group.bench_function("string_without_capacity", |b| {
        b.iter(|| {
            let mut s = String::new();
            s.push_str(black_box("test data"));
            s
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_validation,
    benchmark_serialization,
    benchmark_event_operations,
    benchmark_string_operations,
    benchmark_payload_sizes,
    benchmark_error_handling,
    benchmark_allocations
);

criterion_main!(benches);
