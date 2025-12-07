//! Input validation and sanitization
//!
//! This module provides input validation with:
//! - Schema validation
//! - ULID and UUID validation
//! - Semver validation
//! - Injection detection (SQL, XSS, path traversal)
//! - Payload size limits (64KB)
//! - JSON depth limits (32 levels)
//!
//! Implementation is a stub for Phase 1 foundation.

use crate::config::{SecurityCheck, SecurityConfig, Settings};
use crate::error::Result;
use regex::Regex;
use std::future::Future;
use std::pin::Pin;
use tokio::runtime::Builder;

/// Synchronous helper to run a Future on a new Tokio current-thread runtime.
/// We create a current-thread runtime to avoid requiring futures to be 'static',
/// which keeps the blocking wrappers compatible with borrowed futures.
///
/// This replaces usage of `futures::executor::block_on`, so we don't depend on
/// the `futures` crate for a simple block-on operation.
fn tokio_block_on<F: Future>(fut: F) -> F::Output {
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");
    rt.block_on(fut)
}

/// Validation rules
#[derive(Clone)]
pub struct ValidationRules {
    /// Security configuration
    _config: SecurityConfig,
}

impl ValidationRules {
    /// Create validation rules from configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Security configuration
    ///
    /// # Returns
    ///
    /// Returns a new `ValidationRules` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use xzepr_mcp::middleware::ValidationRules;
    /// use xzepr_mcp::config::Settings;
    ///
    /// let settings = Settings::default();
    /// let rules = ValidationRules::new(&settings.security);
    /// ```
    pub fn new(config: &SecurityConfig) -> Self {
        Self {
            _config: config.clone(),
        }
    }
}

/// Input validator
#[derive(Clone)]
pub struct InputValidator {
    /// Validation rules
    _rules: ValidationRules,
}

impl InputValidator {
    /// Create a new input validator
    ///
    /// # Arguments
    ///
    /// * `rules` - Validation rules
    ///
    /// # Returns
    ///
    /// Returns a new `InputValidator` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use xzepr_mcp::middleware::InputValidator;
    ///
    /// let validator = InputValidator::new();
    /// ```
    /// Default constructor using application defaults.
    ///
    /// This convenience constructor is used by tests and by places that do not
    /// need to pass explicit rules. If explicit rules are required, use
    /// `new_with_rules`.
    pub fn new() -> Self {
        let settings = Settings::default();
        let rules = ValidationRules::new(&settings.security);
        Self { _rules: rules }
    }

    /// Constructor that accepts explicit `ValidationRules`.
    ///
    /// Use this when you have computed rules (for example from Settings or
    /// other configuration) and want to inject them into the validator.
    ///
    /// # Examples
    ///
    /// ```
    /// use xzepr_mcp::middleware::{InputValidator, ValidationRules};
    /// use xzepr_mcp::config::Settings;
    ///
    /// let settings = Settings::default();
    /// let rules = ValidationRules::new(&settings.security);
    /// let validator = InputValidator::new_with_rules(rules);
    /// ```
    pub fn new_with_rules(rules: ValidationRules) -> Self {
        Self { _rules: rules }
    }

    /// Validate ULID format
    ///
    /// This validator rejects ULIDs that match suspicious patterns which are
    /// commonly used for injection or path traversal. In particular:
    /// - Reject path-like strings (containing `/`, `\` or `..`)
    /// - Reject "all zeros" ULID values (commonly a sentinel / placeholder)
    ///
    /// # Arguments
    ///
    /// * `ulid` - ULID string to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if valid
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::InvalidUlid` if invalid
    pub async fn validate_ulid(&self, ulid: &str) -> Result<()> {
        // Trim and normalise the candidate to avoid accidental whitespace issues.
        let candidate = ulid.trim();

        // Basic path traversal checks: disallow values that look like filesystem paths
        // or include navigation tokens. These are never valid identifiers and may be
        // indicative of an attack vector.
        if candidate.contains('/') || candidate.contains('\\') || candidate.contains("..") {
            return Err(crate::error::Error::Validation(
                crate::error::ValidationError::InvalidUlid(candidate.to_string()),
            ));
        }

        // Reject obviously invalid / sentinel ULIDs such as "00000000000000000000000000".
        // While some parsers accept a zero ULID, it's frequently used as a placeholder and
        // should not be considered a valid _resource_ identifier in request payloads.
        if !candidate.is_empty() && candidate.chars().all(|c| c == '0') {
            return Err(crate::error::Error::Validation(
                crate::error::ValidationError::InvalidUlid(candidate.to_string()),
            ));
        }

        // Reject ULIDs made entirely of the same character (e.g., 'ZZZ...') as suspicious and invalid.
        // While the ULID spec allows certain base32 characters, all-same strings are not valid
        // unique identifiers and are commonly used as placeholders or attack input.
        if candidate.len() > 1 {
            if let Some(first_char) = candidate.chars().next() {
                if candidate.chars().all(|c| c == first_char) {
                    return Err(crate::error::Error::Validation(
                        crate::error::ValidationError::InvalidUlid(candidate.to_string()),
                    ));
                }
            }
        }

        // Fallback to library validation for structural correctness
        match ulid::Ulid::from_string(candidate) {
            Ok(_) => Ok(()),
            Err(_) => Err(crate::error::Error::Validation(
                crate::error::ValidationError::InvalidUlid(candidate.to_string()),
            )),
        }
    }

    /// Blocking synchronous wrapper for validate_ulid.
    /// This keeps API compatibility for callers that are not using async/await.
    pub fn validate_ulid_blocking(&self, ulid: &str) -> Result<()> {
        tokio_block_on(self.validate_ulid(ulid))
    }

    /// Validate semver format
    ///
    /// # Arguments
    ///
    /// * `version` - Version string to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if valid
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::InvalidSemver` if invalid
    pub async fn validate_semver(&self, semver: &str) -> Result<()> {
        match semver::Version::parse(semver) {
            Ok(_) => Ok(()),
            Err(_) => Err(crate::error::Error::Validation(
                crate::error::ValidationError::InvalidSemver(semver.to_string()),
            )),
        }
    }

    /// Blocking synchronous wrapper for validate_semver.
    pub fn validate_semver_blocking(&self, version: &str) -> Result<()> {
        tokio_block_on(self.validate_semver(version))
    }

    /// Blocking synchronous wrapper for validate_payload_size
    pub fn validate_payload_size_blocking(
        &self,
        payload_size: usize,
        max_size: usize,
    ) -> Result<()> {
        self.validate_payload_size(payload_size, max_size)
    }

    /// Detect SQL injection patterns
    ///
    /// # Arguments
    ///
    /// * `input` - Input string to check
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if safe
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::PotentialInjection` if suspicious
    pub async fn detect_sql_injection(&self, input: &str) -> Result<()> {
        let lower_input = input.to_lowercase();

        // SQL keywords that indicate injection attempts
        let sql_keywords = [
            "select",
            "insert",
            "update",
            "delete",
            "drop",
            "create",
            "alter",
            "exec",
            "union",
            "join",
            "where",
            "truncate",
            "grant",
            "revoke",
            "sleep",
            "waitfor",
            "benchmark",
            "load_file",
            "into outfile",
        ];

        // SQL injection patterns (regex-like matching)
        let injection_patterns = [
            ("'", "single quote"),
            ("\"", "double quote"),
            ("--", "SQL comment"),
            ("#", "MySQL comment"),
            ("/*", "block comment start"),
            ("*/", "block comment end"),
            (";", "statement terminator"),
            ("=", "equals operator"),
            ("||", "string concatenation"),
            ("@@", "system variable"),
            ("char(", "char function"),
            ("concat(", "concat function"),
            ("0x", "hex encoding"),
        ];

        // Shell/command injection heuristics – detect typical shell metacharacters and constructs.
        // These patterns are conservative heuristics designed to flag command injection payloads
        // (e.g., pipes, redirections, backticks, command substitution), which are not normally
        // valid input for our JSON fields and are strong indicators of an injection attempt.
        let command_patterns = [
            ("`", "backtick command"),
            ("$(", "command substitution"),
            ("; ", "command separator"),
            ("| ", "pipe token (leading)"),
            (" | ", "pipe token (surrounded)"),
            ("& ", "ampersand (leading)"),
            (" &&", "logical AND shell operator"),
            (" ||", "logical OR shell operator"),
            (">", "redirect output"),
            ("<", "redirect input"),
        ];

        // LDAP-like patterns (simple heuristics for LDAP filter/attribute injection)
        // These attempt to catch common LDAP injection forms such as `*)(...` or injection
        // filters that include `objectClass=` and injection-style parentheses sequences.
        let ldap_patterns = [
            ("objectclass=", "LDAP filter 'objectClass'"),
            (")(", "LDAP filter parentheses sequence"),
            ("*)(", "LDAP wildcard/parenthesis pairing"),
        ];

        // Boolean-based SQL injection patterns
        let boolean_patterns = [
            " or ", " and ", "'='", "\"=\"", "1=1", "1=2", "'1'='1", "'x'='x",
        ];

        // Check for SQL keywords first (case-insensitive)
        for keyword in &sql_keywords {
            if lower_input.contains(keyword) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: format!(
                            "Potential SQL injection detected with keyword: {}",
                            keyword
                        ),
                    },
                ));
            }
        }

        // Check for SQL injection patterns (character-based checks)
        for (pattern, description) in &injection_patterns {
            if input.contains(pattern) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: format!("Potential SQL injection detected: {}", description),
                    },
                ));
            }
        }

        // Check for shell/command injection patterns
        for (pattern, description) in &command_patterns {
            if input.contains(pattern) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: format!("Potential command injection detected: {}", description),
                    },
                ));
            }
        }

        // Check for LDAP-like injection patterns (case-insensitive)
        for (pattern, description) in &ldap_patterns {
            if lower_input.contains(pattern) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: format!("Potential LDAP injection detected: {}", description),
                    },
                ));
            }
        }

        // Detect LDAP wildcard usage heuristics such as "admin*" which are commonly used
        // in LDAP filter expansions and can be a sign of LDAP injection.
        //
        // This check is conservative: it looks for trailing '*' suffix usage where the
        // prefix is an identifier-like token (letters/numbers/underscore/dash/dot).
        // We don't attempt to be perfectly exhaustive; this protects many simple
        // wildcard-based LDAP attacks (e.g. "admin*", "user*").
        if lower_input.ends_with('*') {
            let stripped = lower_input.trim_end_matches('*');
            if !stripped.is_empty()
                && stripped
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
            {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: "Potential LDAP wildcard injection detected".to_string(),
                    },
                ));
            }
        }

        // Check for boolean-based SQL injection patterns
        for pattern in &boolean_patterns {
            if lower_input.contains(pattern) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: "Potential boolean-based SQL injection detected".to_string(),
                    },
                ));
            }
        }

        Ok(())
    }

    /// Blocking synchronous wrapper for detect_sql_injection.
    pub fn detect_sql_injection_blocking(&self, input: &str) -> Result<()> {
        tokio_block_on(self.detect_sql_injection(input))
    }

    /// Detect XSS patterns
    ///
    /// # Arguments
    ///
    /// * `input` - Input string to check
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if safe
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::PotentialInjection` if suspicious
    pub async fn detect_xss(&self, input: &str) -> Result<()> {
        let lower_input = input.to_lowercase();

        // HTML tags commonly used in XSS
        let dangerous_tags = [
            "<script",
            "</script>",
            "<iframe",
            "<object",
            "<embed",
            "<svg",
            "<img",
            "<body",
            "<input",
            "<marquee",
            "<details",
            "<form",
            "<button",
            "<a ",
            "<link",
            "<style",
            "<meta",
            "<base",
        ];

        // Event handlers
        let event_handlers = [
            "onload=",
            "onerror=",
            "onclick=",
            "onmouseover=",
            "onfocus=",
            "onblur=",
            "onsubmit=",
            "onchange=",
            "oninput=",
            "onkeydown=",
            "onkeyup=",
            "onkeypress=",
            "ondblclick=",
            "oncontextmenu=",
            "onstart=",
            "ontoggle=",
            "onresize=",
            "onscroll=",
            "onwheel=",
            "ondrag=",
            "ondrop=",
            "oncopy=",
            "oncut=",
            "onpaste=",
            "onanimationend=",
            "onanimationstart=",
            "ontransitionend=",
        ];

        // Protocol handlers
        let protocol_patterns = [
            "javascript:",
            "vbscript:",
            "data:text/html",
            "data:application",
        ];

        // Encoded patterns (common XSS encoding)
        let encoded_patterns = [
            "&lt;script",
            "&#60;script",
            "%3cscript",
            "\\x3cscript",
            "&lt;img",
            "&#60;img",
            "%3cimg",
        ];

        // Header injection (CRLF) patterns - detect attempts to inject additional HTTP headers
        // and CRLF sequences which are a vector for header injection attacks.
        let header_injection_patterns = ["\r\n", "\n", "%0d%0a", "%0a%0d"];

        // Format string attack patterns - many code paths pass strings into formatters that use
        // % specifiers; detecting obviously malicious sequences like repeated `%s` or `%n` can
        // reduce risk in such contexts.
        let format_patterns = ["%s", "%x", "%n", "%p", "%d", "%u", "%f", "%g"];

        // Check dangerous HTML tags early to quickly catch obvious XSS attempts.
        for tag in &dangerous_tags {
            if lower_input.contains(tag) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: format!("Potential XSS detected: dangerous HTML tag {}", tag),
                    },
                ));
            }
        }

        // Check encoded patterns
        for encoded in &encoded_patterns {
            if lower_input.contains(encoded) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: "Potential XSS detected: encoded script pattern".to_string(),
                    },
                ));
            }
        }

        // Check header injection patterns
        for header in &header_injection_patterns {
            if input.contains(header) || lower_input.contains(header) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: format!("Potential header injection detected: {}", header),
                    },
                ));
            }
        }

        // Check format string patterns
        for pat in &format_patterns {
            if input.contains(pat) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: format!("Potential format string detected: {}", pat),
                    },
                ));
            }
        }

        // Unicode attack patterns - right-to-left override (U+202E), left-to-right override (U+202D),
        // zero-width space (U+200B), BOM/zero-width no-break space (U+FEFF), and null character (U+0000).
        // These characters can be used to trick rendering, alter string ordering, hide malicious content,
        // or confuse parsing/validation layers. We treat them as potentially malicious for input strings.
        if input.chars().any(|c| {
            c == '\u{202e}'
                || c == '\u{202d}'
                || c == '\u{200b}'
                || c == '\u{feff}'
                || c == '\u{0000}'
        }) {
            return Err(crate::error::Error::Validation(
                crate::error::ValidationError::PotentialInjection {
                    field: "input".to_string(),
                    reason: "Potential Unicode/directionality attack detected".to_string(),
                },
            ));
        }

        // Additional Unicode obfuscation detection for enclosed alphanumerics, mathematical
        // alphanumeric symbols, fullwidth characters, and common homoglyph ranges (e.g., Cyrillic).
        // These ranges are commonly used to craft homoglyphs and obfuscated payloads that look like
        // ASCII strings to a human reader but are different codepoints, enabling bypasses of filters.
        if input.chars().any(|c| {
            let cp = c as u32;
            // Enclosed Alphanumerics: circled letters/numbers (U+2460–U+24FF)
            (0x2460..=0x24FF).contains(&cp)
            // Mathematical Alphanumeric Symbols: bold/italic variants and siblings (U+1D400–U+1D7FF)
            || (0x1D400..=0x1D7FF).contains(&cp)
            // Fullwidth Forms: fullwidth Latin etc (U+FF00–U+FFEF)
            || (0xFF00..=0xFFEF).contains(&cp)
            // Cyrillic block (common homoglyph substitutions)
            || (0x0400..=0x04FF).contains(&cp)
        }) {
            return Err(crate::error::Error::Validation(
                crate::error::ValidationError::PotentialInjection {
                    field: "input".to_string(),
                    reason: "Potential Unicode obfuscation or homoglyph attack detected"
                        .to_string(),
                },
            ));
        }

        // Check event handlers
        for handler in &event_handlers {
            if lower_input.contains(handler) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: format!("Potential XSS detected: event handler {}", handler),
                    },
                ));
            }
        }

        // Check protocol patterns
        for protocol in &protocol_patterns {
            if lower_input.contains(protocol) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: format!("Potential XSS detected: dangerous protocol {}", protocol),
                    },
                ));
            }
        }

        // Duplicate encoded pattern check removed: encoded script patterns are already
        // detected earlier in this function.

        Ok(())
    }

    /// Blocking synchronous wrapper for detect_xss.
    pub fn detect_xss_blocking(&self, input: &str) -> Result<()> {
        tokio_block_on(self.detect_xss(input))
    }

    /// Detect path traversal patterns
    ///
    /// # Arguments
    ///
    /// * `input` - Input string to check
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if safe
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::PotentialInjection` if suspicious
    pub async fn detect_path_traversal(&self, input: &str) -> Result<()> {
        let lower_input = input.to_lowercase();

        // Basic traversal patterns
        let traversal_patterns = [
            "../",
            "..\\",
            "..\\/",
            "..;/",
            "....//",
            "/etc/",
            "\\windows\\",
            "/bin/",
            "/usr/",
            "/proc/",
            "/sys/",
            "/dev/",
            "/root/",
        ];

        // URL encoded patterns
        let encoded_patterns = [
            "%2e%2e%2f",
            "%2e%2e/",
            "..%2f",
            "%2e%2e\\",
            "..%252f",
            "%252e%252e%252f",
            "%c0%ae%c0%ae/", // overlong UTF-8 encoding
            "..%c0%af",
            "..%255c",
        ];

        // Null byte patterns
        let null_byte_patterns = ["%00", "\0", "\\0"];

        // Check basic traversal patterns
        for pattern in &traversal_patterns {
            if input.contains(pattern) || lower_input.contains(pattern) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: format!(
                            "Potential path traversal detected with pattern: {}",
                            pattern
                        ),
                    },
                ));
            }
        }

        // Check URL encoded patterns
        for pattern in &encoded_patterns {
            if lower_input.contains(pattern) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: "Potential path traversal detected: URL encoded pattern"
                            .to_string(),
                    },
                ));
            }
        }

        // Check null byte patterns
        for pattern in &null_byte_patterns {
            if input.contains(pattern) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: "Potential path traversal detected: null byte injection"
                            .to_string(),
                    },
                ));
            }
        }

        Ok(())
    }

    /// Blocking synchronous wrapper for detect_path_traversal.
    pub fn detect_path_traversal_blocking(&self, input: &str) -> Result<()> {
        tokio_block_on(self.detect_path_traversal(input))
    }

    /// Detect SSRF (Server-Side Request Forgery) patterns
    ///
    /// # Arguments
    ///
    /// * `input` - Input string to check for SSRF / internal resource access
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if safe
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::PotentialInjection` if suspicious SSRF patterns are found
    pub async fn detect_ssrf(&self, input: &str) -> Result<()> {
        let lower_input = input.to_lowercase();

        // Common SSRF patterns (hostnames, IPs, metadata endpoints, and schemes)
        let ssrf_patterns = [
            "http://localhost",
            "https://localhost",
            "http://127.0.0.1",
            "https://127.0.0.1",
            "http://[::1]",
            "http://0.0.0.0",
            "http://169.254.169.254", // AWS metadata host
            "metadata.google.internal",
            "http://metadata.google.internal",
            "file://",
            "gopher://",
            "http://[::ffff:127.0.0.1]",
            "http://[::1]",
        ];

        for pattern in &ssrf_patterns {
            if lower_input.contains(pattern) {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: format!("Potential SSRF detected: {}", pattern),
                    },
                ));
            }
        }

        Ok(())
    }

    /// Blocking synchronous wrapper for detect_ssrf.
    pub fn detect_ssrf_blocking(&self, input: &str) -> Result<()> {
        tokio_block_on(self.detect_ssrf(input))
    }

    /// Detect potential Regular Expression Denial of Service (ReDoS) patterns.
    ///
    /// This function applies heuristics to detect strings that likely represent
    /// crafted regular expressions that can cause catastrophic backtracking or heavy
    /// CPU usage when compiled and executed by a regex engine. It looks for nested
    /// quantifiers and suspiciously long repetitive sequences.
    pub async fn detect_redos(&self, input: &str) -> Result<()> {
        // Lowercased input for consistent pattern checks
        let lower_input = input.to_lowercase();

        // Heuristic regex patterns for ReDoS detection. The regexes are intentionally
        // conservative: they detect nested quantifiers and alternation-with-star constructs
        // which are common in ReDoS payloads.
        let nested_quantifier = Regex::new(r"\([^)]*\+[^\)]*\)\+").unwrap();
        let alt_star_quantifier = Regex::new(r"\([^)]*\|[^)]*\)\*").unwrap();

        // Check for nested quantifier patterns (e.g., (a+)+, ((a+)+)+)
        if nested_quantifier.is_match(&lower_input) {
            return Err(crate::error::Error::Validation(
                crate::error::ValidationError::PotentialInjection {
                    field: "input".to_string(),
                    reason: "Potential ReDoS detected: nested quantifier pattern".to_string(),
                },
            ));
        }

        // Check for alternation followed by star (e.g., (a|a)* or (a|ab)*), which can cause
        // catastrophic backtracking when combined with other constructs.
        if alt_star_quantifier.is_match(&lower_input) {
            return Err(crate::error::Error::Validation(
                crate::error::ValidationError::PotentialInjection {
                    field: "input".to_string(),
                    reason: "Potential ReDoS detected: alternation followed by repetition"
                        .to_string(),
                },
            ));
        }

        // Simple repeated-character heuristic (detects long runs of a single character,
        // which are often used to stress regex engines)
        let mut repeat_count = 1usize;
        let mut prev_char: Option<char> = None;
        for ch in lower_input.chars() {
            if Some(ch) == prev_char {
                repeat_count += 1;
                if repeat_count > 20 {
                    return Err(crate::error::Error::Validation(
                        crate::error::ValidationError::PotentialInjection {
                            field: "input".to_string(),
                            reason: "Potential ReDoS detected: excessive repeated character run"
                                .to_string(),
                        },
                    ));
                }
            } else {
                prev_char = Some(ch);
                repeat_count = 1;
            }
        }

        Ok(())
    }

    /// Blocking synchronous wrapper for detect_redos.
    pub fn detect_redos_blocking(&self, input: &str) -> Result<()> {
        tokio_block_on(self.detect_redos(input))
    }

    /// Validate input against a whitelist
    ///
    /// # Arguments
    ///
    /// * `input` - Input string to check
    /// * `whitelist` - Allowed values
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if input is in whitelist
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::InvalidEnumValue` if not in whitelist
    pub async fn validate_whitelist(&self, input: &str, whitelist: &[&str]) -> Result<()> {
        if whitelist.contains(&input) {
            Ok(())
        } else {
            Err(crate::error::Error::Validation(
                crate::error::ValidationError::InvalidEnumValue {
                    field: "input".to_string(),
                    value: input.to_string(),
                },
            ))
        }
    }

    /// Validate input is alphanumeric
    ///
    /// # Arguments
    ///
    /// * `input` - Input string to check
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if alphanumeric
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::FieldValidation` if not alphanumeric
    pub async fn validate_alphanumeric(&self, input: &str) -> Result<()> {
        if input.chars().all(|c| c.is_alphanumeric()) {
            Ok(())
        } else {
            Err(crate::error::Error::Validation(
                crate::error::ValidationError::FieldValidation {
                    field: "input".to_string(),
                    reason: "Must be alphanumeric".to_string(),
                },
            ))
        }
    }

    /// Validate input length
    ///
    /// # Arguments
    ///
    /// * `input` - Input string to check
    /// * `min` - Minimum length
    /// * `max` - Maximum length
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if length is valid
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::FieldValidation` if length invalid
    pub async fn validate_length(&self, input: &str, min: usize, max: usize) -> Result<()> {
        let len = input.len();
        if len >= min && len <= max {
            Ok(())
        } else {
            Err(crate::error::Error::Validation(
                crate::error::ValidationError::FieldValidation {
                    field: "input".to_string(),
                    reason: format!("Length must be between {} and {}", min, max),
                },
            ))
        }
    }

    /// Validate payload size
    ///
    /// # Arguments
    ///
    /// * `size` - Size in bytes
    /// * `max_size` - Maximum allowed size
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if size is valid
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::PayloadTooLarge` if too large
    pub fn validate_payload_size(&self, size: usize, max_size: usize) -> Result<()> {
        if size <= max_size {
            Ok(())
        } else {
            Err(crate::error::Error::Validation(
                crate::error::ValidationError::PayloadTooLarge {
                    size,
                    max: max_size,
                },
            ))
        }
    }

    /// Validate generic JSON input
    ///
    /// Performs basic validation on JSON input values:
    /// - Checks string fields for potential injection patterns
    /// - Validates size constraints
    /// - Ensures no malicious content
    ///
    /// # Arguments
    ///
    /// * `input` - JSON value to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if input passes validation
    ///
    /// # Errors
    ///
    /// Returns error if validation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use xzepr_mcp::middleware::InputValidator;
    /// use serde_json::json;
    ///
    /// let validator = InputValidator::new();
    ///
    /// let input = json!({"event_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV"});
    /// let result = validator.validate_input(&input);
    /// assert!(result.is_ok());
    /// ```
    /// Synchronous wrapper for input validation.
    ///
    /// Prefer using `validate_input_async` directly from an async context for better performance.
    /// The synchronous wrapper uses a current-thread Tokio runtime so we do not attempt to spawn
    /// or configure a separate multi-threaded runtime when invoked from tests or synchronous code.
    pub fn validate_input(&self, input: &serde_json::Value) -> Result<()> {
        tokio_block_on(self.validate_input_iter(input))
    }

    /// Asynchronous, box-pinned validation function to support recursion.
    ///
    /// Delegates to `validate_input_iter` which performs an iterative traversal
    /// of the JSON value using a stack. The iterative traversal avoids recursive
    /// `async` recursion, enforces configured limits (depth, object keys, array
    /// size and string size), and runs the detection subsystems on string values.
    pub fn validate_input_async<'a>(
        &'a self,
        input: &'a serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Delegate actual work to the async iterative validator.
            self.validate_input_iter(input).await
        })
    }

    /// Iterative asynchronous validator that enforces maximum depth, key count, array sizes,
    /// and executes detectors for string contents. Using an iterative stack avoids recursion
    /// and enables enforcement of configured resource limits for JSON inputs.
    pub async fn validate_input_iter(&self, input: &serde_json::Value) -> Result<()> {
        // Load configured limits from validation rules
        let max_depth = self._rules._config.max_json_depth;
        let max_keys = self._rules._config.max_json_keys;
        let max_array_len = self._rules._config.max_array_len;
        let max_string_len = self._rules._config.max_string_len;

        // Stack of (node_reference, depth)
        let mut stack: Vec<(&serde_json::Value, usize)> = Vec::new();
        stack.push((input, 0usize));

        while let Some((node, depth)) = stack.pop() {
            // Enforce depth limit
            if depth > max_depth {
                return Err(crate::error::Error::Validation(
                    crate::error::ValidationError::PotentialInjection {
                        field: "input".to_string(),
                        reason: format!(
                            "JSON nesting depth ({}) exceeds maximum allowed {}",
                            depth, max_depth
                        ),
                    },
                ));
            }

            match node {
                serde_json::Value::String(s) => {
                    // Enforce string size limits
                    if s.len() > max_string_len {
                        return Err(crate::error::Error::Validation(
                            crate::error::ValidationError::PayloadTooLarge {
                                size: s.len(),
                                max: max_string_len,
                            },
                        ));
                    }

                    // Run configured detection heuristics (most detection toggles are configured by SecurityConfig)
                    if self
                        ._rules
                        ._config
                        .detection
                        .is_enabled(SecurityCheck::SqlInjection)
                    {
                        self.detect_sql_injection(s).await?;
                    }
                    if self._rules._config.detection.is_enabled(SecurityCheck::Xss) {
                        self.detect_xss(s).await?;
                    }
                    if self
                        ._rules
                        ._config
                        .detection
                        .is_enabled(SecurityCheck::PathTraversal)
                    {
                        self.detect_path_traversal(s).await?;
                    }
                    if self
                        ._rules
                        ._config
                        .detection
                        .is_enabled(SecurityCheck::Ssrf)
                    {
                        self.detect_ssrf(s).await?;
                    }

                    // ReDoS detection heuristics - always applied to string fields
                    self.detect_redos(s).await?;
                }

                serde_json::Value::Object(map) => {
                    // Key count limit enforcement
                    if map.len() > max_keys {
                        return Err(crate::error::Error::Validation(
                            crate::error::ValidationError::PotentialInjection {
                                field: "input".to_string(),
                                reason: format!("Too many keys in JSON object: {}", map.len()),
                            },
                        ));
                    }

                    // Iterate over object entries (use iter() to avoid moving the map)
                    for (key, value) in map {
                        // Prototype pollution detection
                        if key == "__proto__"
                            || key == "prototype"
                            || key == "constructor.prototype"
                            || key.contains(".prototype")
                            || key.contains("__proto__")
                            || key == "constructor"
                        {
                            return Err(crate::error::Error::Validation(
                                crate::error::ValidationError::PotentialInjection {
                                    field: "input".to_string(),
                                    reason: format!(
                                        "Potential prototype pollution detected: {}",
                                        key
                                    ),
                                },
                            ));
                        }

                        // Push child JSON value onto stack for further evaluation
                        stack.push((value, depth + 1));
                    }
                }

                serde_json::Value::Array(arr) => {
                    // Element count limit enforcement
                    if arr.len() > max_array_len {
                        return Err(crate::error::Error::Validation(
                            crate::error::ValidationError::PotentialInjection {
                                field: "input".to_string(),
                                reason: format!(
                                    "Array length ({}) exceeds maximum allowed {}",
                                    arr.len(),
                                    max_array_len
                                ),
                            },
                        ));
                    }

                    // Push array elements onto the stack
                    for item in arr {
                        stack.push((item, depth + 1));
                    }
                }

                _ => {
                    // Numbers, booleans, and null are considered safe for the purpose of
                    // injection detection and size checks (handled by other methods when needed).
                }
            }
        }

        Ok(())
    }

    /// Blocking synchronous wrapper for validate_input that uses a current-thread runtime.
    pub fn validate_input_blocking(&self, input: &serde_json::Value) -> Result<()> {
        tokio_block_on(self.validate_input_iter(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;

    #[test]
    fn test_validation_rules_creation() {
        let settings = Settings::default();
        let _rules = ValidationRules::new(&settings.security);
    }

    #[test]
    fn test_input_validator_creation() {
        let settings = Settings::default();
        let rules = ValidationRules::new(&settings.security);
        let _validator = InputValidator::new_with_rules(rules);
    }

    #[test]
    fn test_validate_ulid_stub() {
        let settings = Settings::default();
        let _rules = ValidationRules::new(&settings.security);
        let validator = InputValidator::new();

        let result = validator.validate_ulid_blocking("01ARZ3NDEKTSV4RRFFQ69G5FAV");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_semver_stub() {
        let settings = Settings::default();
        let _rules = ValidationRules::new(&settings.security);
        let validator = InputValidator::new();

        let result = validator.validate_semver_blocking("1.0.0");
        assert!(result.is_ok());
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}
