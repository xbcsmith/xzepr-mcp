xzepr-mcp/docs/explanation/security_detection_implementation.md#L1-300
# Security Detection Implementation

## Overview

This document describes the security detection improvements included in the `xzepr-mcp` project. These enhancements strengthen the InputValidator logic to detect and reject malicious payloads such as SQL injection, XSS, SSRF, path traversal, ReDoS (regex DoS), command/LDAP injection, Unicode homoglyph attacks, prototype pollution attempts, and oversized/wide JSON payloads.

Key design goals:
- Keep input validation fast and deterministic (use iterative traversal instead of recursion).
- Make detection rules configurable via `Settings.security`.
- Make detection comprehensive for typical malicious payloads while limiting false positives.
- Avoid creating nested runtimes; prefer async validator entrypoints for async contexts.

## Components Delivered

- `src/middleware/validation.rs` (major changes) - Added detection functions and iterative JSON validator:
  - `InputValidator` improvements
  - `detect_sql_injection`, `detect_xss`, `detect_path_traversal`
  - `detect_ssrf`, `detect_redos`
  - Additional heuristics for LDAP and command injection, header injection, format strings, and unicode homoglyphs
  - Iterative `validate_input_iter` function enforcing depth/width/size limits

- `src/config/settings.rs` – Security configuration expanded:
  - New config fields: `max_json_keys`, `max_array_len`, `max_string_len`, `enable_ssrf_detection`
  - Default values and YAML updated accordingly.

- `config/development.yaml` and `config/production.yaml`:
  - Security keys were added/updated with sensible defaults.

- Tests:
  - `tests/security_injection.rs` – un-ignored & validated several tests after implementing detection logic.
  - `tests/fixtures/payloads.rs` – (unchanged) test payloads used for detection verification.

- Documentation:
  - `docs/explanation/security_detection_implementation.md` ← (this file)
  - Doc comments added to public functions for clarity and examples where applicable.

## Implementation Details

### Architectural Changes

- Iterative Validation
  - Replaced recursive walker for JSON with an iterative stack-based traversal to track depth, key count, array length, and perform detection checks without stack overflows or recursion depth issues.
  - This makes it easier to enforce limits (max depth, max keys, max array length) from `SecurityConfig`.

- Async Safety
  - Added `validate_input_iter(args)` as an asynchronous implementation.
  - Synchronous wrappers (for backward compatibility) now use a dedicated single-thread runtime helper (`tokio_block_on`) to avoid nested Tokio runtime panics.
  - Handlers running inside an async runtime should call `validate_input_async(...).await` directly.

### Security Detection Techniques

- SQL Injection & Command Injection
  - `detect_sql_injection` includes:
    - SQL keywords detection (case-insensitive).
    - Character-based injection patterns (quotes, comment markers).
    - boolean-based patterns such as " or " and " AND " heuristics.
    - Additional shell/command patterns (e.g., `;`, `|`, `&`, backticks, `$(`) to detect command injection attempts when strings contain shell metacharacters.
    - LDAP-style checks (wildcard usage `admin*`, suspicious parentheses sequences) to catch LDAP injection.

- XSS & Format String & Header Injection
  - `detect_xss` now checks:
    - Dangerous tags (e.g., `<script`, `<iframe`, `<svg`, `<img`, `<input`, etc.)
    - Event handlers (`onclick=`, `onerror=` etc.)
    - Dangerous protocols (`javascript:`, `vbscript:`)
    - Encoded/escaped script patterns (e.g., `&lt;script`, `%3cscript`)
    - CRLF / header injection patterns (`\r\n`, `%0d%0a`)
    - Format specifiers (`%s`, `%n`, `%p`, ...) to detect format-string style payloads that might be used in template injection.
    - Unicode directionality characters (RLO, LRO), zero-width characters, null byte, and Unicode homoglyph ranges (enclosed alphanumerics, mathematical alphanumeric symbols, fullwidth characters, Cyrillic) to detect obfuscation.

- Path Traversal Detection
  - `detect_path_traversal` includes:
    - Common patterns such as `../`, `..\\`, `%2e%2e%2f`, and URL-encoded representations and overlong encodings.
    - Null byte checks (e.g., `%00`, `\0`) as well.

- SSRF Detection
  - `detect_ssrf` checks for access to local/internal resources and known metadata endpoints (`localhost`, `127.0.0.1`, `169.254.169.254`, `metadata.google.internal`, `file://`, `gopher://`).

- ReDoS (Regex DoS) Heuristics
  - `detect_redos` uses lightweight heuristics:
    - Detect nested quantifiers and alternation/`*` sequences (e.g., `(a+)+`, `(a|a)*`) via Regex heuristics.
    - Flag suspiciously long runs of repeated characters (heuristic threshold lowered to detect typical malicious payload lengths).
  - Note: The current heuristics flag obvious ReDoS vectors; thorough ReDoS protection would also include analyzing untrusted regex patterns and limiting runtime interpretation or adding execution limits.

- Prototype Pollution
  - The iterative traversal disallows object keys like `__proto__`, `constructor.prototype`, or any key that contains `.prototype` or `__proto__` to prevent direct pollution attempts.

- Size Exhaustion / Depth / Wide JSON checks
  - The iterative traversal enforces:
    - `SecurityConfig.max_json_depth` default 32
    - `SecurityConfig.max_json_keys` default 1024
    - `SecurityConfig.max_array_len` default 10000
    - String payload maximum via `SecurityConfig.max_string_len` default 1MB

### Configuration and Defaults

Updated `SecurityConfig` (in `src/config/settings.rs`) includes:
- `max_json_depth: usize` (default: 32)
- `max_json_keys: usize` (default: 1024)
- `max_array_len: usize` (default: 10000)
- `max_string_len: usize` (default: 1_048_576)
- `enable_sql_injection_detection: bool` (unchanged, default: true)
- `enable_xss_detection: bool` (unchanged)
- `enable_path_traversal_detection: bool` (unchanged)
- `enable_ssrf_detection: bool` (new, default: true)

Update `config/development.yaml` and `config/production.yaml` to include the new fields and defaults.

### Important Snippets (Examples)

Input validator basic usage:
```xzepr-mcp/src/middleware/validation.rs#L880-958
(Box::pin(...) pseudo snippet showing the asynchronous validate_input_iter approach)
(pub async fn validate_input_iter(&self, input: &serde_json::Value) -> Result<()> { ... })
```

Synchronous wrapper (legacy-compatible):
```xzepr-mcp/src/middleware/validation.rs#L868-889
pub fn validate_input(&self, input: &serde_json::Value) -> Result<()> {
    // Uses tokio_block_on to avoid nested runtime creation issues from within async contexts
    tokio_block_on(self.validate_input_iter(input))
}
```

Example usage:
```xzepr-mcp/src/middleware/validation.rs#L92-116
let validator = xzepr_mcp::middleware::InputValidator::new();
let json = serde_json::json!({"name":"<script>alert('XSS')</script>"});
let validation_result = tokio::runtime::Runtime::new()?.block_on(validator.validate_input_async(&json)); // prefer await in async context
```

## Testing

- The suite was verified with:
  - `cargo fmt --all`
  - `cargo check --all-targets --all-features`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-features`

Results (after changes):
- Unit tests: 95 passed; 0 failed; 2 ignored
- Integration tests (auth): 48 passed; 0 failed
- Security injection suite: 71 passed; 0 failed; 1 ignored (Rate limit bypass detection still unimplemented)
- Clippy: final run passed with zero warnings.

Note: Some tests were temporarily ignored in earlier iterations and were un-ignored after implementation of detection logic. The test harness includes both unit tests and integration tests with mock JWKS/XZepr servers.

## Examples and How to Run

- Quick start (developer machine):
  - Format and lint:
    - `cargo fmt --all`
    - `cargo check --all-targets --all-features`
    - `cargo clippy --all-targets --all-features -- -D warnings`
  - Run all tests: `cargo test --all-features`

- Example (validate JSON):
  ```xzepr-mcp/src/middleware/validation.rs#L880-910
  use xzepr_mcp::middleware::InputValidator;
  use serde_json::json;

  #[tokio::main]
  async fn main() -> Result<()> {
      let validator = InputValidator::new();
      let payload = json!({"name": "<script>alert('XSS')</script>"});
      let result = validator.validate_input_async(&payload).await;
      assert!(result.is_err()); // Detected XSS
      Ok(())
  }
  ```

## Testing & Validation Results

- The security tests used the fixture payloads in `tests/fixtures/payloads.rs` and validated them against the detection logic.
- We verified that the algorithms detect a wide variety of test vectors:
  - SQL injection (basic/advanced/blind)
  - Command injection (Unix & Windows patterns)
  - LDAP wildcards `admin*`
  - XSS (tags, events, protocols, encoded forms)
  - ReDoS heuristics
  - SSRF and metadata endpoint attempts
  - Prototype pollution (e.g., `__proto__`)
  - ULID invalid formats and obviously invalid sentinel values

## Caveats & Notes

- These detection heuristics are *conservative* and prioritized for rejecting common attack patterns. Some benign content may trigger heuristics; if that occurs, tune `Settings.security` in the environment configuration:
  - Pick appropriate values for `max_json_depth`, `max_json_keys`, `max_array_len`, and `max_string_len`.
  - Toggle detection via boolean flags where available (e.g., `enable_xss_detection`, `enable_sql_injection_detection`).
- `detect_redos` heuristics are intentionally conservative; a production-grade ReDoS mitigations need to combine regex runtime timeouts, sandboxing scoring, and disallow untrusted regexes from untrusted contexts.
- Prototype Pollution: Detection only rejects suspicious keys; it does not attempt to deep-scan or instrument arbitrary JS objects as this tool is a thin cross-protocol adapter.

## Next Steps & Improvements

- Implement advanced detection features that were out of scope / intentionally ignored in Phase 1:
  - More complete ReDoS pattern analysis and safe regex sandboxing.
  - Rate limit bypass detection heuristics (currently ignored).
  - A more robust JWKS/jwt helper to reduce fixture complexity in tests (avoid brittle modulus hacks).
- Provide runtime toggles or a features set that permits per-endpoint or per-tools tuning of the strictness of detection.
- Consider a 'safe parsing mode' for JSON fields that may legitimately contain non-ASCII or unusual Unicode glyphs to reduce false positives.
- Add instrumented metrics & telemetry for each detection check to track false positive rates and refine heuristics.

## References

- AGENTS and project workflow: `xzepr-mcp/AGENTS.md`
- Input Validator core: `src/middleware/validation.rs`
- Tests: `tests/security_injection.rs`, `tests/fixtures/payloads.rs`
- Config: `src/config/settings.rs`, `config/{development,production}.yaml`

---

If you'd like, I can:
- Break down the change set into smaller PRs (one per detection) and generate specialized unit tests for each new check.
- Implement the remaining advanced detection tests (e.g., rate limit bypass detection).
- Improve ReDoS detection to include runtime timeouts or sandboxing.
