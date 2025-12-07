//! Security Tests for Injection Attack Detection
//!
//! Comprehensive security testing for SQL injection, XSS, command injection,
//! path traversal, and other injection attack vectors.

mod fixtures;

use fixtures::payloads::*;
use serde_json::json;
use xzepr_mcp::middleware::InputValidator;

#[tokio::test]
async fn test_sql_injection_detection_basic() {
    let validator = InputValidator::new();

    for payload in sql_injection::basic_payloads() {
        let result = validator.detect_sql_injection(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect SQL injection in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_sql_injection_detection_advanced() {
    let validator = InputValidator::new();

    for payload in sql_injection::advanced_payloads() {
        let result = validator.detect_sql_injection(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect advanced SQL injection in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_sql_injection_detection_blind() {
    let validator = InputValidator::new();

    for payload in sql_injection::blind_payloads() {
        let result = validator.detect_sql_injection(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect blind SQL injection in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_xss_detection_basic() {
    let validator = InputValidator::new();

    for payload in xss::basic_payloads() {
        let result = validator.detect_xss(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect XSS in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_xss_detection_encoded() {
    let validator = InputValidator::new();

    for payload in xss::encoded_payloads() {
        let result = validator.detect_xss(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect encoded XSS in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_xss_detection_event_handlers() {
    let validator = InputValidator::new();

    for payload in xss::event_handler_payloads() {
        let result = validator.detect_xss(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect event handler XSS in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_command_injection_detection_basic() {
    let validator = InputValidator::new();

    for payload in command_injection::basic_payloads() {
        let result = validator.detect_sql_injection(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect command injection on Windows: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_command_injection_detection_windows() {
    let validator = InputValidator::new();

    for payload in command_injection::windows_payloads() {
        let result = validator.detect_sql_injection(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect Windows command injection in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_path_traversal_detection_basic() {
    let validator = InputValidator::new();

    for payload in path_traversal::basic_payloads() {
        let result = validator.detect_path_traversal(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect path traversal in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_path_traversal_detection_null_byte() {
    let validator = InputValidator::new();

    for payload in path_traversal::null_byte_payloads() {
        let result = validator.detect_path_traversal(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect null byte path traversal in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_ldap_injection_detection() {
    let validator = InputValidator::new();

    for payload in ldap_injection::payloads() {
        let result = validator.detect_sql_injection(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect LDAP injection in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_header_injection_detection() {
    let validator = InputValidator::new();

    for payload in header_injection::payloads() {
        let result = validator.detect_xss(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect header injection in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_ulid_validation_with_invalid_inputs() {
    let validator = InputValidator::new();

    for payload in identifier_injection::invalid_ulids() {
        let result = validator.validate_ulid(payload).await;
        assert!(
            result.is_err(),
            "Failed to reject invalid ULID: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_ulid_validation_with_valid_input() {
    let validator = InputValidator::new();
    let valid_ulid = "01HQKZ5VJ8QXR9BZVW3TN8C6XE";

    let result = validator.validate_ulid(valid_ulid).await;
    assert!(result.is_ok(), "Valid ULID should pass validation");
}

#[tokio::test]
async fn test_semver_validation_with_invalid_inputs() {
    let validator = InputValidator::new();

    for payload in semver_injection::invalid_semvers() {
        let result = validator.validate_semver(payload).await;
        assert!(
            result.is_err(),
            "Failed to reject invalid semver: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_semver_validation_with_valid_input() {
    let validator = InputValidator::new();
    let valid_semvers = vec!["1.0.0", "2.1.3", "0.0.1", "10.20.30"];

    for semver in valid_semvers {
        let result = validator.validate_semver(semver).await;
        assert!(
            result.is_ok(),
            "Valid semver should pass validation: {}",
            semver
        );
    }
}

#[tokio::test]
async fn test_size_exhaustion_large_string() {
    let validator = InputValidator::new();
    let large_payload = size_exhaustion::large_string(10); // 10MB

    let result = validator.validate_payload_size(large_payload.len(), 1024 * 1024);
    assert!(result.is_err(), "Large string should exceed size limit");
}

#[tokio::test]
async fn test_size_exhaustion_deeply_nested_json() {
    let validator = InputValidator::new();
    let nested = size_exhaustion::deeply_nested_json(1000);

    let result = validator.validate_input_async(&nested).await;
    assert!(
        result.is_err(),
        "Deeply nested JSON should exceed depth limit"
    );
}

#[tokio::test]
async fn test_size_exhaustion_wide_json() {
    let validator = InputValidator::new();
    let wide = size_exhaustion::wide_json(10000);

    let result = validator.validate_input_async(&wide).await;
    assert!(result.is_err(), "Wide JSON should exceed key count limit");
}

#[tokio::test]
async fn test_size_exhaustion_large_array() {
    let validator = InputValidator::new();
    let large_array = size_exhaustion::large_array(100000);

    let result = validator.validate_input_async(&large_array).await;
    assert!(result.is_err(), "Large array should exceed element limit");
}

#[tokio::test]
async fn test_unicode_attack_detection() {
    let validator = InputValidator::new();

    for payload in unicode_attacks::payloads() {
        let result = validator.detect_xss(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect unicode attack in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_prototype_pollution_detection() {
    let validator = InputValidator::new();
    let payload = prototype_pollution::json_payload();

    let result = validator.validate_input_async(&payload).await;
    assert!(
        result.is_err(),
        "Failed to detect prototype pollution attempt"
    );
}

#[tokio::test]
async fn test_ssrf_detection() {
    let validator = InputValidator::new();

    for payload in ssrf::payloads() {
        let result = validator.validate_input_async(&json!(payload)).await;
        assert!(
            result.is_err(),
            "Failed to detect SSRF in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_format_string_detection() {
    let validator = InputValidator::new();

    for payload in format_string::payloads() {
        let result = validator.detect_xss(payload).await;
        assert!(
            result.is_err(),
            "Failed to detect format string attack in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_redos_detection() {
    let validator = InputValidator::new();

    for payload in redos::payloads() {
        let result = validator.validate_input_async(&json!(payload)).await;
        assert!(
            result.is_err(),
            "Failed to detect ReDoS in payload: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_event_data_validation_with_sql_injection() {
    let validator = InputValidator::new();

    for payload in sql_injection::basic_payloads() {
        let event = create_malicious_event(payload);
        let result = validator.validate_input_async(&event).await;

        assert!(
            result.is_err(),
            "Failed to detect SQL injection in event data: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_event_data_validation_with_xss() {
    let validator = InputValidator::new();

    for payload in xss::basic_payloads() {
        let event = create_malicious_event(payload);
        let result = validator.validate_input_async(&event).await;

        assert!(
            result.is_err(),
            "Failed to detect XSS in event data: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_fully_malicious_event_validation() {
    let validator = InputValidator::new();
    let payload = "'; DROP TABLE events; --";
    let event = create_fully_malicious_event(payload);

    let result = validator.validate_input_async(&event).await;
    assert!(
        result.is_err(),
        "Failed to detect injection in fully malicious event"
    );
}

#[tokio::test]
async fn test_valid_event_data_passes_validation() {
    let validator = InputValidator::new();
    let valid_event = json!({
        "name": "valid-event-name",
        "version": "1.0.0",
        "release": "production",
        "platform": "linux",
        "arch": "x86_64",
        "commit": "abc123def456"
    });

    let result = validator.validate_input_async(&valid_event).await;
    assert!(result.is_ok(), "Valid event data should pass validation");
}

#[tokio::test]
async fn test_validation_sanitizes_error_messages() {
    let validator = InputValidator::new();
    let malicious_payload = "<script>alert('XSS')</script>";

    let result = validator.detect_xss(malicious_payload).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    // Error message should not contain the malicious payload
    assert!(
        !error_msg.contains("<script>"),
        "Error message should not expose malicious payload"
    );
}

#[tokio::test]
async fn test_nested_injection_detection() {
    let validator = InputValidator::new();

    let nested_event = json!({
        "name": "test",
        "version": "1.0.0",
        "metadata": {
            "description": "'; DROP TABLE users; --",
            "tags": ["<script>alert('XSS')</script>", "normal-tag"]
        }
    });

    let result = validator.validate_input_async(&nested_event).await;
    assert!(result.is_err(), "Should detect injection in nested fields");
}

#[tokio::test]
async fn test_array_injection_detection() {
    let validator = InputValidator::new();

    let event_with_array = json!({
        "name": "test",
        "version": "1.0.0",
        "tags": [
            "normal",
            "'; DROP TABLE events; --",
            "<script>alert('XSS')</script>"
        ]
    });

    let result = validator.validate_input_async(&event_with_array).await;
    assert!(result.is_err(), "Should detect injection in array elements");
}

#[tokio::test]
async fn test_combined_attack_vectors() {
    let validator = InputValidator::new();

    // Payload combining multiple attack types
    let combined_payload = "'; DROP TABLE events; --<script>alert('XSS')</script>../../etc/passwd";
    let event = create_malicious_event(combined_payload);

    let result = validator.validate_input_async(&event).await;
    assert!(result.is_err(), "Should detect combined attack vectors");
}

#[tokio::test]
async fn test_whitelist_validation() {
    let validator = InputValidator::new();

    // Test platform field with whitelist
    let valid_platforms = vec!["linux", "darwin", "windows"];
    for platform in valid_platforms {
        let result = validator
            .validate_whitelist(platform, &["linux", "darwin", "windows"])
            .await;
        assert!(result.is_ok(), "Valid platform should pass: {}", platform);
    }

    // Test invalid platform
    let invalid_platform = "'; DROP TABLE--";
    let result = validator
        .validate_whitelist(invalid_platform, &["linux", "darwin", "windows"])
        .await;
    assert!(result.is_err(), "Invalid platform should fail validation");
}

#[tokio::test]
async fn test_alphanumeric_validation() {
    let validator = InputValidator::new();

    // Valid alphanumeric strings
    let valid_inputs = vec!["abc123", "test", "ABC", "123"];
    for input in valid_inputs {
        let result = validator.validate_alphanumeric(input).await;
        assert!(result.is_ok(), "Valid alphanumeric should pass: {}", input);
    }

    // Invalid inputs with special characters
    let invalid_inputs = vec!["abc-123", "test@test", "abc;123", "test<script>"];
    for input in invalid_inputs {
        let result = validator.validate_alphanumeric(input).await;
        assert!(
            result.is_err(),
            "Invalid alphanumeric should fail: {}",
            input
        );
    }
}

#[tokio::test]
async fn test_input_length_validation() {
    let validator = InputValidator::new();

    // Test minimum length
    let short_input = "ab";
    let result = validator.validate_length(short_input, 3, 100).await;
    assert!(result.is_err(), "Input below minimum length should fail");

    // Test maximum length
    let long_input = "a".repeat(200);
    let result = validator.validate_length(&long_input, 3, 100).await;
    assert!(result.is_err(), "Input above maximum length should fail");

    // Test valid length
    let valid_input = "valid";
    let result = validator.validate_length(valid_input, 3, 100).await;
    assert!(result.is_ok(), "Input within length bounds should pass");
}

#[tokio::test]
#[ignore = "Rate limit bypass detection not yet implemented"]
async fn test_rate_limit_bypass_attempts() {
    let validator = InputValidator::new();

    // Test payloads that might bypass rate limiting
    let bypass_attempts = vec![
        "X-Forwarded-For: 127.0.0.1",
        "X-Real-IP: 192.168.1.1",
        "X-Client-IP: 10.0.0.1",
    ];

    for attempt in bypass_attempts {
        let result = validator.detect_xss(attempt).await;
        assert!(
            result.is_err(),
            "Rate limit bypass attempt should be detected: {}",
            attempt
        );
    }
}

#[tokio::test]
async fn test_null_byte_injection_in_identifiers() {
    let validator = InputValidator::new();

    let null_byte_ulids = vec![
        "01HQKZ5VJ8QXR9BZVW3TN8C6XE\0",
        "01HQKZ5VJ8QXR9BZVW3TN8C6XE%00",
    ];

    for ulid in null_byte_ulids {
        let result = validator.validate_ulid(ulid).await;
        assert!(
            result.is_err(),
            "Null byte in ULID should be rejected: {}",
            ulid
        );
    }
}

#[tokio::test]
async fn test_case_sensitivity_bypass_attempts() {
    let validator = InputValidator::new();

    // Attackers often try case variations to bypass filters
    let case_variations = vec![
        "<ScRiPt>alert('XSS')</ScRiPt>",
        "' Or '1'='1",
        "SeLeCt * FrOm users",
    ];

    for payload in case_variations {
        let result = validator.detect_xss(payload).await;
        assert!(
            result.is_err() || validator.detect_sql_injection(payload).await.is_err(),
            "Case variation bypass should be detected: {}",
            payload
        );
    }
}

#[tokio::test]
async fn test_validation_performance_with_large_payloads() {
    use std::time::Instant;

    let validator = InputValidator::new();
    let large_event = json!({
        "name": "a".repeat(1000),
        "version": "1.0.0",
        "metadata": size_exhaustion::wide_json(100)
    });

    let start = Instant::now();
    let _result = validator.validate_input_async(&large_event).await;
    let duration = start.elapsed();

    // Validation should complete within reasonable time (1 second)
    assert!(
        duration.as_secs() < 1,
        "Validation took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_concurrent_validation_requests() {
    let validator = InputValidator::new();
    let mut tasks = Vec::new();

    for i in 0..100 {
        let validator_clone = validator.clone();
        let task = tokio::spawn(async move {
            let event = json!({
                "name": format!("test-{}", i),
                "version": "1.0.0"
            });
            validator_clone.validate_input_async(&event).await
        });
        tasks.push(task);
    }

    let mut success_count = 0;
    for task in tasks {
        if task.await.unwrap().is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(
        success_count, 100,
        "All concurrent validations should succeed"
    );
}
