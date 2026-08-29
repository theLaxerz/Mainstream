//! Shared input validation for IPC-facing commands.
//!
//! Defense in depth: the webview is local-first, but RSS, email HTML, and a
//! missing CSP historically meant untrusted strings could reach `open`, HTTP
//! clients, and the filesystem.

use crate::db::DbError;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Component, Path};
use url::Url;

/// Schemes `open` is allowed to hand to macOS. Blocks `file:`, `javascript:`,
/// `data:`, `smb:`, and other handlers that would expand IPC into local RCE.
const OPEN_URL_SCHEMES: &[&str] = &[
    "https",
    "http",
    "mailto",
    "message",
    "imessage",
    "sms",
    "tel",
    "calshow",
    "x-apple.systempreferences",
];

const MAX_HTTP_RESPONSE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_HEALTH_EXPORT_BYTES: u64 = 200 * 1024 * 1024;

pub fn max_http_response_bytes() -> u64 {
    MAX_HTTP_RESPONSE_BYTES
}

pub fn max_health_export_bytes() -> u64 {
    MAX_HEALTH_EXPORT_BYTES
}

pub fn deny(msg: impl Into<String>) -> DbError {
    DbError::Message(msg.into())
}

fn scheme_of(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    let colon = trimmed.find(':')?;
    if colon == 0 {
        return None;
    }
    let scheme = &trimmed[..colon];
    if scheme
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'.' || b == b'-')
    {
        Some(scheme)
    } else {
        None
    }
}

/// True when `host` is loopback, RFC1918, link-local, CGNAT, or unspecified.
pub fn is_non_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_non_public_ipv4(v4),
        IpAddr::V6(v6) => is_non_public_ipv6(v6),
    }
}

fn is_non_public_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_unspecified()
        || ip.octets() == [255, 255, 255, 255]
        || ip.octets()[0] == 0
        // Carrier-grade NAT 100.64.0.0/10
        || (ip.octets()[0] == 100 && (64..=127).contains(&ip.octets()[1]))
        // IETF protocol assignments 192.0.0.0/24 (includes 192.0.0.8 etc.)
        || (ip.octets()[0] == 192 && ip.octets()[1] == 0 && ip.octets()[2] == 0)
}

fn is_non_public_ipv6(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_non_public_ipv4(v4);
    }
    ip.is_loopback()
        || ip.is_multicast()
        || ip.is_unspecified()
        || (ip.segments()[0] & 0xfe00) == 0xfc00 // unique local fc00::/7
        || (ip.segments()[0] & 0xffc0) == 0xfe80 // link-local fe80::/10
}

fn host_is_blocked(host: &str) -> bool {
    let host = host.trim().trim_matches(|c| c == '[' || c == ']');
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty()
        || host == "localhost"
        || host == "localhost.localdomain"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host == "metadata.google.internal"
    {
        return true;
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        return is_non_public_ip(ip);
    }
    false
}

fn ensure_no_userinfo(url: &Url) -> Result<(), DbError> {
    if !url.username().is_empty() || url.password().is_some() {
        return Err(deny("URL must not contain credentials"));
    }
    Ok(())
}

/// HTTPS-only URL for fetching untrusted remote bytes (email HTML images).
pub fn parse_public_https_url(raw: &str) -> Result<Url, DbError> {
    let url = parse_public_http_url(raw)?;
    if url.scheme() != "https" {
        return Err(deny("URL must use HTTPS"));
    }
    Ok(url)
}

pub fn public_http_client(
    timeout_secs: u64,
    user_agent: Option<&str>,
) -> Result<reqwest::blocking::Client, DbError> {
    let mut builder = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .redirect(reqwest::redirect::Policy::custom(
            |attempt| match blocked_http_client_redirect(attempt.previous().len(), attempt.url()) {
                Ok(()) => attempt.follow(),
                Err(_) => attempt.error("blocked redirect"),
            },
        ));
    if let Some(ua) = user_agent {
        builder = builder.user_agent(ua);
    }
    builder
        .build()
        .map_err(|e| deny(format!("http client: {e}")))
}

/// HTTP(S) URL that is safe to fetch from an untrusted source (RSS, email HTML).
pub fn parse_public_http_url(raw: &str) -> Result<Url, DbError> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 2048 {
        return Err(deny("URL is empty or too long"));
    }
    if raw.contains(['\n', '\r', '\0', '\t']) {
        return Err(deny("URL contains invalid characters"));
    }
    let url = Url::parse(raw).map_err(|_| deny("invalid URL"))?;
    match url.scheme() {
        "https" | "http" => {}
        other => return Err(deny(format!("blocked URL scheme: {other}"))),
    }
    ensure_no_userinfo(&url)?;
    let host = url
        .host_str()
        .ok_or_else(|| deny("URL is missing a host"))?;
    if host_is_blocked(host) {
        return Err(deny("URL host is not allowed"));
    }
    Ok(url)
}

/// RSS / news feed URLs — same as public HTTP, plus no query-less `file:` tricks.
pub fn validate_feed_url(raw: &str) -> Result<String, DbError> {
    let url = parse_public_http_url(raw)?;
    Ok(url.as_str().to_string())
}

/// URLs passed to macOS `open`.
pub fn validate_open_url(raw: &str) -> Result<String, DbError> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 4096 {
        return Err(deny("target is required"));
    }
    if raw.contains(['\n', '\r', '\0']) {
        return Err(deny("target contains invalid characters"));
    }
    let Some(scheme) = scheme_of(raw) else {
        return Err(deny("target must include a URL scheme"));
    };
    let scheme_l = scheme.to_ascii_lowercase();
    if !OPEN_URL_SCHEMES.iter().any(|allowed| scheme_l == *allowed) {
        return Err(deny(format!("blocked URL scheme: {scheme_l}")));
    }
    // Extra host checks for http(s) so a malicious feed cannot `open` file-like
    // hosts. Custom schemes (message:, calshow:) are allowlisted above.
    if scheme_l == "http" || scheme_l == "https" {
        parse_public_http_url(raw)?;
    }
    Ok(raw.to_string())
}

pub fn validate_app_target(raw: &str) -> Result<String, DbError> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 1024 {
        return Err(deny("app target is required"));
    }
    if raw.contains(['\n', '\r', '\0']) || raw.starts_with('-') {
        return Err(deny("app target contains invalid characters"));
    }
    if raw.contains('/') || raw.starts_with('~') {
        if !raw.ends_with(".app") {
            return Err(deny("app paths must end with .app"));
        }
        let path = Path::new(raw);
        if path.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(deny("app path must not contain '..'"));
        }
        return Ok(raw.to_string());
    }
    Ok(raw.to_string())
}

pub fn validate_imap_host(raw: &str) -> Result<String, DbError> {
    let host = raw.trim();
    if host.is_empty() || host.len() > 253 {
        return Err(deny("IMAP host is invalid"));
    }
    if host.contains(['/', '\\', ' ', '\n', '\r', '\t', '@', '\0']) {
        return Err(deny("IMAP host contains invalid characters"));
    }
    Ok(host.to_string())
}

pub fn validate_health_export_path(raw: &str) -> Result<(), DbError> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 1024 {
        return Err(deny("Health export path is required"));
    }
    if raw.contains(['\n', '\r', '\0']) {
        return Err(deny("Health export path contains invalid characters"));
    }
    let path = Path::new(raw);
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(deny("Health export path must not contain '..'"));
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext != "xml" && ext != "zip" {
        return Err(deny(
            "Health export must be an Apple Health export.xml or .zip",
        ));
    }
    Ok(())
}

/// Keys the generic get/set settings IPC may touch. Secrets use dedicated commands.
pub fn is_generic_setting_key(key: &str) -> bool {
    matches!(key, "dashboard.layout.v1")
}

pub fn is_secret_setting_key(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    k.contains("password")
        || k.contains("token")
        || k.contains("secret")
        || k.contains("api_key")
        || k == "streaming.tmdb_api_key"
        || k == "home.blink_email"
}

pub fn require_generic_setting_key(key: &str) -> Result<(), DbError> {
    if is_generic_setting_key(key) {
        Ok(())
    } else {
        Err(deny("setting key is not writable via this command"))
    }
}

/// True when `candidate` resolves inside `root` (after canonicalize).
pub fn path_is_within(root: &Path, candidate: &Path) -> bool {
    let Ok(root) = root.canonicalize() else {
        return false;
    };
    let Ok(candidate) = candidate.canonicalize() else {
        return false;
    };
    candidate.starts_with(root)
}

pub fn blocked_http_client_redirect(previous: usize, next: &Url) -> Result<(), &'static str> {
    if previous >= 4 {
        return Err("too many redirects");
    }
    if parse_public_http_url(next.as_str()).is_err() {
        return Err("blocked redirect target");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_url_allows_https_and_message() {
        assert!(validate_open_url("https://example.com/a").is_ok());
        assert!(validate_open_url("message://%3Cid%3E").is_ok());
        assert!(validate_open_url("calshow:123").is_ok());
        assert!(validate_open_url("imessage://+15551212").is_ok());
    }

    #[test]
    fn open_url_blocks_dangerous_schemes() {
        assert!(validate_open_url("file:///etc/passwd").is_err());
        assert!(validate_open_url("javascript:alert(1)").is_err());
        assert!(validate_open_url("data:text/html,hi").is_err());
        assert!(validate_open_url("smb://fileserver/share").is_err());
        assert!(validate_open_url("http://127.0.0.1/").is_err());
        assert!(validate_open_url("https://localhost/secret").is_err());
    }

    #[test]
    fn app_target_requires_bundle_for_paths() {
        assert!(validate_app_target("Safari").is_ok());
        assert!(validate_app_target("com.apple.Safari").is_ok());
        assert!(validate_app_target("/Applications/Safari.app").is_ok());
        assert!(validate_app_target("/etc/passwd").is_err());
        assert!(validate_app_target("/tmp/../Applications/Safari.app").is_err());
        assert!(validate_app_target("-a Calculator").is_err());
    }

    #[test]
    fn feed_url_blocks_ssrf() {
        assert!(validate_feed_url("https://feeds.bbci.co.uk/rss.xml").is_ok());
        assert!(validate_feed_url("http://169.254.169.254/latest/meta-data").is_err());
        assert!(validate_feed_url("http://192.168.1.1/feed").is_err());
        assert!(validate_feed_url("file:///etc/passwd").is_err());
        assert!(validate_feed_url("https://user:pass@example.com/feed").is_err());
    }

    #[test]
    fn health_path_must_be_export() {
        assert!(validate_health_export_path("/Users/me/export.xml").is_ok());
        assert!(validate_health_export_path("/Users/me/export.zip").is_ok());
        assert!(validate_health_export_path("/Users/me/Library/Messages/chat.db").is_err());
        assert!(validate_health_export_path("../export.xml").is_err());
    }

    #[test]
    fn setting_keys_are_scoped() {
        assert!(is_generic_setting_key("dashboard.layout.v1"));
        assert!(!is_generic_setting_key("streaming.tmdb_api_key"));
        assert!(is_secret_setting_key("streaming.tmdb_api_key"));
        assert!(is_secret_setting_key("home.blink_email"));
    }
}
