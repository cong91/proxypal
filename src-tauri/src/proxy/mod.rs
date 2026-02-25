//! Rotation Proxy Provider
//!
//! This module implements a dynamic proxy rotation mechanism that fetches fresh
//! proxy IPs from an external provider API, caches them with TTL-based expiration,
//! and seamlessly injects resolved proxies into the running CLIProxyAPI sidecar.
//!
//! ## Usage
//!
//! ```
//! // Parse rotation:// URL
//! let provider = RotationProxyProvider::new("rotation://proxyxoay.shop?key=XXX&nhamang=random&tinhthanh=0")?;
//!
//! // Get or refresh cached proxy
//! let cached = provider.get_or_refresh().await?;
//!
//! // Resolve to standard proxy URL
//! let proxy_url = RotationProxyProvider::resolve_proxy_url(&cached);
//! ```

use crate::types::proxy::{CachedProxy, RotationConfig, RotationProxyResponse};
use regex::Regex;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Provider for rotation proxy with automatic fetching and caching
#[derive(Debug)]
pub struct RotationProxyProvider {
    /// Rotation configuration parsed from rotation:// URL
    config: RotationConfig,
    /// Thread-safe cache for the current proxy
    cache: Arc<RwLock<Option<CachedProxy>>>,
    /// HTTP client for API requests
    client: reqwest::Client,
}

impl RotationProxyProvider {
    /// Create a new rotation proxy provider from a rotation:// URL
    ///
    /// # URL Format
    /// `rotation://host?key=API_KEY&nhamang=NETWORK&tinhthanh=PROVINCE`
    ///
    /// # Example
    /// `rotation://proxyxoay.shop?key=abc123&nhamang=random&tinhthanh=0`
    pub fn new(rotation_url: &str) -> Result<Self, String> {
        let config = parse_rotation_url(rotation_url)?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            config,
            cache: Arc::new(RwLock::new(None)),
            client,
        })
    }

    /// Fetch a new proxy from the provider API with retry logic
    ///
    /// Implements retry with exponential backoff.
    /// If provider returns waiting/cooldown status, retries follow provider TTL.
    pub async fn fetch_proxy(&self) -> Result<CachedProxy, String> {
        let max_retries = 5;
        let mut last_error = String::new();

        for attempt in 0..max_retries {
            match self.fetch_proxy_once().await {
                Ok(proxy) => return Ok(proxy),
                Err(e) => {
                    last_error = e;
                    if attempt < max_retries - 1 {
                        let delay = if let Some(wait_seconds) = parse_waiting_error_delay(&last_error)
                        {
                            Duration::from_secs(wait_seconds.saturating_add(1).min(120))
                        } else {
                            Duration::from_secs(2u64.pow(attempt as u32 + 1))
                        };

                        println!(
                            "[RotationProxy] Fetch attempt {} failed, retrying in {}s: {}",
                            attempt + 1,
                            delay.as_secs(),
                            last_error
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(format!(
            "Failed to fetch proxy after {} attempts: {}",
            max_retries, last_error
        ))
    }

    /// Single fetch attempt (internal)
    async fn fetch_proxy_once(&self) -> Result<CachedProxy, String> {
        let response = self
            .client
            .get(&self.config.api_url)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API returned error status: {}", response.status()));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read API response body: {}", e))?;

        println!("[RotationProxy][DEBUG] Raw API response: {}", response_text);

        let proxy_response: RotationProxyResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse API response: {} | body: {}", e, response_text))?;

        if !is_success_status(&proxy_response.status) {
            if is_waiting_status(&proxy_response.status) {
                let wait_seconds = parse_ttl_from_message(&proxy_response.message);
                println!(
                    "[RotationProxy] Provider asks to wait {}s before rotating: {}",
                    wait_seconds, proxy_response.message
                );
                return Err(format!(
                    "[WAITING_STATUS]{}|{}",
                    wait_seconds, proxy_response.message
                ));
            }

            return Err(format!(
                "API returned non-success status: {} | message: {}",
                status_to_string(&proxy_response.status),
                proxy_response.message
            ));
        }

        // Parse TTL from message field (format: "Proxy mới sẽ thay đổi sau 462s")
        let ttl_seconds = parse_ttl_from_message(&proxy_response.message);

        // Apply safety margin (subtract 30s to pre-emptively rotate)
        let effective_ttl = if ttl_seconds > 30 {
            ttl_seconds - 30
        } else {
            ttl_seconds
        };

        let now = Instant::now();
        let cached = CachedProxy {
            http_proxy: proxy_response.proxy_http.clone(),
            socks5_proxy: proxy_response.proxy_socks5.clone(),
            expires_at: now + Duration::from_secs(effective_ttl),
            ttl_seconds: effective_ttl,
            created_at: now,
        };

        println!(
            "[RotationProxy] Fetched new proxy, TTL: {}s (effective: {}s)",
            ttl_seconds, effective_ttl
        );

        Ok(cached)
    }

    /// Get cached proxy or fetch a new one if expired
    ///
    /// Uses double-check locking pattern to prevent thundering herd
    pub async fn get_or_refresh(&self) -> Result<CachedProxy, String> {
        // First check with read lock
        {
            let cache = self.cache.read().await;
            if let Some(ref cached) = *cache {
                if !cached.is_expired() {
                    return Ok(cached.clone());
                }
            }
        }

        // Cache miss or expired - acquire write lock
        let mut cache = self.cache.write().await;

        // Double-check after acquiring write lock
        if let Some(ref cached) = *cache {
            if !cached.is_expired() {
                return Ok(cached.clone());
            }
        }

        // Fetch new proxy
        let new_proxy = self.fetch_proxy().await?;
        *cache = Some(new_proxy.clone());

        println!(
            "[RotationProxy] Cache updated with new proxy, expires in {}s",
            new_proxy.ttl_seconds
        );

        Ok(new_proxy)
    }

    /// Force rotate the proxy (invalidate cache and fetch new)
    ///
    /// Used when the current proxy is detected as dead or when user
    /// manually requests rotation.
    pub async fn force_rotate(&self) -> Result<CachedProxy, String> {
        println!("[RotationProxy] Force rotating proxy...");

        // Clear cache
        {
            let mut cache = self.cache.write().await;
            *cache = None;
        }

        // Fetch new proxy
        self.get_or_refresh().await
    }

    /// Get current cached proxy without fetching
    pub async fn get_cached(&self) -> Option<CachedProxy> {
        let cache = self.cache.read().await;
        cache.clone()
    }

    /// Check if the cached proxy is about to expire (within threshold)
    pub async fn is_about_to_expire(&self, threshold_seconds: u64) -> bool {
        let cache = self.cache.read().await;
        if let Some(ref cached) = *cache {
            let expires_in = cached.expires_in_seconds();
            expires_in <= threshold_seconds
        } else {
            true // No cache means we need to fetch
        }
    }

    /// Convert cached proxy to standard proxy URL
    ///
    /// Supports both HTTP and SOCKS5 proxy strings.
    /// Format from provider: `host:port:user:pass`
    /// Converts to: `http://user:pass@host:port` or `socks5://user:pass@host:port`
    ///
    /// Prefers SOCKS5 if available, falls back to HTTP.
    pub fn resolve_proxy_url(cached: &CachedProxy) -> Option<String> {
        // Prefer SOCKS5 if available
        if !cached.socks5_proxy.is_empty() && cached.socks5_proxy != ":" {
            return parse_proxy_string(&cached.socks5_proxy, "socks5");
        }
        
        if !cached.http_proxy.is_empty() && cached.http_proxy != ":" {
            return parse_proxy_string(&cached.http_proxy, "http");
        }
        
        None
    }

    /// Get the API URL (for debugging)
    pub fn api_url(&self) -> &str {
        &self.config.api_url
    }
}

/// Parse rotation:// URL into RotationConfig
///
/// # URL Format
/// `rotation://host?key=API_KEY&nhamang=NETWORK&tinhthanh=PROVINCE`
fn parse_rotation_url(url: &str) -> Result<RotationConfig, String> {
    if !url.starts_with("rotation://") {
        return Err(format!(
            "Invalid rotation URL scheme. Expected rotation://, got: {}",
            url
        ));
    }

    // Parse as URL
    let parsed = url::Url::parse(url).map_err(|e| format!("Failed to parse URL: {}", e))?;

    let host = parsed
        .host_str()
        .ok_or_else(|| "Missing host in rotation URL".to_string())?;

    let query_pairs: std::collections::HashMap<_, _> =
        parsed.query_pairs().map(|(k, v)| (k.to_string(), v.to_string())).collect();

    let api_key = query_pairs
        .get("key")
        .ok_or_else(|| "Missing 'key' parameter in rotation URL".to_string())?
        .clone();

    let nhamang = query_pairs
        .get("nhamang")
        .cloned()
        .unwrap_or_else(|| "random".to_string());

    let tinhthanh = query_pairs
        .get("tinhthanh")
        .cloned()
        .unwrap_or_else(|| "0".to_string());

    // Build full API URL
    let api_url = format!(
        "https://{}/api/get.php?key={}&nhamang={}&tinhthanh={}",
        host, api_key, nhamang, tinhthanh
    );

    Ok(RotationConfig {
        api_url,
        api_key,
        nhamang,
        tinhthanh,
    })
}

/// Parse TTL from message string
///
/// Extracts the number of seconds from strings like:
/// - "Proxy mới sẽ thay đổi sau 462s"
/// - "Thay đổi sau 1800 giây"
///
/// Defaults to 1800s (30 minutes) if parsing fails.
fn parse_ttl_from_message(message: &str) -> u64 {
    // Try to match patterns like "462s", "462 giây", "462 giay"
    let re = Regex::new(r"(?i)(\d+)\s*(?:s|giây|giay|seconds?|sec)").unwrap();

    if let Some(captures) = re.captures(message) {
        if let Some(seconds_str) = captures.get(1) {
            if let Ok(seconds) = seconds_str.as_str().parse::<u64>() {
                return seconds;
            }
        }
    }

    // Default to 30 minutes if parsing fails
    println!(
        "[RotationProxy] Could not parse TTL from message: '{}', defaulting to 1800s",
        message
    );
    1800
}

fn status_to_string(status: &serde_json::Value) -> String {
    match status {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

fn is_success_status(status: &serde_json::Value) -> bool {
    match status {
        serde_json::Value::String(s) => {
            let normalized = s.trim().to_ascii_lowercase();
            normalized == "success" || normalized == "ok" || normalized == "100"
        }
        serde_json::Value::Number(n) => {
            n.as_i64() == Some(100) || n.as_u64() == Some(100)
        }
        _ => false,
    }
}

fn is_waiting_status(status: &serde_json::Value) -> bool {
    match status {
        serde_json::Value::String(s) => {
            let normalized = s.trim().to_ascii_lowercase();
            normalized == "101" || normalized.contains("wait") || normalized.contains("cooldown")
        }
        serde_json::Value::Number(n) => {
            n.as_i64() == Some(101) || n.as_u64() == Some(101)
        }
        _ => false,
    }
}

fn parse_waiting_error_delay(error: &str) -> Option<u64> {
    let re = Regex::new(r"^\[WAITING_STATUS\](\d+)\|").ok()?;
    let captures = re.captures(error)?;
    captures.get(1)?.as_str().parse::<u64>().ok()
}

/// Parse proxy string from provider format to standard URL
///
/// Provider format: `host:port:user:pass`
/// Output: `http://user:pass@host:port` or `socks5://user:pass@host:port`
///
/// # Arguments
/// * `proxy_string` - The proxy string in format `host:port:user:pass`
/// * `protocol` - The protocol to use ("http" or "socks5")
fn parse_proxy_string(proxy_string: &str, protocol: &str) -> Option<String> {
    let parts: Vec<&str> = proxy_string.split(':').collect();

    // Some providers return host:port:: when auth is not required
    if parts.len() == 4 {
        let host = parts[0];
        let port = parts[1];
        let user = parts[2];
        let pass = parts[3];

        if host.is_empty() || port.is_empty() {
            return None;
        }

        if user.is_empty() && pass.is_empty() {
            return Some(format!("{}://{}:{}", protocol, host, port));
        }

        if user.is_empty() || pass.is_empty() {
            return None;
        }

        return Some(format!("{}://{}:{}@{}:{}", protocol, user, pass, host, port));
    }

    // Fallback support for host:port
    if parts.len() == 2 {
        let host = parts[0];
        let port = parts[1];

        if host.is_empty() || port.is_empty() {
            return None;
        }

        return Some(format!("{}://{}:{}", protocol, host, port));
    }

    println!(
        "[RotationProxy] Invalid proxy string format: expected host:port:user:pass or host:port::, got: {}",
        proxy_string
    );
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rotation_url_valid() {
        let url = "rotation://proxyxoay.shop?key=abc123&nhamang=random&tinhthanh=0";
        let config = parse_rotation_url(url).unwrap();

        assert_eq!(config.api_url, "https://proxyxoay.shop/api/get.php?key=abc123&nhamang=random&tinhthanh=0");
        assert_eq!(config.api_key, "abc123");
        assert_eq!(config.nhamang, "random");
        assert_eq!(config.tinhthanh, "0");
    }

    #[test]
    fn test_parse_rotation_url_missing_optional_params() {
        let url = "rotation://proxyxoay.shop?key=abc123";
        let config = parse_rotation_url(url).unwrap();

        assert_eq!(config.api_key, "abc123");
        assert_eq!(config.nhamang, "random"); // default
        assert_eq!(config.tinhthanh, "0"); // default
    }

    #[test]
    fn test_parse_rotation_url_invalid_scheme() {
        let url = "http://proxyxoay.shop?key=abc123";
        let result = parse_rotation_url(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_rotation_url_missing_key() {
        let url = "rotation://proxyxoay.shop?nhamang=random";
        let result = parse_rotation_url(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ttl_from_message() {
        assert_eq!(parse_ttl_from_message("Proxy mới sẽ thay đổi sau 462s"), 462);
        assert_eq!(parse_ttl_from_message("Thay đổi sau 1800 giây"), 1800);
        assert_eq!(parse_ttl_from_message("Timeout: 300 seconds"), 300);
        assert_eq!(parse_ttl_from_message("Invalid message"), 1800); // default
    }

    #[test]
    fn test_parse_proxy_string_http_valid() {
        let proxy = "192.168.1.1:8080:user:pass";
        let result = parse_proxy_string(proxy, "http");
        assert_eq!(result, Some("http://user:pass@192.168.1.1:8080".to_string()));
    }

    #[test]
    fn test_parse_proxy_string_socks5_valid() {
        let proxy = "192.168.1.2:1080:user:pass";
        let result = parse_proxy_string(proxy, "socks5");
        assert_eq!(result, Some("socks5://user:pass@192.168.1.2:1080".to_string()));
    }

    #[test]
    fn test_parse_proxy_string_no_auth_valid() {
        let proxy = "192.168.1.1:8080::";
        let result = parse_proxy_string(proxy, "http");
        assert_eq!(result, Some("http://192.168.1.1:8080".to_string()));
    }

    #[test]
    fn test_parse_proxy_string_invalid() {
        let proxy = "192.168.1.1";
        let result = parse_proxy_string(proxy, "http");
        assert_eq!(result, None);
    }

    #[test]
    fn test_cached_proxy_expiration() {
        let cached = CachedProxy {
            http_proxy: "test".to_string(),
            socks5_proxy: "test".to_string(),
            expires_at: Instant::now() + Duration::from_secs(100),
            ttl_seconds: 100,
            created_at: Instant::now(),
        };

        assert!(!cached.is_expired());
        assert!(cached.expires_in_seconds() > 0);
        assert!(cached.expires_in_seconds() <= 100);
    }

    #[test]
    fn test_resolve_proxy_url() {
        let cached = CachedProxy {
            http_proxy: "192.168.1.1:8080:user:pass".to_string(),
            socks5_proxy: "192.168.1.2:1080:user:pass".to_string(),
            expires_at: Instant::now() + Duration::from_secs(100),
            ttl_seconds: 100,
            created_at: Instant::now(),
        };

        // Should prefer SOCKS5
        let result = RotationProxyProvider::resolve_proxy_url(&cached);
        assert_eq!(result, Some("socks5://user:pass@192.168.1.2:1080".to_string()));
    }

    #[test]
    fn test_resolve_proxy_url_fallback() {
        let cached = CachedProxy {
            http_proxy: "192.168.1.1:8080:user:pass".to_string(),
            socks5_proxy: ":".to_string(), // empty
            expires_at: Instant::now() + Duration::from_secs(100),
            ttl_seconds: 100,
            created_at: Instant::now(),
        };

        // Should fallback to HTTP
        let result = RotationProxyProvider::resolve_proxy_url(&cached);
        assert_eq!(result, Some("http://user:pass@192.168.1.1:8080".to_string()));
    }

    #[tokio::test]
    async fn debug_live_rotation_provider_e2e() {
        let rotation_url = "rotation://proxyxoay.shop?key=HnMBZgKRukdXQBHWssjmGP&&nhamang=random&&tinhthanh=0";
        let provider = RotationProxyProvider::new(rotation_url)
            .expect("rotation URL should be parsed successfully");

        println!("[RotationProxy][E2E] API URL: {}", provider.api_url());

        let cached = provider
            .fetch_proxy()
            .await
            .expect("live fetch_proxy should succeed");

        let resolved_proxy = RotationProxyProvider::resolve_proxy_url(&cached);

        println!("[RotationProxy][E2E] HTTP Proxy: {}", cached.http_proxy);
        println!("[RotationProxy][E2E] SOCKS5 Proxy: {}", cached.socks5_proxy);
        println!("[RotationProxy][E2E] Parsed TTL: {}", cached.ttl_seconds);
        println!("[RotationProxy][E2E] Proxy String: {:?}", resolved_proxy);

        assert!(
            !cached.http_proxy.is_empty() || !cached.socks5_proxy.is_empty(),
            "at least one proxy field should be available"
        );
    }
}
