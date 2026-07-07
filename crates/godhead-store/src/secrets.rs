use regex::Regex;
use std::sync::OnceLock;

/// Law XV.2 — defense-in-depth scan of outbound writes for known secret
/// shapes. A hit refuses the write and is logged severity: violation.
/// Patterns cover the fixture set of SC-F10/SC-H06: connection strings with
/// inline passwords, cloud key ids, bearer tokens, PEM material, and
/// vendor API-key prefixes.
static PATTERNS: OnceLock<Vec<(&'static str, Regex)>> = OnceLock::new();

fn patterns() -> &'static Vec<(&'static str, Regex)> {
    PATTERNS.get_or_init(|| {
        vec![
            (
                "url-with-password",
                Regex::new(r"[a-zA-Z][a-zA-Z0-9+.-]*://[^/\s:@]+:[^/\s@]+@").unwrap(),
            ),
            (
                "aws-access-key-id",
                Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap(),
            ),
            (
                "bearer-token",
                Regex::new(r"(?i)\bbearer\s+[A-Za-z0-9._~+/-]{16,}=*").unwrap(),
            ),
            (
                "pem-block",
                Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY-----").unwrap(),
            ),
            (
                "api-key-prefix",
                Regex::new(r"\bsk-[A-Za-z0-9_-]{16,}\b").unwrap(),
            ),
            (
                "github-token",
                Regex::new(r"\bgh[pousr]_[A-Za-z0-9]{20,}\b").unwrap(),
            ),
        ]
    })
}

/// Scans serialized content; returns the name of the first matching pattern.
pub fn scan(content: &str) -> Option<&'static str> {
    patterns()
        .iter()
        .find(|(_, re)| re.is_match(content))
        .map(|(name, _)| *name)
}
