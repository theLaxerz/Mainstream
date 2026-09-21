//! Shared input validation for IPC-facing commands.
//!
//! Defense in depth: the webview is local-first, but RSS, email HTML, and a
//! missing CSP historically meant untrusted strings could reach `open`, HTTP
//! clients, and the filesystem.

use crate::db::DbError;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use std::path::{Component, Path, PathBuf};
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
const MAX_XML_ELEMENT_DEPTH: usize = 64;
/// Cap zip directory walks so a hostile entry count cannot wrap a 32-bit
/// allocator (CVE-2026-90783 MKVToolNix ODML superindex class).
const MAX_ZIP_ENTRIES_SCANNED: usize = 100_000;

pub fn max_http_response_bytes() -> u64 {
    MAX_HTTP_RESPONSE_BYTES
}

pub fn max_health_export_bytes() -> u64 {
    MAX_HEALTH_EXPORT_BYTES
}

pub fn max_zip_entries_scanned() -> usize {
    MAX_ZIP_ENTRIES_SCANNED
}

pub fn max_xml_element_depth() -> usize {
    MAX_XML_ELEMENT_DEPTH
}

pub fn deny(msg: impl Into<String>) -> DbError {
    DbError::Message(msg.into())
}

/// C0, C1, DEL, NEL, and Unicode line/paragraph separators.
///
/// Same class as GHSA-w27v-7q3p-w38r (browsers strip TAB/CR/LF from URL
/// schemes before parsing), GHSA-jxfw-x594-9x9m (U+2028 / U+2029 log/header
/// breaks), and CVE-2026-90773 (`procs` OSC/CSI via C1 `U+009B` / ESC).
fn contains_disallowed_controls(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(
            c,
            '\0'..='\x1f' | '\u{7f}'..='\u{9f}' | '\u{2028}' | '\u{2029}'
        )
    })
}

/// Percent-encoded `.` / `/` / `\` / NUL, a leftover `%25` (double-decode),
/// and Unicode slash lookalikes.
/// GitLab CVE-2026-85706 (KEV) and Mockoon CVE-2026-59149 confined paths with
/// a prefix/join check and missed encoded `file.Path` escapes.
/// Canva CVE-2026-92839 / Open WebUI CVE-2026-54017: `%252e` survives a
/// single `%2e` denylist and becomes `.` after a second percent-decode.
fn contains_encoded_path_escape(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    lower.contains("%2e")
        || lower.contains("%2f")
        || lower.contains("%5c")
        || lower.contains("%00")
        || lower.contains("%25")
        || s.contains('\\')
        || s.contains('\u{2215}') // division slash
        || s.contains('\u{2044}') // fraction slash
        || s.contains('\u{ff0f}') // fullwidth solidus
}

/// Markup that Electron gsubs interpolated into HTML from a filename
/// (CVE-2026-90491). React escapes, but zip/cache names must not carry a
/// DOM/attribute breakout if they later reach a non-JSX sink.
fn contains_html_markup(s: &str) -> bool {
    s.contains(['<', '>', '"', '\'', '`'])
}

/// getID3 [CVE-2026-94106](https://nvd.nist.gov/vuln/detail/CVE-2026-94106)
/// concatenated filenames into `shell_exec`. We pass argv, but zip/cache
/// names still must not carry shell metacharacters if a later caller
/// interpolates them into AppleScript or a command string.
fn contains_shell_metacharacters(s: &str) -> bool {
    s.contains(['$', ';', '|', '&', '(', ')', '*', '?', '[', ']', '{', '}', '!'])
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
    let o = ip.octets();
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_unspecified()
        || ip.octets() == [255, 255, 255, 255]
        || o[0] == 0
        // Carrier-grade NAT 100.64.0.0/10
        || (o[0] == 100 && (64..=127).contains(&o[1]))
        // IETF protocol assignments 192.0.0.0/24 (includes 192.0.0.8 etc.)
        || (o[0] == 192 && o[1] == 0 && o[2] == 0)
        // Deprecated 6to4 anycast 192.88.99.0/24
        || (o[0] == 192 && o[1] == 88 && o[2] == 99)
        // Benchmarking 198.18.0.0/15
        || (o[0] == 198 && (18..=19).contains(&o[1]))
        // Reserved / class E 240.0.0.0/4 (also covers broadcast 255.255.255.255)
        || o[0] >= 240
        // Azure WireServer / IMDS fabric (not RFC1918; used for instance metadata).
        || is_azure_wireserver_ipv4(ip)
}

/// Azure `168.63.129.16` — host agent / WireServer, same role as IMDS.
fn is_azure_wireserver_ipv4(ip: Ipv4Addr) -> bool {
    ip.octets() == [168, 63, 129, 16]
}

fn is_non_public_ipv6(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_non_public_ipv4(v4);
    }
    // Deprecated IPv4-compatible ::a.b.c.d (not :: or ::1).
    if let Some(v4) = ipv4_compatible(ip) {
        return is_non_public_ipv4(v4);
    }
    if let Some(v4) = sixto4_embedded_ipv4(ip) {
        return is_non_public_ipv4(v4);
    }
    if let Some(v4) = teredo_client_ipv4(ip) {
        return is_non_public_ipv4(v4);
    }
    // RFC 8215 local-use NAT64 64:ff9b:1::/48 — never a public fetch target.
    if is_local_use_nat64(ip) {
        return true;
    }
    if let Some(v4) = nat64_embedded_ipv4(ip) {
        return is_non_public_ipv4(v4);
    }
    if let Some(v4) = isatap_embedded_ipv4(ip) {
        return is_non_public_ipv4(v4);
    }
    ip.is_loopback()
        || ip.is_multicast()
        || ip.is_unspecified()
        || (ip.segments()[0] & 0xfe00) == 0xfc00 // unique local fc00::/7
        || (ip.segments()[0] & 0xffc0) == 0xfe80 // link-local fe80::/10
        || (ip.segments()[0] & 0xffc0) == 0xfec0 // deprecated site-local fec0::/10
}

fn ipv4_compatible(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    let s = ip.segments();
    if s[0] == 0 && s[1] == 0 && s[2] == 0 && s[3] == 0 && s[4] == 0 && s[5] == 0 {
        let v4 = Ipv4Addr::new(
            (s[6] >> 8) as u8,
            (s[6] & 0xff) as u8,
            (s[7] >> 8) as u8,
            (s[7] & 0xff) as u8,
        );
        if !v4.is_unspecified() && !v4.is_loopback() {
            return Some(v4);
        }
    }
    None
}

/// 6to4 2002:V4ADDR::/48 — treat the embedded IPv4 as the real destination.
fn sixto4_embedded_ipv4(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    let s = ip.segments();
    if s[0] == 0x2002 {
        Some(Ipv4Addr::new(
            (s[1] >> 8) as u8,
            (s[1] & 0xff) as u8,
            (s[2] >> 8) as u8,
            (s[2] & 0xff) as u8,
        ))
    } else {
        None
    }
}

/// Teredo 2001:0000::/32 — client IPv4 is the last 32 bits, XOR 0xffffffff.
fn teredo_client_ipv4(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    let s = ip.segments();
    if s[0] == 0x2001 && s[1] == 0 {
        let raw = ((s[6] as u32) << 16) | (s[7] as u32);
        Some(Ipv4Addr::from(raw ^ 0xffff_ffff))
    } else {
        None
    }
}

/// RFC 6052 well-known NAT64 prefix 64:ff9b::/96 — IPv4 is the last 32 bits.
/// Same class as GHSA-88jm-rxg9-3v5r / GHSA-pph6-vfjv-vpjw / CVE-2026-48737:
/// `64:ff9b::a9fe:a9fe` is 169.254.169.254 behind a NAT64 gateway.
fn nat64_embedded_ipv4(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    let s = ip.segments();
    if s[0] == 0x64 && s[1] == 0xff9b && s[2] == 0 && s[3] == 0 && s[4] == 0 && s[5] == 0 {
        Some(Ipv4Addr::new(
            (s[6] >> 8) as u8,
            (s[6] & 0xff) as u8,
            (s[7] >> 8) as u8,
            (s[7] & 0xff) as u8,
        ))
    } else {
        None
    }
}

fn is_local_use_nat64(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    s[0] == 0x64 && s[1] == 0xff9b && s[2] == 1
}

/// ISATAP `*::5efe:a.b.c.d` — IPv4 is the last 32 bits.
fn isatap_embedded_ipv4(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    let s = ip.segments();
    if s[5] == 0x5efe {
        Some(Ipv4Addr::new(
            (s[6] >> 8) as u8,
            (s[6] & 0xff) as u8,
            (s[7] >> 8) as u8,
            (s[7] & 0xff) as u8,
        ))
    } else {
        None
    }
}

fn host_is_blocked(host: &str) -> bool {
    let host = host.trim().trim_matches(|c| c == '[' || c == ']');
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    // Fail closed on raw Unicode hosts (homographs of localhost / metadata).
    // `url` IDNA-encodes international domains to ASCII punycode first, so
    // legitimate feeds still work. Same class as GHSA-wmmp-3585-3rmp.
    if !host.is_ascii() {
        return true;
    }
    if host.is_empty()
        || host == "localhost"
        || host == "localhost.localdomain"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host.ends_with(".onion")
        || host == "metadata.google.internal"
        || host == "instance-data"
        || is_container_or_cluster_hostname(&host)
    {
        return true;
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        return is_non_public_ip(ip);
    }
    // Browsers and some HTTP stacks accept decimal/hex/short IPv4 that the
    // `url` crate leaves as a domain (`2130706433` → 127.0.0.1). Same class
    // as the Sep 2026 link-preview-js DNS-rebinding SSRF bypass (CVE-2026-61704).
    if let Some(v4) = parse_obscured_ipv4(&host) {
        return is_non_public_ipv4(v4);
    }
    false
}

/// In-cluster Kubernetes DNS and Docker Desktop helper names. Not covered by
/// `.internal` / `.local` suffixes (`kubernetes.default`, `kubernetes.default.svc`).
fn is_container_or_cluster_hostname(host: &str) -> bool {
    host == "kubernetes"
        || host == "kubernetes.default"
        || host == "kubernetes.default.svc"
        || host == "kubernetes.default.svc.cluster.local"
        || host.ends_with(".svc.cluster.local")
        || host == "host.docker.internal"
        || host == "gateway.docker.internal"
        || host == "kubernetes.docker.internal"
        || host == "docker.for.mac.localhost"
        || host == "docker.for.mac.host.internal"
        || host == "host.containers.internal"
        || host.ends_with(".docker.internal")
}

/// Parse IPv4 forms that `Ipv4Addr::from_str` rejects: a 32-bit dword,
/// `0x…` hex, octal octets, and 1–3 dotted parts (`127.1` → 127.0.0.1).
fn parse_obscured_ipv4(host: &str) -> Option<Ipv4Addr> {
    let host = host.trim().trim_end_matches('.');
    if host.is_empty() || host.len() > 21 {
        return None;
    }
    if let Some(hex) = host.strip_prefix("0x") {
        if hex.is_empty() || hex.len() > 8 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        return u32::from_str_radix(hex, 16).ok().map(Ipv4Addr::from);
    }
    let parts: Vec<&str> = host.split('.').collect();
    if parts.is_empty() || parts.len() > 4 {
        return None;
    }
    fn part_value(part: &str) -> Option<u32> {
        if part.is_empty() || part.len() > 11 {
            return None;
        }
        if part.len() > 1
            && part.starts_with('0')
            && part.bytes().all(|b| (b'0'..=b'7').contains(&b))
        {
            return u32::from_str_radix(part, 8).ok();
        }
        if part.bytes().all(|b| b.is_ascii_digit()) {
            return part.parse().ok();
        }
        None
    }
    let nums: Vec<u32> = parts.into_iter().map(part_value).collect::<Option<_>>()?;
    match nums.as_slice() {
        [a] => Some(Ipv4Addr::from(*a)),
        [a, b] if *a <= 0xff && *b <= 0x00ff_ffff => Some(Ipv4Addr::new(
            *a as u8,
            ((b >> 16) & 0xff) as u8,
            ((b >> 8) & 0xff) as u8,
            (b & 0xff) as u8,
        )),
        [a, b, c] if *a <= 0xff && *b <= 0xff && *c <= 0xffff => Some(Ipv4Addr::new(
            *a as u8,
            *b as u8,
            ((c >> 8) & 0xff) as u8,
            (c & 0xff) as u8,
        )),
        [a, b, c, d] if nums.iter().all(|n| *n <= 0xff) => {
            Some(Ipv4Addr::new(*a as u8, *b as u8, *c as u8, *d as u8))
        }
        _ => None,
    }
}

/// Resolve `url`'s host and reject if any address is non-public.
/// Call this immediately before an HTTP fetch of an untrusted URL (RSS, email
/// images). Hostname allowlisting alone is not enough: a public-looking name
/// can rebind to loopback or link-local between parse and connect.
pub fn ensure_public_resolved_host(url: &Url) -> Result<(), DbError> {
    let host = url
        .host_str()
        .ok_or_else(|| deny("URL is missing a host"))?;
    if host_is_blocked(host) {
        return Err(deny("URL host is not allowed"));
    }
    if host.parse::<IpAddr>().is_ok() || parse_obscured_ipv4(host).is_some() {
        return Ok(());
    }
    let port = url.port_or_known_default().unwrap_or(443);
    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|_| deny("URL host could not be resolved"))?;
    let mut saw = false;
    for addr in addrs {
        saw = true;
        if is_non_public_ip(addr.ip()) {
            return Err(deny("URL host resolved to a private address"));
        }
    }
    if !saw {
        return Err(deny("URL host could not be resolved"));
    }
    Ok(())
}

/// IMAP connect-time check: a hostname that looks public can still resolve to
/// loopback / link-local / metadata. RFC1918 stays allowed for LAN mail servers.
pub fn ensure_imap_host_resolves_safely(host: &str) -> Result<(), DbError> {
    if host.parse::<IpAddr>().is_ok() || parse_obscured_ipv4(host).is_some() {
        return Ok(());
    }
    let addrs = (host, 993u16)
        .to_socket_addrs()
        .map_err(|_| deny("IMAP host could not be resolved"))?;
    let mut saw = false;
    for addr in addrs {
        saw = true;
        if imap_resolved_ip_is_blocked(addr.ip()) {
            return Err(deny("IMAP host resolved to a blocked address"));
        }
    }
    if !saw {
        return Err(deny("IMAP host could not be resolved"));
    }
    Ok(())
}

fn imap_resolved_ip_is_blocked(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_multicast()
                || is_azure_wireserver_ipv4(v4)
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return imap_resolved_ip_is_blocked(IpAddr::V4(v4));
            }
            if is_local_use_nat64(v6) {
                return true;
            }
            if let Some(v4) = nat64_embedded_ipv4(v6) {
                return imap_resolved_ip_is_blocked(IpAddr::V4(v4));
            }
            v6.is_loopback()
                || v6.is_multicast()
                || v6.is_unspecified()
                || (v6.segments()[0] & 0xffc0) == 0xfe80
        }
    }
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
    http_client_with_redirect(timeout_secs, user_agent, |previous, next| {
        blocked_http_client_redirect(previous.len(), next)
    })
}

/// HTTP client for requests that carry tokens, cookies, or API keys.
///
/// Same class as CVE-2026-88013 (rclone) and CVE-2026-63481 (Hurl): default
/// redirect following forwards `Authorization` / cookies / query secrets to a
/// different host, and HTTPS→HTTP drops them onto the wire.
pub fn authenticated_http_client(
    timeout_secs: u64,
    user_agent: Option<&str>,
) -> Result<reqwest::blocking::Client, DbError> {
    http_client_with_redirect(timeout_secs, user_agent, blocked_authenticated_redirect)
}

fn http_client_with_redirect(
    timeout_secs: u64,
    user_agent: Option<&str>,
    policy: fn(&[Url], &Url) -> Result<(), &'static str>,
) -> Result<reqwest::blocking::Client, DbError> {
    let mut builder = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .redirect(reqwest::redirect::Policy::custom(
            move |attempt| match policy(attempt.previous(), attempt.url()) {
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

/// Reject a declared `Content-Length` that would allocate past our read cap
/// before the body is copied (Apache MINA [CVE-2026-47321](https://nvd.nist.gov/vuln/detail/CVE-2026-47321)
/// / Checkmk [CVE-2026-77021](https://nvd.nist.gov/vuln/detail/CVE-2026-77021)
/// unbounded inflate class). Chunked bodies are still bounded by `copy_http_body`.
pub fn http_declared_length_is_safe(content_length: Option<u64>) -> bool {
    match content_length {
        None => true,
        Some(len) => len <= MAX_HTTP_RESPONSE_BYTES,
    }
}

/// Copy an HTTP body with a hard byte ceiling. `reqwest` `.bytes()` / `.text()` /
/// `.json()` otherwise buffer the full response, including a gzip/deflate
/// payload if compression is ever enabled on the client.
pub fn copy_http_body(resp: reqwest::blocking::Response) -> Result<Vec<u8>, DbError> {
    if !http_declared_length_is_safe(resp.content_length()) {
        return Err(deny("HTTP response is too large"));
    }
    let mut limited = Read::take(resp, MAX_HTTP_RESPONSE_BYTES.saturating_add(1));
    let mut buf = Vec::new();
    limited
        .read_to_end(&mut buf)
        .map_err(|e| deny(format!("HTTP body: {e}")))?;
    if buf.len() as u64 > MAX_HTTP_RESPONSE_BYTES {
        return Err(deny("HTTP response exceeded size cap"));
    }
    Ok(buf)
}

pub fn copy_http_text(resp: reqwest::blocking::Response) -> Result<String, DbError> {
    let bytes = copy_http_body(resp)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub fn copy_http_json<T: serde::de::DeserializeOwned>(
    resp: reqwest::blocking::Response,
) -> Result<T, DbError> {
    let bytes = copy_http_body(resp)?;
    serde_json::from_slice(&bytes).map_err(|e| deny(format!("json: {e}")))
}

/// HTTP(S) URL that is safe to fetch from an untrusted source (RSS, email HTML).
pub fn parse_public_http_url(raw: &str) -> Result<Url, DbError> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 2048 {
        return Err(deny("URL is empty or too long"));
    }
    if contains_disallowed_controls(raw) {
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
    if contains_disallowed_controls(raw) {
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
    } else if scheme_l == "mailto" {
        ensure_safe_mailto(raw)?;
    }
    Ok(raw.to_string())
}

/// Mail.app / GNOME Evolution class (CVE-2026-88859, CVE-2020-11879):
/// `mailto:?attach=` hands local files to the mail composer, and nested
/// `javascript:` / `data:` values can be assigned to iframe `src` after decode.
fn ensure_safe_mailto(raw: &str) -> Result<(), DbError> {
    let url = Url::parse(raw).map_err(|_| deny("invalid mailto URL"))?;
    if url.scheme() != "mailto" {
        return Err(deny("blocked URL scheme"));
    }
    if nested_dangerous_scheme(url.path()) {
        return Err(deny("mailto contains a blocked nested scheme"));
    }
    for (key, value) in url.query_pairs() {
        let key_l = key.to_ascii_lowercase();
        if matches!(key_l.as_str(), "attach" | "attachment" | "attachments") {
            return Err(deny("mailto must not attach local files"));
        }
        if nested_dangerous_scheme(&value) {
            return Err(deny("mailto contains a blocked nested scheme"));
        }
    }
    Ok(())
}

fn nested_dangerous_scheme(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    if nested_dangerous_scheme_needles(&lower) {
        return true;
    }
    // Canva CVE-2026-92839: deeplink handlers that percent-decode twice turn
    // `javascript%253a` into `javascript:` after the second pass. `Url`
    // query_pairs already decode once; decode once more and re-check.
    let again = percent_decode_once(&lower);
    again != lower && nested_dangerous_scheme_needles(&again)
}

fn nested_dangerous_scheme_needles(s: &str) -> bool {
    s.contains("javascript:")
        || s.contains("vbscript:")
        || s.contains("file://")
        || s.contains("data:text/")
        || s.contains("data:image/")
        || s.contains("data:application/")
}

fn percent_decode_once(s: &str) -> String {
    urlencoding::decode(s)
        .map(|decoded| decoded.into_owned())
        .unwrap_or_else(|_| s.to_string())
}

/// True when `src` is a `data:` URL whose media type is an image.
///
/// `@nuxtjs/mdc` [CVE-2026-63671](https://github.com/advisories/GHSA-mxm6-v9r6-r94c)
/// compared `data:text/html` against `url.protocol` (`"data:"`), so the MIME
/// deny-list never matched. Require the type between `data:` and the first
/// `;` or `,` to be `image/<subtype>` — not a substring `image/` later in the
/// payload.
pub fn is_inline_image_data_url(src: &str) -> bool {
    let lower = src.trim().to_ascii_lowercase();
    let Some(rest) = lower.strip_prefix("data:") else {
        return false;
    };
    let media = rest.split([';', ',']).next().unwrap_or("");
    media.starts_with("image/") && media.len() > "image/".len()
}

pub fn validate_app_target(raw: &str) -> Result<String, DbError> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 1024 {
        return Err(deny("app target is required"));
    }
    if contains_disallowed_controls(raw) || raw.starts_with('-') {
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
    if contains_disallowed_controls(host) || host.contains(['/', '\\', ' ', '@', '%']) {
        return Err(deny("IMAP host contains invalid characters"));
    }
    // Block loopback / link-local / metadata. RFC1918 is allowed so a user can
    // still point IMAP at a NAS or self-hosted mail server on their LAN.
    if imap_host_is_blocked(host) {
        return Err(deny("IMAP host is not allowed"));
    }
    Ok(host.to_string())
}

fn imap_host_is_blocked(host: &str) -> bool {
    let host = host.trim().trim_matches(|c| c == '[' || c == ']');
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if !host.is_ascii() {
        return true;
    }
    if host.is_empty()
        || host == "localhost"
        || host == "localhost.localdomain"
        || host.ends_with(".localhost")
        || host.ends_with(".onion")
        || host == "metadata.google.internal"
        || host == "instance-data"
        || is_container_or_cluster_hostname(&host)
    {
        return true;
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        return imap_resolved_ip_is_blocked(ip);
    }
    if let Some(v4) = parse_obscured_ipv4(&host) {
        return imap_resolved_ip_is_blocked(IpAddr::V4(v4));
    }
    false
}

/// Implicit TLS IMAP only. Port 143 (STARTTLS) and privileged ports are rejected
/// so a compromised webview cannot probe SMTP/HTTP/SMB via `imap::connect`.
pub fn validate_imap_port(port: u16) -> Result<u16, DbError> {
    if port == 993 {
        Ok(port)
    } else {
        Err(deny("IMAP port must be 993 (IMAPS)"))
    }
}

/// Mailbox name handed to IMAP `SELECT`. Rejects protocol metacharacters.
pub fn validate_imap_mailbox(raw: &str) -> Result<String, DbError> {
    let name = raw.trim();
    if name.is_empty() || name.len() > 255 {
        return Err(deny("IMAP mailbox is invalid"));
    }
    if contains_disallowed_controls(name) || name.contains(['"', '\\']) {
        return Err(deny("IMAP mailbox contains invalid characters"));
    }
    Ok(name.to_string())
}

/// IMAP LOGIN / XOAUTH2 username. Rejects protocol metacharacters.
pub fn validate_imap_user(raw: &str) -> Result<String, DbError> {
    let user = raw.trim();
    if user.is_empty() || user.len() > 320 {
        return Err(deny("IMAP username is invalid"));
    }
    // Also reject SASL separator `\x01` used in XOAUTH2 `user=…\x01auth=Bearer…`.
    if contains_disallowed_controls(user) || user.contains(['"', '\u{0001}']) {
        return Err(deny("IMAP username contains invalid characters"));
    }
    Ok(user.to_string())
}

/// IMAP LOGIN password. Same class of bug as RUSTSEC-2026-0271 (FTP CRLF): a
/// password containing CR/LF terminates the LOGIN line and smuggles a second
/// IMAP command on the authenticated connection.
pub fn validate_imap_password(raw: &str) -> Result<String, DbError> {
    let password = raw.trim();
    if password.is_empty() || password.len() > 512 {
        return Err(deny("IMAP password is invalid"));
    }
    if contains_disallowed_controls(password) {
        return Err(deny("IMAP password contains invalid characters"));
    }
    Ok(password.to_string())
}

/// OAuth access/refresh tokens interpolated into XOAUTH2 or HTTP bodies.
pub fn validate_oauth_token(raw: &str) -> Result<String, DbError> {
    let token = raw.trim();
    if token.is_empty() || token.len() > 8192 {
        return Err(deny("OAuth token is invalid"));
    }
    if contains_disallowed_controls(token) || token.contains('\u{0001}') {
        return Err(deny("OAuth token contains invalid characters"));
    }
    Ok(token.to_string())
}

/// YouTube channel IDs / handles after URL stripping.
pub fn validate_youtube_channel_id(raw: &str) -> Result<String, DbError> {
    let id = raw.trim();
    if id.is_empty() || id.len() > 64 {
        return Err(deny("YouTube channel id is invalid"));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(deny("YouTube channel id contains invalid characters"));
    }
    Ok(id.to_string())
}

/// Filename stem for cached device images (`blink-123.jpg`).
pub fn validate_cache_file_stem(raw: &str) -> Result<String, DbError> {
    let id = raw.trim();
    if id.is_empty() || id.len() > 128 || id.contains("..") {
        return Err(deny("device id is invalid"));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(deny("device id contains invalid characters"));
    }
    Ok(id.to_string())
}

/// DNS label used inside `https://rest-{tier}.immedia-semi.com`.
pub fn is_safe_dns_label(raw: &str) -> bool {
    let s = raw.trim();
    (1..=63).contains(&s.len())
        && !s.starts_with('-')
        && !s.ends_with('-')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

pub fn validate_dns_label(raw: &str, what: &str) -> Result<String, DbError> {
    if is_safe_dns_label(raw) {
        Ok(raw.trim().to_string())
    } else {
        Err(deny(format!("{what} is invalid")))
    }
}

/// Mail.app account name interpolated into AppleScript string literals.
pub fn validate_mailapp_account_name(raw: &str) -> Result<String, DbError> {
    let name = raw.trim();
    if name.is_empty() || name.len() > 200 {
        return Err(deny("Mail account name is invalid"));
    }
    // Reject quote/backslash even though we escape them — keeps the AppleScript
    // literal boring if a compromised webview supplies the account name.
    if contains_disallowed_controls(name) || name.contains(['"', '\\']) {
        return Err(deny("Mail account name contains invalid characters"));
    }
    Ok(name.to_string())
}

/// Public HTTP(S) article URL persisted from RSS. Blocks `javascript:` / `file:`.
pub fn validate_stored_http_url(raw: &str) -> Result<String, DbError> {
    let url = parse_public_http_url(raw)?;
    Ok(url.as_str().to_string())
}

/// YouTube watch / shorts URLs stored from channel RSS.
pub fn validate_youtube_watch_url(raw: &str) -> Result<String, DbError> {
    let url = parse_public_https_url(raw)?;
    let host = url
        .host_str()
        .unwrap_or("")
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if !matches!(
        host.as_str(),
        "youtube.com" | "www.youtube.com" | "m.youtube.com" | "youtu.be" | "www.youtu.be"
    ) {
        return Err(deny("YouTube URL host is not allowed"));
    }
    Ok(url.as_str().to_string())
}

/// Google / Microsoft public OAuth client IDs pasted from IPC.
pub fn validate_oauth_client_id(raw: &str) -> Result<String, DbError> {
    let id = raw.trim();
    if id.len() < 8 || id.len() > 256 {
        return Err(deny("OAuth client ID is invalid"));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(deny("OAuth client ID contains invalid characters"));
    }
    Ok(id.to_string())
}

/// Messages chat identifier / guid handed to `open imessage://…`.
pub fn validate_imessage_ref(raw: &str) -> Result<String, DbError> {
    let s = raw.trim();
    if s.is_empty() || s.len() > 256 {
        return Err(deny("chat identifier is invalid"));
    }
    if contains_disallowed_controls(s) || s.contains(['/', '\\']) || s.contains("..") {
        return Err(deny("chat identifier contains invalid characters"));
    }
    Ok(s.to_string())
}

/// Open-Meteo geocode query / pin name.
pub fn validate_weather_query(raw: &str) -> Result<String, DbError> {
    let q = raw.trim();
    if q.len() < 2 {
        return Err(deny("search query is too short"));
    }
    if q.len() > 100 {
        return Err(deny("search query is too long"));
    }
    if contains_disallowed_controls(q) {
        return Err(deny("search query contains invalid characters"));
    }
    Ok(q.to_string())
}

pub fn validate_lat_lon(lat: f64, lon: f64) -> Result<(), DbError> {
    if !lat.is_finite() || !lon.is_finite() {
        return Err(deny("coordinates are invalid"));
    }
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        return Err(deny("coordinates are out of range"));
    }
    Ok(())
}

/// Account / merchant / category labels persisted from IPC or CSV.
pub fn validate_finance_label(raw: &str, what: &str, max_len: usize) -> Result<String, DbError> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(deny(format!("{what} is required")));
    }
    if value.len() > max_len {
        return Err(deny(format!("{what} is too long")));
    }
    if contains_disallowed_controls(value) {
        return Err(deny(format!("{what} contains invalid characters")));
    }
    Ok(value.to_string())
}

/// Optional transaction description (empty becomes None at the call site).
pub fn validate_finance_description(raw: &str) -> Result<String, DbError> {
    let value = raw.trim();
    if value.len() > 500 {
        return Err(deny("description is too long"));
    }
    if contains_disallowed_controls(value) {
        return Err(deny("description contains invalid characters"));
    }
    Ok(value.to_string())
}

/// TMDB v3 API key pasted from IPC (query-string `api_key=`).
pub fn validate_tmdb_api_key(raw: &str) -> Result<String, DbError> {
    let key = raw.trim();
    if key.len() < 8 || key.len() > 64 {
        return Err(deny("TMDB API key is invalid"));
    }
    if !key.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(deny("TMDB API key contains invalid characters"));
    }
    Ok(key.to_string())
}

/// TMDB `poster_path` (`/abc.jpg`). Prefixed onto `https://image.tmdb.org/t/p/w342`.
pub fn validate_tmdb_poster_path(raw: &str) -> Result<String, DbError> {
    let path = raw.trim();
    if !path.starts_with('/') || path.len() > 200 || path.contains("..") {
        return Err(deny("poster path is invalid"));
    }
    if !path
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '.' || c == '_' || c == '-')
    {
        return Err(deny("poster path contains invalid characters"));
    }
    Ok(format!("https://image.tmdb.org/t/p/w342{path}"))
}

/// ISO 4217-style currency code.
pub fn validate_currency_code(raw: &str) -> Result<String, DbError> {
    let code = raw.trim().to_ascii_uppercase();
    if code.len() != 3 || !code.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(deny("currency must be a 3-letter code"));
    }
    Ok(code)
}

pub fn validate_health_export_path(raw: &str) -> Result<(), DbError> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 1024 {
        return Err(deny("Health export path is required"));
    }
    if contains_disallowed_controls(raw) || contains_encoded_path_escape(raw) {
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

/// Refuse a planted symlink after the path string was already validated.
/// rustypaste CVE-2026-90774 validated the destination, then applied a
/// second name; a symlink `export.xml` → `chat.db` is the same class.
pub fn reject_symlink_path(path: &Path) -> Result<(), DbError> {
    let meta = std::fs::symlink_metadata(path).map_err(DbError::Io)?;
    if meta.file_type().is_symlink() {
        return Err(deny("path must not be a symbolic link"));
    }
    Ok(())
}

/// Join `name` under `root` and re-check confinement *after* composition.
/// rustypaste CVE-2026-90774 (2026-09-13) validated the upload directory,
/// then applied a custom filename header that escaped it.
pub fn confined_join(root: &Path, name: &str) -> Result<PathBuf, DbError> {
    if name.is_empty() || name.len() > 255 {
        return Err(deny("file name is invalid"));
    }
    if contains_disallowed_controls(name)
        || contains_encoded_path_escape(name)
        || contains_html_markup(name)
        || contains_shell_metacharacters(name)
    {
        return Err(deny("file name contains invalid characters"));
    }
    let name_path = Path::new(name);
    if name_path.is_absolute()
        || name_path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        || name_path.file_name().and_then(|n| n.to_str()) != Some(name)
    {
        return Err(deny("file name must be a single path segment"));
    }
    let candidate = root.join(name);
    // Re-validate the composed path — do not trust the pre-join stem check.
    if candidate.file_name().and_then(|n| n.to_str()) != Some(name)
        || candidate.parent() != Some(root)
    {
        return Err(deny("composed path escaped its directory"));
    }
    Ok(candidate)
}

/// Join, then if the dest already exists require it is a regular file still
/// inside `root`. Joomla SP Page Builder CVE-2026-81564/81565 renamed/uploaded
/// media without re-checking the destination after the name was applied.
pub fn confined_dest(root: &Path, name: &str) -> Result<PathBuf, DbError> {
    let candidate = confined_join(root, name)?;
    if candidate.symlink_metadata().is_ok() {
        reject_non_regular_file(&candidate)?;
        if !path_is_within(root, &candidate) {
            return Err(deny("composed path escaped its directory"));
        }
    }
    Ok(candidate)
}

/// After a cache write, refuse a dest that is now a symlink, directory, or
/// that canonicalizes outside `root` (TOCTOU rename/replace). CPython
/// CVE-2026-82049 extracted a hard link onto a planted symlink and mutated
/// a file outside the dest tree.
pub fn confirm_confined_write(root: &Path, path: &Path) -> Result<(), DbError> {
    if let Err(err) = reject_non_regular_file(path) {
        let _ = std::fs::remove_file(path);
        return Err(err);
    }
    if !path_is_within(root, path) {
        let _ = std::fs::remove_file(path);
        return Err(deny("composed path escaped its directory"));
    }
    Ok(())
}

fn reject_non_regular_file(path: &Path) -> Result<(), DbError> {
    let meta = std::fs::symlink_metadata(path).map_err(DbError::Io)?;
    if meta.file_type().is_symlink() || !meta.file_type().is_file() {
        return Err(deny("path must be a regular file"));
    }
    Ok(())
}

/// Absolute paths for macOS system binaries we spawn. Bare names follow PATH
/// (OpenTelemetry CVE-2026-81192 / CWE-426 untrusted search path).
pub const MACOS_OPEN: &str = "/usr/bin/open";
pub const MACOS_OSASCRIPT: &str = "/usr/bin/osascript";
pub const MACOS_SWIFT: &str = "/usr/bin/swift";

const MACOS_SAFE_PATH: &str = "/usr/bin:/bin:/usr/sbin:/sbin";

/// Spawn a known macOS system binary without searching `PATH`.
pub fn macos_system_command(bin: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new(bin);
    cmd.env("PATH", MACOS_SAFE_PATH);
    cmd
}

/// argv after `open` for a single URL/path operand. `--` stops option parsing
/// so a leading-dash operand cannot become `-a`/`-e` (HexStrike CVE-2026-90690
/// additional_args / MikroTik CVE-2026-86060 argument-delimiter class).
pub fn open_operand_args(operand: &str) -> [&str; 2] {
    ["--", operand]
}

pub fn open_app_args<'a>(flag: &'a str, app: &'a str) -> [&'a str; 3] {
    [flag, app, "--"]
}

pub fn open_operand_status(operand: &str) -> std::io::Result<std::process::ExitStatus> {
    macos_system_command(MACOS_OPEN)
        .args(open_operand_args(operand))
        .status()
}

pub fn open_app_status(flag: &str, app: &str) -> std::io::Result<std::process::ExitStatus> {
    macos_system_command(MACOS_OPEN)
        .args(open_app_args(flag, app))
        .status()
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

/// True when `candidate` resolves *inside* `root` (after canonicalize).
///
/// Rejects encoded `..` / slash before resolve (CVE-2026-85706 GitLab
/// `file.Path` class) and requires a proper descendant so a prefix-only
/// match on the root itself cannot pass (CVE-2026-59149 Mockoon class).
pub fn path_is_within(root: &Path, candidate: &Path) -> bool {
    let raw = candidate.to_string_lossy();
    if contains_encoded_path_escape(&raw)
        || Path::new(raw.as_ref())
            .components()
            .any(|c| matches!(c, Component::ParentDir))
    {
        return false;
    }
    let Ok(root) = root.canonicalize() else {
        return false;
    };
    let Ok(candidate) = candidate.canonicalize() else {
        return false;
    };
    candidate.starts_with(&root) && candidate != root
}

pub fn blocked_http_client_redirect(previous: usize, next: &Url) -> Result<(), &'static str> {
    if previous >= 4 {
        return Err("too many redirects");
    }
    if parse_public_http_url(next.as_str()).is_err() {
        return Err("blocked redirect target");
    }
    if ensure_public_resolved_host(next).is_err() {
        return Err("blocked redirect target");
    }
    Ok(())
}

/// Keep Bearer / cookie / query-key requests on the original HTTPS origin.
pub fn blocked_authenticated_redirect(previous: &[Url], next: &Url) -> Result<(), &'static str> {
    blocked_http_client_redirect(previous.len(), next)?;
    let Some(from) = previous.last() else {
        return Ok(());
    };
    if from.scheme() == "https" && next.scheme() != "https" {
        return Err("https downgrade");
    }
    let from_host = from.host_str().unwrap_or_default();
    let next_host = next.host_str().unwrap_or_default();
    if !from_host.eq_ignore_ascii_case(next_host) {
        return Err("cross-host redirect");
    }
    if from.port_or_known_default() != next.port_or_known_default() {
        return Err("cross-origin redirect");
    }
    Ok(())
}

/// Same class as CVE-2026-88014 (rclone zip-slip): `ends_with("export.xml")`
/// treats sibling names (`notexport.xml`) as in-namespace, and `path.Clean`
/// can hide `..`. After `ZipArchive::enclosed_name` rejects absolute/`..`
/// paths, require the basename to be exactly `export.xml`.
pub fn zip_entry_is_apple_health_export(enclosed: &Path) -> bool {
    if enclosed.as_os_str().is_empty() {
        return false;
    }
    let raw = enclosed.to_string_lossy();
    if contains_encoded_path_escape(&raw) {
        return false;
    }
    if enclosed
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return false;
    }
    enclosed
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.eq_ignore_ascii_case("export.xml"))
        .unwrap_or(false)
}

/// Raw zip header name before `enclosed_name` normalizes it.
/// Length cap is the SIPp oversized-header class (CVE-2026-90780).
pub fn zip_entry_name_is_confined(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 512
        && !contains_disallowed_controls(name)
        && !contains_encoded_path_escape(name)
        && !contains_html_markup(name)
        && !contains_shell_metacharacters(name)
        && !name.contains("..")
}

pub fn zip_entry_index_is_safe(index: usize, archive_len: usize) -> bool {
    index < archive_len && index < MAX_ZIP_ENTRIES_SCANNED
}

/// Reject members whose declared sizes wrap a 32-bit allocator or exceed
/// our read cap (CVE-2026-90783 integer-wrap allocation class).
///
/// adm-zip [CVE-2026-77301](https://github.com/advisories/GHSA-7q85-xj36-vmfc)
/// allocated from the central-directory uncompressed size before comparing
/// it to the compressed payload — a 5-byte member declaring ~1.8 GB OOMs
/// the process. Cap inflation as well as the absolute byte ceiling.
pub fn zip_member_size_is_safe(uncompressed: u64, compressed: u64) -> bool {
    const MAX_ZIP_INFLATION: u64 = 1024;
    let Some(total) = uncompressed.checked_add(compressed) else {
        return false;
    };
    let max_from_ratio = compressed
        .saturating_mul(MAX_ZIP_INFLATION)
        .saturating_add(65_536);
    uncompressed <= MAX_HEALTH_EXPORT_BYTES
        && compressed <= MAX_HEALTH_EXPORT_BYTES
        && uncompressed <= max_from_ratio
        && total >= uncompressed
        && uncompressed <= i32::MAX as u64
}

/// Refuse zip members that are not regular files. CPython CVE-2026-82049
/// (2026-09-14) let a tar hard link follow a planted symlink out of the
/// extract tree; zip Unix extra fields can encode the same special types.
pub fn zip_member_is_regular_file(is_symlink: bool, unix_mode: Option<u32>) -> bool {
    const S_IFMT: u32 = 0o170000;
    const S_IFREG: u32 = 0o100000;
    if is_symlink {
        return false;
    }
    match unix_mode {
        None => true,
        Some(mode) => {
            let file_type = mode & S_IFMT;
            file_type == 0 || file_type == S_IFREG
        }
    }
}

/// getID3 [CVE-2026-94108](https://nvd.nist.gov/vuln/detail/CVE-2026-94108)
/// expanded external entities from media-file XML. quick-xml does not fetch
/// SYSTEM ids, but a DTD still lets an import declare entities. Fail closed
/// on DTD markup and UTF-16 BOMs (Expat [CVE-2026-93990](https://nvd.nist.gov/vuln/detail/CVE-2026-93990)
/// unpaired-surrogate class).
pub fn xml_payload_forbids_dtd(head: &[u8]) -> bool {
    if head.starts_with(&[0xff, 0xfe]) || head.starts_with(&[0xfe, 0xff]) {
        return false;
    }
    let head = if head.starts_with(&[0xef, 0xbb, 0xbf]) {
        &head[3..]
    } else {
        head
    };
    let Ok(text) = std::str::from_utf8(head) else {
        return false;
    };
    let upper = text.to_ascii_uppercase();
    !(upper.contains("<!DOCTYPE")
        || upper.contains("<!ENTITY")
        || upper.contains("<!ELEMENT")
        || upper.contains("<!ATTLIST")
        || upper.contains("<!NOTATION"))
}

/// Neethi [CVE-2026-91863](https://nvd.nist.gov/vuln/detail/CVE-2026-91863)
/// nested WS-Policy documents until the parser stack died. Cap Health XML
/// element depth so a hostile `export.xml` cannot do the same.
pub fn xml_element_depth_is_safe(depth: usize) -> bool {
    depth > 0 && depth <= MAX_XML_ELEMENT_DEPTH
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
        // GHSA-w27v-7q3p-w38r class: TAB/C0 stripped from schemes by browsers.
        assert!(validate_open_url("java\tscript:alert(1)").is_err());
        assert!(validate_open_url("https://example.com/\talert").is_err());
        assert!(validate_open_url("message://id\u{2028}evil").is_err());
        assert!(validate_app_target("Safari\t-a Calculator").is_err());
        // CVE-2026-88859 / CVE-2020-11879 class: nested schemes and attach=.
        assert!(validate_open_url("mailto:ada@example.com").is_ok());
        assert!(validate_open_url("mailto:ada@example.com?subject=Hi").is_ok());
        assert!(validate_open_url("mailto:?attach=/etc/passwd").is_err());
        assert!(validate_open_url("mailto:ada@example.com?attachment=file:///tmp/x").is_err());
        assert!(validate_open_url("mailto:ada@example.com?%61ttach=/tmp/secret").is_err());
        assert!(validate_open_url("mailto:ada@example.com?body=javascript:alert(1)").is_err());
        assert!(validate_open_url("mailto:javascript:alert(1)").is_err());
        // Canva CVE-2026-92839 class: one extra percent-decode must not revive a scheme.
        assert!(validate_open_url("mailto:ada@example.com?body=javascript%253aalert(1)").is_err());
        // CVE-2026-90773 class: C1 CSI / OSC (U+009B) in IPC strings.
        assert!(validate_open_url("https://example.com/\u{9b}alert").is_err());
        assert!(validate_app_target("Safari\u{9b}-a Calculator").is_err());
        // File Viewer CVE-2026-91127: escaped HTML still executed javascript: /
        // vbscript: / data: / blob: hrefs when the scheme was not allowlisted.
        assert!(validate_open_url("vbscript:msgbox(1)").is_err());
        assert!(validate_open_url("blob:https://example.com/uuid").is_err());
        assert!(validate_open_url("view-source:https://example.com").is_err());
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
        // Obscured IPv4 that the URL crate leaves as a domain name.
        assert!(validate_feed_url("http://2130706433/").is_err());
        assert!(validate_feed_url("http://127.1/").is_err());
        assert!(validate_feed_url("http://0x7f000001/").is_err());
        assert!(validate_feed_url("http://0177.0.0.1/").is_err());
        assert!(validate_feed_url("http://240.0.0.1/").is_err());
        assert!(validate_feed_url("http://198.18.0.1/").is_err());
        assert!(validate_feed_url("https://facebookcorewwwi.onion/").is_err());
        assert!(validate_feed_url("http://[2002:ac10:0101::1]/").is_err());
        // NAT64 well-known prefix wrapping metadata / loopback (hex and dotted).
        assert!(validate_feed_url("http://[64:ff9b::a9fe:a9fe]/").is_err());
        assert!(validate_feed_url("http://[64:ff9b::169.254.169.254]/").is_err());
        assert!(validate_feed_url("http://[64:ff9b::7f00:1]/").is_err());
        assert!(validate_feed_url("http://[64:ff9b:1::a9fe:a9fe]/").is_err());
        assert!(validate_feed_url("http://[64:ff9b:1::8.8.8.8]/").is_err());
        // ISATAP wrapping RFC1918 (global-looking prefix, embedded 10.0.0.1).
        assert!(validate_feed_url("http://[2001:db8::5efe:a00:1]/").is_err());
        assert!(validate_feed_url("http://instance-data/").is_err());
        // Azure WireServer / IMDS fabric + NAT64 wrap (168.63.129.16 → a83f:8110).
        assert!(validate_feed_url("http://168.63.129.16/").is_err());
        assert!(validate_feed_url("http://[64:ff9b::a83f:8110]/").is_err());
        assert!(validate_feed_url("http://kubernetes.default/").is_err());
        assert!(validate_feed_url("http://kubernetes.default.svc/").is_err());
        assert!(validate_feed_url("http://host.docker.internal/").is_err());
        // AnyIO CVE-2026-63374: IDNA 2003 mapped ß→ss so TLS matched the
        // wrong certificate. `url` must emit IDNA 2008 punycode, not `strasse`.
        let idna = parse_public_http_url("https://straße.de/rss").unwrap();
        let idna_host = idna.host_str().unwrap();
        assert!(
            idna_host.starts_with("xn--") && !idna_host.contains("strasse"),
            "IDNA 2008 punycode required, got {idna_host}"
        );
        assert!(host_is_blocked("straße.de"));
        assert!(parse_obscured_ipv4("8.8.8.8").is_some());
        assert!(!is_non_public_ipv4(parse_obscured_ipv4("8.8.8.8").unwrap()));
        assert!(is_non_public_ipv4(parse_obscured_ipv4("127.1").unwrap()));
        let loopback = Url::parse("http://127.0.0.1/secret").unwrap();
        assert!(ensure_public_resolved_host(&loopback).is_err());
        let metadata = Url::parse("http://169.254.169.254/").unwrap();
        assert!(ensure_public_resolved_host(&metadata).is_err());
    }

    #[test]
    fn health_path_must_be_export() {
        assert!(validate_health_export_path("/Users/me/export.xml").is_ok());
        assert!(validate_health_export_path("/Users/me/export.zip").is_ok());
        assert!(validate_health_export_path("/Users/me/Library/Messages/chat.db").is_err());
        assert!(validate_health_export_path("../export.xml").is_err());
        // CVE-2026-85706 / CVE-2026-59149 class: encoded traversal and prefix escapes.
        assert!(validate_health_export_path("/Users/me/%2e%2e/export.xml").is_err());
        assert!(validate_health_export_path("/Users/me/%252e%252e/export.xml").is_err());
        assert!(validate_health_export_path("/Users/me/export.xml%00.bak").is_err());
        assert!(validate_health_export_path("/Users/me/export\\..\\secret.xml").is_err());
    }

    #[test]
    fn zip_entry_requires_exact_export_xml_basename() {
        // CVE-2026-88014 class: sibling / prefix matching and `.` / `..`.
        assert!(zip_entry_is_apple_health_export(Path::new("export.xml")));
        // Caddy CVE-2026-77281 fileHidden class: macOS is case-insensitive,
        // so the basename check must not miss EXPORT.XML.
        assert!(zip_entry_is_apple_health_export(Path::new("EXPORT.XML")));
        assert!(zip_entry_is_apple_health_export(Path::new(
            "apple_health_export/export.xml"
        )));
        assert!(!zip_entry_is_apple_health_export(Path::new(
            "notexport.xml"
        )));
        assert!(!zip_entry_is_apple_health_export(Path::new(
            "apple_health_export.xml"
        )));
        assert!(!zip_entry_is_apple_health_export(Path::new(
            "../export.xml"
        )));
        assert!(!zip_entry_is_apple_health_export(Path::new("/export.xml")));
        assert!(!zip_entry_is_apple_health_export(Path::new(".")));
        assert!(!zip_entry_is_apple_health_export(Path::new(
            "apple_health_export/../export.xml"
        )));
        assert!(!zip_entry_is_apple_health_export(Path::new(
            "apple_health_export/%2e%2e/export.xml"
        )));
        assert!(zip_entry_name_is_confined("apple_health_export/export.xml"));
        assert!(!zip_entry_name_is_confined("../export.xml"));
        assert!(!zip_entry_name_is_confined("%2e%2e/export.xml"));
        assert!(!zip_entry_name_is_confined("%252e%252e/export.xml"));
        assert!(!zip_entry_name_is_confined("export.xml%00"));
        // CVE-2026-90780 class: oversized zip header names.
        assert!(!zip_entry_name_is_confined(&format!(
            "{}/export.xml",
            "a".repeat(512)
        )));
        // CVE-2026-90491 class: filename markup must not reach a DOM sink.
        assert!(!zip_entry_name_is_confined(
            "\"><img src=x onerror=alert(1)>.xml"
        ));
        assert!(!zip_entry_name_is_confined("export.xml`"));
        // getID3 CVE-2026-94106: filenames must not carry shell metacharacters.
        assert!(!zip_entry_name_is_confined("export.xml;rm -rf /"));
        assert!(!zip_entry_name_is_confined("$(whoami)/export.xml"));
        assert!(!zip_entry_name_is_confined("export.xml|id"));
        assert!(zip_entry_index_is_safe(0, 1));
        assert!(!zip_entry_index_is_safe(1, 1));
        assert_eq!(max_zip_entries_scanned(), MAX_ZIP_ENTRIES_SCANNED);
        assert!(!zip_entry_index_is_safe(
            max_zip_entries_scanned(),
            max_zip_entries_scanned() + 1
        ));
        assert!(zip_member_size_is_safe(1024, 512));
        assert!(zip_member_size_is_safe(0, 0));
        assert!(!zip_member_size_is_safe(MAX_HEALTH_EXPORT_BYTES + 1, 1));
        assert!(!zip_member_size_is_safe(1, MAX_HEALTH_EXPORT_BYTES + 1));
        assert!(!zip_member_size_is_safe(u64::MAX, 1));
        // adm-zip CVE-2026-77301: 105-byte ZIP declaring ~1.8 GB uncompressed.
        assert!(!zip_member_size_is_safe(1_774_399_200, 5));
        assert!(!zip_member_size_is_safe(2_000_000, 1));
        // CVE-2026-82049 class: special files / links must not be treated as
        // the Apple Health export.xml payload.
        assert!(zip_member_is_regular_file(false, None));
        assert!(zip_member_is_regular_file(false, Some(0o100644)));
        assert!(zip_member_is_regular_file(false, Some(0o644)));
        assert!(!zip_member_is_regular_file(true, Some(0o100644)));
        assert!(!zip_member_is_regular_file(false, Some(0o120777))); // symlink
        assert!(!zip_member_is_regular_file(false, Some(0o040755))); // directory
        assert!(!zip_member_is_regular_file(false, Some(0o010644))); // fifo
    }

    #[test]
    fn confined_join_rechecks_after_compose() {
        // CVE-2026-90774 rustypaste class: validate, then apply a second name.
        let root = Path::new("/tmp/mainstream-cache");
        assert_eq!(
            confined_join(root, "blink-123.jpg").unwrap(),
            root.join("blink-123.jpg")
        );
        assert!(confined_join(root, "../etc/passwd").is_err());
        assert!(confined_join(root, "/etc/passwd").is_err());
        assert!(confined_join(root, "foo/bar.jpg").is_err());
        assert!(confined_join(root, "foo.jpg/").is_err());
        assert!(confined_join(root, "foo.jpg\u{9b}").is_err());
        assert!(confined_join(root, "%2e%2e.jpg").is_err());
        assert!(confined_join(root, "%252e.jpg").is_err());
        assert!(confined_join(root, "").is_err());
        // CVE-2026-90491 class: HTML/attribute breakout in composed names.
        assert!(confined_join(root, "<img>.jpg").is_err());
        assert!(confined_join(root, "foo\".jpg").is_err());
        assert!(confined_join(root, "ok;rm.jpg").is_err());
        assert!(confined_join(root, "ok$(id).jpg").is_err());
        let dest = confined_dest(root, "ok.jpg").unwrap();
        assert_eq!(dest, root.join("ok.jpg"));
        assert_eq!(
            open_operand_args("https://example.com/a"),
            ["--", "https://example.com/a"]
        );
        assert_eq!(open_operand_args("-a"), ["--", "-a"]);
        assert_eq!(open_app_args("-a", "Messages"), ["-a", "Messages", "--"]);
        assert_eq!(
            open_app_args("-b", "com.apple.Safari"),
            ["-b", "com.apple.Safari", "--"]
        );
    }

    #[test]
    fn reject_symlink_after_name_check() {
        let dir = std::env::temp_dir().join(format!("ms-sec-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let target = dir.join("secret.db");
        let link = dir.join("export.xml");
        std::fs::write(&target, b"x").unwrap();
        let _ = std::fs::remove_file(&link);
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target, &link).unwrap();
            assert!(reject_symlink_path(&link).is_err());
            assert!(reject_symlink_path(&target).is_ok());
            assert!(confirm_confined_write(&dir, &link).is_err());
            assert!(confirm_confined_write(&dir, &target).is_ok());
        }
        let _ = std::fs::remove_file(&link);
        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn authenticated_redirect_stays_on_https_origin() {
        // Public literals skip DNS in `ensure_public_resolved_host`.
        let from = Url::parse("https://8.8.8.8/token").unwrap();
        let same = Url::parse("https://8.8.8.8/v2").unwrap();
        let other = Url::parse("https://1.1.1.1/steal").unwrap();
        let http = Url::parse("http://8.8.8.8/token").unwrap();
        let other_port = Url::parse("https://8.8.8.8:8443/token").unwrap();
        assert!(blocked_authenticated_redirect(std::slice::from_ref(&from), &same).is_ok());
        assert!(blocked_authenticated_redirect(std::slice::from_ref(&from), &other).is_err());
        assert!(blocked_authenticated_redirect(std::slice::from_ref(&from), &http).is_err());
        assert!(blocked_authenticated_redirect(std::slice::from_ref(&from), &other_port).is_err());
    }

    #[test]
    fn setting_keys_are_scoped() {
        assert!(is_generic_setting_key("dashboard.layout.v1"));
        assert!(!is_generic_setting_key("streaming.tmdb_api_key"));
        assert!(is_secret_setting_key("streaming.tmdb_api_key"));
        assert!(is_secret_setting_key("home.blink_email"));
    }

    #[test]
    fn imap_host_blocks_loopback_and_metadata() {
        assert!(validate_imap_host("imap.gmail.com").is_ok());
        assert!(validate_imap_host("10.0.0.5").is_ok());
        assert!(validate_imap_host("127.0.0.1").is_err());
        assert!(validate_imap_host("localhost").is_err());
        assert!(validate_imap_host("169.254.169.254").is_err());
        assert!(validate_imap_host("::1").is_err());
        assert!(validate_imap_host("metadata.google.internal").is_err());
        assert!(validate_imap_host("2130706433").is_err());
        assert!(validate_imap_host("127.1").is_err());
        assert!(validate_imap_host("mail.example.onion").is_err());
        assert!(validate_imap_host("instance-data").is_err());
        assert!(validate_imap_host("168.63.129.16").is_err());
        assert!(validate_imap_host("kubernetes.default").is_err());
        assert!(validate_imap_host("kubernetes.default.svc").is_err());
        assert!(validate_imap_host("host.docker.internal").is_err());
        assert!(validate_imap_host("localhоst").is_err());
        assert!(validate_imap_host("fe80::1%lo0").is_err());
    }

    #[test]
    fn imap_port_mailbox_and_user_are_strict() {
        assert_eq!(validate_imap_port(993).unwrap(), 993);
        assert!(validate_imap_port(143).is_err());
        assert!(validate_imap_port(25).is_err());
        assert!(validate_imap_port(80).is_err());
        assert!(validate_imap_mailbox("INBOX").is_ok());
        assert!(validate_imap_mailbox("INBOX\r\nUID SEARCH ALL").is_err());
        assert!(validate_imap_mailbox("folder\"name").is_err());
        assert!(validate_imap_user("ada@gmail.com").is_ok());
        assert!(validate_imap_user("user\r\nLOGIN evil").is_err());
        assert!(validate_imap_user("user\u{0001}auth=Bearer stolen").is_err());
        assert!(validate_imap_password("app-password-ok").is_ok());
        assert!(validate_imap_password("secret\r\nCREATE pwned").is_err());
        assert!(validate_oauth_token("ya29.a0AfH6S").is_ok());
        assert!(validate_oauth_token("tok\nsmuggle").is_err());
        assert!(validate_oauth_token("tok\u{0001}extra").is_err());
    }

    #[test]
    fn youtube_and_cache_stems_reject_path_tricks() {
        assert!(validate_youtube_channel_id("UCuAXFkgsw1L7xaCfnd5JJOw").is_ok());
        assert!(validate_youtube_channel_id("UC../etc").is_err());
        assert!(validate_youtube_channel_id("https://evil.example").is_err());
        assert_eq!(
            validate_cache_file_stem("blink-12345").unwrap(),
            "blink-12345"
        );
        assert!(validate_cache_file_stem("blink-../etc/passwd").is_err());
        assert!(validate_cache_file_stem("blink-foo/bar").is_err());
        assert!(is_safe_dns_label("prod"));
        assert!(is_safe_dns_label("u004"));
        assert!(!is_safe_dns_label("prod.evil.com"));
        assert!(!is_safe_dns_label("-bad"));
        assert!(validate_mailapp_account_name("Gmail").is_ok());
        assert!(validate_mailapp_account_name("Gmail\nmalicious").is_err());
        assert!(validate_mailapp_account_name("Gmail\"evil").is_err());
        assert!(validate_weather_query("Austin").is_ok());
        assert!(validate_weather_query("x").is_err());
        assert!(validate_weather_query(&"a".repeat(101)).is_err());
        assert!(validate_lat_lon(30.27, -97.74).is_ok());
        assert!(validate_lat_lon(91.0, 0.0).is_err());
        assert!(validate_lat_lon(0.0, 181.0).is_err());
        assert!(validate_stored_http_url("https://example.com/story").is_ok());
        assert!(validate_stored_http_url("javascript:alert(1)").is_err());
        assert!(validate_stored_http_url("file:///etc/passwd").is_err());
        assert!(validate_youtube_watch_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ").is_ok());
        assert!(validate_youtube_watch_url("https://evil.example/watch?v=x").is_err());
        assert!(validate_oauth_client_id("1234567890-abc.apps.googleusercontent.com").is_ok());
        assert!(validate_oauth_client_id("http://evil").is_err());
        assert!(validate_imessage_ref("+15551212").is_ok());
        assert!(validate_imessage_ref("iMessage;-;ada@example.com").is_ok());
        assert!(validate_imessage_ref("../Library/LaunchAgents").is_err());
        assert_eq!(validate_currency_code("usd").unwrap(), "USD");
        assert!(validate_currency_code("US").is_err());
        assert!(validate_currency_code("usd1").is_err());
        assert_eq!(
            validate_finance_label("Checking", "account name", 80).unwrap(),
            "Checking"
        );
        assert!(validate_finance_label("bad\nname", "account name", 80).is_err());
        assert!(validate_finance_label(&"x".repeat(81), "account name", 80).is_err());
        assert!(validate_finance_description("Coffee").is_ok());
        assert!(validate_finance_description("line\nbreak").is_err());
        assert!(validate_tmdb_api_key("a1b2c3d4e5f67890").is_ok());
        assert!(validate_tmdb_api_key("short").is_err());
        assert!(validate_tmdb_api_key("key&extra=1").is_err());
        assert_eq!(
            validate_tmdb_poster_path("/abc.jpg").unwrap(),
            "https://image.tmdb.org/t/p/w342/abc.jpg"
        );
        assert!(validate_tmdb_poster_path("https://evil.example/x.jpg").is_err());
        assert!(validate_tmdb_poster_path("/../etc/passwd").is_err());
        let nat64_public = "64:ff9b::808:808".parse::<Ipv6Addr>().unwrap();
        assert!(!is_non_public_ipv6(nat64_public));
        let nat64_meta = "64:ff9b::a9fe:a9fe".parse::<Ipv6Addr>().unwrap();
        assert!(is_non_public_ipv6(nat64_meta));
    }

    #[test]
    fn cargo_lock_excludes_aug2026_crates_io_poison() {
        // https://blog.rust-lang.org/2026/08/20/supply-chain-attack-on-arrayref/
        let lock = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"));
        for (name, ver) in [
            ("arrayref", "0.3.10"),
            ("internment", "0.8.7"),
            ("append-only-vec", "0.1.9"),
            // RUSTSEC-2026-0282 — double-free when Drop panics (< 0.3.1)
            ("aligned_box", "0.3.0"),
        ] {
            let stanza = format!("name = \"{name}\"\nversion = \"{ver}\"");
            assert!(
                !lock.contains(&stanza),
                "Cargo.lock must not include poisoned crate {name}@{ver}"
            );
        }
        for dropper in [
            "proc-macro1",
            "proc-macro-en",
            "aovine",
            "arone",
            "aronenao",
            "tinymember",
            // RUSTSEC-2026-0280 / 0281 — PolinRider malware yanked 2026-09-07
            "greentic-setup-dev",
            "greentic-setup",
            "greentic-start",
            "greentic-start-dev",
            "greentic-operator",
            "greentic-operator-dev",
            // RUSTSEC-2026-0283 — unmaintained; use zeroize
            "clear_on_drop",
            // RUSTSEC-2026-0284 — unsound Map::into_iter / SharedIncin::clear
            "lockfree",
            // CVE-2026-61544 / GHSA 2026-09-15 — remote panic in QUIC handshake
            "libp2p-quic",
            // RUSTSEC-2026-0286 — OOB read decoding CKA_ALLOWED_MECHANISMS
            "cryptoki",
            // RUSTSEC-2026-0287 / 0288 — cosmian_kyber unmaintained; AVX2
            // cmov skips Fujisaki-Okamoto implicit rejection (key recovery)
            "cosmian_kyber",
            // RUSTSEC-2026-0289 / 0290 — pqc_kyber unmaintained; same AVX2 FO skip
            "pqc_kyber",
            // RUSTSEC-2026-0291 — double free in OwnedAlloc::drop_in_place
            "owned-alloc",
            // RUSTSEC-2026-0292 — panic-safety UAF in Chunk/InlineArray
            "imbl-sized-chunks",
            // RUSTSEC-2026-0293 — Consumer::skip/clear double-free
            "ringbuf",
            // RUSTSEC-2026-0294 / CVE-2026-92612 — unsound UTF-8 String API
            "iceoryx2-bb-container",
            // RUSTSEC-2026-0295 — ApplyResult Clone UAF
            "z3",
        ] {
            let needle = format!("name = \"{dropper}\"");
            assert!(
                !lock.contains(&needle),
                "Cargo.lock must not include dropper crate {dropper}"
            );
        }
        // RUSTSEC-2026-0285 / GHSA-2mjx-qc3c-rqvc: rustls 0.23.13–0.23.44
        // TLS 1.3 handshake messages accepted across encryption-level boundaries.
        for patch in 13..=44 {
            let stanza = format!("name = \"rustls\"\nversion = \"0.23.{patch}\"");
            assert!(
                !lock.contains(&stanza),
                "Cargo.lock must not pin rustls 0.23.{patch} (RUSTSEC-2026-0285)"
            );
        }
    }

    #[test]
    fn macos_system_bins_are_absolute_and_not_path_looked_up() {
        for bin in [MACOS_OPEN, MACOS_OSASCRIPT, MACOS_SWIFT] {
            assert!(bin.starts_with("/usr/bin/"), "{bin} must be absolute");
            assert!(!bin.contains(".."), "{bin} must not contain '..'");
        }
        let src = include_str!("security.rs");
        assert!(src.contains("macos_system_command(MACOS_OPEN)"));
        let mail = include_str!("commands/mail.rs");
        let mailapp = include_str!("commands/email_mailapp.rs");
        assert!(mail.contains("macos_system_command(MACOS_SWIFT)"));
        assert!(!mail.contains(r#"Command::new("swift")"#));
        assert!(mailapp.contains("macos_system_command(MACOS_OSASCRIPT)"));
        assert!(!mailapp.contains(r#"Command::new("osascript")"#));
    }

    #[test]
    fn inline_image_data_url_requires_image_media_type() {
        assert!(is_inline_image_data_url("data:image/jpeg;base64,/9j/4AAQ"));
        assert!(is_inline_image_data_url("DATA:IMAGE/PNG,iVBORw0K"));
        assert!(!is_inline_image_data_url(
            "data:text/html,<img src=x>image/jpeg"
        ));
        assert!(!is_inline_image_data_url("data:text/html;base64,image/png"));
        assert!(!is_inline_image_data_url("data:image"));
        assert!(!is_inline_image_data_url("https://example.com/a.jpg"));
    }

    #[test]
    fn frontend_does_not_assign_html_sinks() {
        // Opencast CVE-2026-77615 / Vendure GHSA-xhq9-whgq-49j5: cue/description
        // text assigned to innerHTML. @platejs GHSA-qrfj-mgw8-j9c6: HTML
        // deserialization that runs browser behavior. React text nodes + CSP
        // `script-src 'self'` are the control; keep the sinks out of src/.
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src");
        let mut files = Vec::new();
        collect_frontend_sources(&root, &mut files);
        assert!(
            !files.is_empty(),
            "expected frontend sources under {}",
            root.display()
        );
        let mut hits = Vec::new();
        for path in &files {
            let text = std::fs::read_to_string(path).expect("read frontend source");
            for needle in [
                "dangerouslySetInnerHTML",
                "innerHTML",
                "outerHTML",
                "insertAdjacentHTML",
                "document.write",
                "parseFromString",
                "createContextualFragment",
                "srcdoc",
            ] {
                if text.contains(needle) {
                    hits.push(format!("{}: {needle}", path.display()));
                }
            }
        }
        assert!(
            hits.is_empty(),
            "frontend HTML sinks must stay unused: {hits:?}"
        );
        // md-editor-v3 CVE-2026-84992 interpolates fenced-code language into
        // unquoted HTML attributes. Keep attribute concatenation out of src/.
        let mut attr_hits = Vec::new();
        for path in &files {
            let text = std::fs::read_to_string(path).expect("read frontend source");
            for needle in ["language=${", "class=\"language-${", "`language="] {
                if text.contains(needle) {
                    attr_hits.push(format!("{}: {needle}", path.display()));
                }
            }
        }
        assert!(
            attr_hits.is_empty(),
            "frontend must not interpolate into HTML attributes: {attr_hits:?}"
        );
        // GHSA-xvpv-wq7p-548c (2026-09-18): npm preinstall exfil package.
        let npm_lock = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../package-lock.json"));
        for dropper in ["test890-auth"] {
            let needle = format!("\"{dropper}\"");
            assert!(
                !npm_lock.contains(&needle),
                "package-lock.json must not include malware package {dropper}"
            );
        }
    }

    #[test]
    fn health_xml_forbids_dtd_and_utf16() {
        // getID3 CVE-2026-94108 / Expat CVE-2026-93990 class.
        assert!(xml_payload_forbids_dtd(
            br#"<?xml version="1.0"?><HealthData></HealthData>"#
        ));
        assert!(xml_payload_forbids_dtd(
            b"\xEF\xBB\xBF<?xml version=\"1.0\"?><HealthData/>"
        ));
        assert!(!xml_payload_forbids_dtd(
            br#"<?xml version="1.0"?><!DOCTYPE foo [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><HealthData/>"#
        ));
        assert!(!xml_payload_forbids_dtd(
            br#"<?xml version="1.0"?><!ENTITY xxe SYSTEM "file:///etc/passwd">"#
        ));
        assert!(!xml_payload_forbids_dtd(&[0xff, 0xfe, b'<', 0, b'?', 0]));
        assert!(!xml_payload_forbids_dtd(&[0xfe, 0xff, 0, b'<', 0, b'?']));
        assert!(!xml_payload_forbids_dtd(&[0xff, 0xfe]));
        // Neethi CVE-2026-91863: nested elements must not exhaust the stack.
        assert!(xml_element_depth_is_safe(1));
        assert!(xml_element_depth_is_safe(MAX_XML_ELEMENT_DEPTH));
        assert!(!xml_element_depth_is_safe(0));
        assert!(!xml_element_depth_is_safe(MAX_XML_ELEMENT_DEPTH + 1));
        assert_eq!(max_xml_element_depth(), MAX_XML_ELEMENT_DEPTH);
        // Apache MINA CVE-2026-47321 / Checkmk CVE-2026-77021: cap declared
        // Content-Length before allocating the HTTP body.
        assert!(http_declared_length_is_safe(None));
        assert!(http_declared_length_is_safe(Some(1024)));
        assert!(http_declared_length_is_safe(Some(MAX_HTTP_RESPONSE_BYTES)));
        assert!(!http_declared_length_is_safe(Some(
            MAX_HTTP_RESPONSE_BYTES + 1
        )));
        assert!(!http_declared_length_is_safe(Some(u64::MAX)));
    }

    fn collect_frontend_sources(dir: &Path, out: &mut Vec<PathBuf>) {
        let entries = std::fs::read_dir(dir).expect("read frontend dir");
        for entry in entries {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
                collect_frontend_sources(&path, out);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| matches!(e, "ts" | "tsx" | "js" | "jsx"))
            {
                out.push(path);
            }
        }
    }
}
