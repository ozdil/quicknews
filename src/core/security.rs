use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::Duration;
use url::Url;

pub const MAX_HTTP_PAYLOAD_SIZE: usize = 2 * 1024 * 1024; // 2 MiB ceiling
pub const MAX_LOCAL_FILE_SIZE: usize = 10 * 1024 * 1024; // 10 MiB ceiling

#[derive(Debug)]
pub enum SecurityError {
    InvalidScheme(String),
    InvalidHost(String),
    InvalidPort(u16),
    InvalidPath(String),
    HostResolutionFailed(String),
    SsrfBlocked(String),
    SymlinkForbidden(PathBuf),
    PayloadTooLarge(usize),
    Io(std::io::Error),
    Network(String),
    UrlParse(url::ParseError),
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::InvalidScheme(s) => write!(f, "Guvenlik hatasi: gecersiz protokol {}", s),
            SecurityError::InvalidHost(h) => write!(f, "Guvenlik hatasi: gecersiz ana bilgisayar {}", h),
            SecurityError::InvalidPort(p) => write!(f, "Guvenlik hatasi: guvensiz veya yasakli port {}", p),
            SecurityError::InvalidPath(p) => write!(f, "Guvenlik hatasi: gecersiz dosya yolu {}", p),
            SecurityError::HostResolutionFailed(h) => write!(f, "Guvenlik hatasi: ana bilgisayar cozumlenemedi {}", h),
            SecurityError::SsrfBlocked(ip) => write!(f, "SSRF engeli: ozel veya yerel ag erisimi yasak ({})", ip),
            SecurityError::SymlinkForbidden(p) => write!(f, "Guvenlik engeli: sembolik bag yasak ({})", p.display()),
            SecurityError::PayloadTooLarge(sz) => write!(f, "Boyut asimi: veri siniri asildi ({} bayt)", sz),
            SecurityError::Io(e) => write!(f, "Girdi/cikti hatasi: {}", e),
            SecurityError::Network(e) => write!(f, "Ag hatasi: {}", e),
            SecurityError::UrlParse(e) => write!(f, "URL ayrirma hatasi: {}", e),
        }
    }
}

impl std::error::Error for SecurityError {}

impl From<std::io::Error> for SecurityError {
    fn from(e: std::io::Error) -> Self {
        SecurityError::Io(e)
    }
}

impl From<url::ParseError> for SecurityError {
    fn from(e: url::ParseError) -> Self {
        SecurityError::UrlParse(e)
    }
}

/// Checks if an IPv4 address belongs to a private, loopback, link-local, or reserved range.
pub fn is_private_or_reserved_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();

    // 0.0.0.0/8 - Current network
    if octets[0] == 0 {
        return true;
    }
    // 10.0.0.0/8 - RFC 1918 Private
    if octets[0] == 10 {
        return true;
    }
    // 127.0.0.0/8 - Loopback
    if octets[0] == 127 {
        return true;
    }
    // 169.254.0.0/16 - Link Local & Cloud Metadata (RFC 3927)
    if octets[0] == 169 && octets[1] == 254 {
        return true;
    }
    // 172.16.0.0/12 - RFC 1918 Private
    if octets[0] == 172 && (octets[1] >= 16 && octets[1] <= 31) {
        return true;
    }
    // 192.168.0.0/16 - RFC 1918 Private
    if octets[0] == 192 && octets[1] == 168 {
        return true;
    }
    // 100.64.0.0/10 - Shared address space RFC 6598
    if octets[0] == 100 && (octets[1] >= 64 && octets[1] <= 127) {
        return true;
    }
    // 192.0.0.0/24 - IETF Protocol assignments
    if octets[0] == 192 && octets[1] == 0 && octets[2] == 0 {
        return true;
    }
    // 192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24 - Documentation (TEST-NET)
    if (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
        || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
        || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
    {
        return true;
    }
    // 198.18.0.0/15 - Benchmarking RFC 2544
    if octets[0] == 198 && (octets[1] == 18 || octets[1] == 19) {
        return true;
    }
    // 224.0.0.0/4 - Multicast
    if octets[0] >= 224 && octets[0] <= 239 {
        return true;
    }
    // 240.0.0.0/4 - Reserved & 255.255.255.255 Broadcast
    if octets[0] >= 240 {
        return true;
    }

    false
}

/// Checks if an IPv6 address belongs to a private, loopback, link-local, or reserved range.
pub fn is_private_or_reserved_ipv6(ip: Ipv6Addr) -> bool {
    let segments = ip.segments();

    // ::1 Loopback
    if ip == Ipv6Addr::LOCALHOST {
        return true;
    }
    // :: Unspecified
    if ip == Ipv6Addr::UNSPECIFIED {
        return true;
    }
    // fc00::/7 - Unique Local Address (ULA)
    if (segments[0] & 0xfe00) == 0xfc00 {
        return true;
    }
    // fe80::/10 - Link-Local Unicast
    if (segments[0] & 0xffc0) == 0xfe80 {
        return true;
    }
    // ff00::/8 - Multicast
    if (segments[0] & 0xff00) == 0xff00 {
        return true;
    }
    // IPv4-mapped IPv6 (::ffff:x.x.x.x)
    if let Some(mapped_v4) = ip.to_ipv4() {
        return is_private_or_reserved_ipv4(mapped_v4);
    }
    // 64:ff9b::/96 - Well-Known NAT64 Prefix (RFC 6052)
    if segments[0] == 0x0064
        && segments[1] == 0xff9b
        && segments[2] == 0
        && segments[3] == 0
        && segments[4] == 0
        && segments[5] == 0
    {
        let v4 = Ipv4Addr::new(
            (segments[6] >> 8) as u8,
            (segments[6] & 0xff) as u8,
            (segments[7] >> 8) as u8,
            (segments[7] & 0xff) as u8,
        );
        return is_private_or_reserved_ipv4(v4);
    }
    // 2002::/16 - 6to4 (RFC 3056)
    if segments[0] == 0x2002 {
        let v4 = Ipv4Addr::new(
            (segments[1] >> 8) as u8,
            (segments[1] & 0xff) as u8,
            (segments[2] >> 8) as u8,
            (segments[2] & 0xff) as u8,
        );
        return is_private_or_reserved_ipv4(v4);
    }
    // 2001:db8::/32 - Documentation (RFC 3849)
    if segments[0] == 0x2001 && segments[1] == 0x0db8 {
        return true;
    }
    // 2001:2::/48 - Benchmarking (RFC 5180)
    if segments[0] == 0x2001 && segments[1] == 0x0002 {
        return true;
    }
    // 100::/64 - Discard-Only (RFC 6666)
    if segments[0] == 0x0100 && segments[1] == 0 && segments[2] == 0 && segments[3] == 0 {
        return true;
    }

    false
}

pub fn is_private_or_reserved_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_private_or_reserved_ipv4(v4),
        IpAddr::V6(v6) => is_private_or_reserved_ipv6(v6),
    }
}

/// Validates target URL against SSRF, bad schemes, and forbidden hostnames.
/// Resolves DNS and returns the parsed URL along with verified public SocketAddrs.
pub fn validate_url_ssrf_and_resolve(raw_url: &str) -> Result<(Url, Vec<std::net::SocketAddr>), SecurityError> {
    let parsed = Url::parse(raw_url)?;

    // Only HTTP and HTTPS schemes are allowed
    match parsed.scheme() {
        "http" | "https" => {}
        other => return Err(SecurityError::InvalidScheme(other.to_string())),
    }

    // Port restriction: Only standard HTTP/HTTPS/Web ports are allowed
    let port = parsed.port_or_known_default().unwrap_or(80);
    if port != 80 && port != 443 && port != 8080 && port != 8443 && port != 3000 {
        return Err(SecurityError::InvalidPort(port));
    }

    let host_str = match parsed.host_str() {
        Some(h) if !h.trim().is_empty() => h.to_lowercase(),
        _ => return Err(SecurityError::InvalidHost("Bos ana bilgisayar".to_string())),
    };

    // Filter prohibited domains and hostnames
    if host_str == "localhost"
        || host_str.ends_with(".localhost")
        || host_str.ends_with(".local")
        || host_str.ends_with(".internal")
        || host_str.ends_with(".lan")
        || host_str.ends_with(".test")
        || host_str.ends_with(".example")
        || host_str.ends_with(".invalid")
    {
        return Err(SecurityError::SsrfBlocked(host_str));
    }

    let mut resolved_addrs = Vec::new();

    // If host is a direct IP
    if let Ok(ip) = host_str.parse::<IpAddr>() {
        if is_private_or_reserved_ip(ip) {
            return Err(SecurityError::SsrfBlocked(ip.to_string()));
        }
        resolved_addrs.push(std::net::SocketAddr::new(ip, port));
    } else {
        // Resolve host to DNS and verify none of the resolved IPs are internal
        let socket_str = format!("{}:{}", host_str, port);
        match socket_str.to_socket_addrs() {
            Ok(iter) => {
                let addrs: Vec<_> = iter.collect();
                if addrs.is_empty() {
                    return Err(SecurityError::HostResolutionFailed(format!(
                        "Host {} icin cozumlenmis IP adresi bulunamadi",
                        host_str
                    )));
                }
                for addr in addrs {
                    if is_private_or_reserved_ip(addr.ip()) {
                        return Err(SecurityError::SsrfBlocked(format!(
                            "Host {} ozel IP adresine cozuldu: {}",
                            host_str,
                            addr.ip()
                        )));
                    }
                    resolved_addrs.push(addr);
                }
            }
            Err(e) => {
                return Err(SecurityError::HostResolutionFailed(format!(
                    "Host {} cozumlenemedi: {}",
                    host_str, e
                )));
            }
        }
    }

    Ok((parsed, resolved_addrs))
}

/// Validates target URL against SSRF, bad schemes, and forbidden hostnames.
pub fn validate_url_ssrf(raw_url: &str) -> Result<Url, SecurityError> {
    let (url, _) = validate_url_ssrf_and_resolve(raw_url)?;
    Ok(url)
}

/// Fetches web content over HTTP/HTTPS with hard bounded buffer, SSRF guard, and DNS Rebinding defense.
pub async fn fetch_bounded_content(
    url_str: &str,
    max_bytes: usize,
    timeout_secs: u64,
) -> Result<String, SecurityError> {
    let (validated_url, resolved_addrs) = validate_url_ssrf_and_resolve(url_str)?;
    let host_str = validated_url.host_str().unwrap_or("").to_string();

    let mut client_builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 {
                attempt.error("Cok fazla yonlendirme (yonlendirme dongusu)")
            } else {
                let target = attempt.url();
                if let Err(e) = validate_url_ssrf(target.as_str()) {
                    attempt.error(format!("Yonlendirme SSRF engeline takildi: {}", e))
                } else {
                    attempt.follow()
                }
            }
        }))
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .user_agent("QuickNews/0.1 (Omarchy Linux; Text-First News Reader; +https://github.com/omarchy/quicknews)");

    // Pin resolved verified public IP address to prevent DNS Rebinding (TOCTOU attacks)
    if let Some(first_socket) = resolved_addrs.first() {
        client_builder = client_builder.resolve(&host_str, *first_socket);
    }

    let client = client_builder
        .build()
        .map_err(|e| SecurityError::Network(e.to_string()))?;

    let mut response = client
        .get(validated_url)
        .send()
        .await
        .map_err(|e| SecurityError::Network(e.to_string()))?;

    if !response.status().is_success() {
        return Err(SecurityError::Network(format!(
            "HTTP Hatasi: {}",
            response.status()
        )));
    }

    let mut body_bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| SecurityError::Network(e.to_string()))?
    {
        if body_bytes.len() + chunk.len() > max_bytes {
            return Err(SecurityError::PayloadTooLarge(body_bytes.len() + chunk.len()));
        }
        body_bytes.extend_from_slice(&chunk);
    }

    Ok(String::from_utf8_lossy(&body_bytes).to_string())
}

/// Atomically writes data to a file with strict 0600 file permissions and 0700 directory permissions.
/// Employs exclusive creation (O_CREAT | O_EXCL | O_NOFOLLOW) to prevent symlink following / TOCTOU attacks.
pub fn atomic_write_file(path: &Path, data: &[u8]) -> Result<(), SecurityError> {
    let parent = path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    // Check parent directory metadata
    if parent.exists() {
        let parent_meta = fs::symlink_metadata(parent)?;
        if parent_meta.file_type().is_symlink() {
            return Err(SecurityError::SymlinkForbidden(parent.to_path_buf()));
        }
    } else {
        fs::create_dir_all(parent)?;
        let mut perms = fs::metadata(parent)?.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(parent, perms)?;
    }

    // Check target file if exists
    if path.exists() {
        let meta = fs::symlink_metadata(path)?;
        if meta.file_type().is_symlink() {
            return Err(SecurityError::SymlinkForbidden(path.to_path_buf()));
        }
    }

    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file");

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let tmp_path = parent.join(format!(".tmp_{}_{}_{:x}", file_name, std::process::id(), nanos));

    // Exclusively create temporary file without following symlinks
    let write_res = (|| -> Result<(), SecurityError> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&tmp_path)?;

        file.write_all(data)?;
        file.sync_all()?;
        Ok(())
    })();

    if let Err(e) = write_res {
        let _ = fs::remove_file(&tmp_path);
        return Err(e);
    }

    if let Err(e) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(SecurityError::Io(e));
    }

    Ok(())
}

/// Safely reads file content with strict symlink check and bounded size.
/// Uses O_NOFOLLOW to avoid symlink TOCTOU race conditions.
pub fn safe_read_file(path: &Path, max_bytes: usize) -> Result<Vec<u8>, SecurityError> {
    if !path.exists() {
        return Err(SecurityError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Dosya bulunamadi",
        )));
    }

    // Open directly with O_NOFOLLOW to prevent symlink race attacks
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|e| {
            if e.raw_os_error() == Some(libc::ELOOP) {
                SecurityError::SymlinkForbidden(path.to_path_buf())
            } else {
                SecurityError::Io(e)
            }
        })?;

    let mut handle = file.take((max_bytes + 1) as u64);
    let mut buffer = Vec::new();
    handle.read_to_end(&mut buffer)?;

    if buffer.len() > max_bytes {
        return Err(SecurityError::PayloadTooLarge(buffer.len()));
    }

    Ok(buffer)
}

/// RAII Process Group Guard for external sub-processes (HANCORE standard).
pub struct ProcessGroupGuard {
    pgid: libc::pid_t,
}

impl ProcessGroupGuard {
    pub fn new(pid: u32) -> Self {
        let pgid = pid as libc::pid_t;
        if pgid > 1 {
            unsafe {
                libc::setpgid(pgid, pgid);
            }
        }
        Self { pgid }
    }

    pub fn pgid(&self) -> libc::pid_t {
        self.pgid
    }
}

impl Drop for ProcessGroupGuard {
    fn drop(&mut self) {
        if self.pgid > 1 {
            unsafe {
                libc::killpg(self.pgid, libc::SIGTERM);
            }
        }
    }
}
