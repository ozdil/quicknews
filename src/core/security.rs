use std::fs::{self, File, OpenOptions};
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

    false
}

pub fn is_private_or_reserved_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_private_or_reserved_ipv4(v4),
        IpAddr::V6(v6) => is_private_or_reserved_ipv6(v6),
    }
}

/// Validates target URL against SSRF, bad schemes, and forbidden hostnames.
pub fn validate_url_ssrf(raw_url: &str) -> Result<Url, SecurityError> {
    let parsed = Url::parse(raw_url)?;

    // Only HTTP and HTTPS schemes are allowed
    match parsed.scheme() {
        "http" | "https" => {}
        other => return Err(SecurityError::InvalidScheme(other.to_string())),
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

    // If host is a direct IP
    if let Ok(ip) = host_str.parse::<IpAddr>() {
        if is_private_or_reserved_ip(ip) {
            return Err(SecurityError::SsrfBlocked(ip.to_string()));
        }
    } else {
        // Resolve host to DNS and verify none of the resolved IPs are internal
        let port = parsed.port_or_known_default().unwrap_or(80);
        let socket_str = format!("{}:{}", host_str, port);
        if let Ok(addrs) = socket_str.to_socket_addrs() {
            for addr in addrs {
                if is_private_or_reserved_ip(addr.ip()) {
                    return Err(SecurityError::SsrfBlocked(format!(
                        "Host {} ozel IP adresine cozuldu: {}",
                        host_str,
                        addr.ip()
                    )));
                }
            }
        }
    }

    Ok(parsed)
}

/// Fetches web content over HTTP/HTTPS with hard bounded buffer and SSRF guard.
pub async fn fetch_bounded_content(
    url_str: &str,
    max_bytes: usize,
    timeout_secs: u64,
) -> Result<String, SecurityError> {
    let validated_url = validate_url_ssrf(url_str)?;

    let client = reqwest::Client::builder()
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
        .user_agent("QuickNews/0.1 (Omarchy Linux; Text-First News Reader; +https://github.com/omarchy/quicknews)")
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

    String::from_utf8(body_bytes).map_err(|_| {
        SecurityError::Network("Icerik gecerli UTF-8 metin karakterleri icermiyor".to_string())
    })
}

/// Atomically writes data to a file with strict 0600 file permissions and 0700 directory permissions.
/// Rejects symlinks unconditionally.
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

    let tmp_path = parent.join(format!(".tmp_{}_{}", file_name, std::process::id()));

    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp_path)?;

        file.write_all(data)?;
        file.sync_all()?;
    }

    fs::rename(&tmp_path, path)?;
    Ok(())
}

/// Safely reads file content with strict symlink check and bounded size.
pub fn safe_read_file(path: &Path, max_bytes: usize) -> Result<Vec<u8>, SecurityError> {
    if !path.exists() {
        return Err(SecurityError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Dosya bulunamadi",
        )));
    }

    let meta = fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Err(SecurityError::SymlinkForbidden(path.to_path_buf()));
    }

    let file = File::open(path)?;
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
        unsafe {
            libc::setpgid(pgid, pgid);
        }
        Self { pgid }
    }

    pub fn pgid(&self) -> libc::pid_t {
        self.pgid
    }
}

impl Drop for ProcessGroupGuard {
    fn drop(&mut self) {
        if self.pgid > 0 {
            unsafe {
                libc::killpg(self.pgid, libc::SIGTERM);
            }
        }
    }
}
