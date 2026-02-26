use async_trait::async_trait;
use crate::proxy::{ProxyProvider, ProxyProviderBase};
use crate::types::proxy::{CachedProxy, RotationConfig, RotationProxyResponse, RotationStatus};
use regex::Regex;
use std::error::Error;
use std::time::{Duration, Instant};
use log::{debug, error, info, warn};

/// Provider cho proxy.vn / proxyxoay.shop
#[derive(Debug)]
pub struct ProxyVNProvider {
    base: ProxyProviderBase,
    config: RotationConfig,
}

impl ProxyVNProvider {
    /// Parse rotation:// URL
    fn parse_url(url: &str) -> Result<RotationConfig, String> {
        if !url.starts_with("rotation://") {
            return Err("Invalid URL scheme".into());
        }

        let parsed = url::Url::parse(url)
            .map_err(|e| format!("Failed to parse URL: {}", e))?;

        let host = parsed.host_str()
            .ok_or("Missing host")?;

        let query_pairs: std::collections::HashMap<_, _> =
            parsed.query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

        let api_key = query_pairs.get("key")
            .ok_or("Missing 'key' parameter")?
            .clone();

        let nhamang = query_pairs.get("nhamang")
            .cloned()
            .unwrap_or_else(|| "random".to_string());

        let tinhthanh = query_pairs.get("tinhthanh")
            .cloned()
            .unwrap_or_else(|| "0".to_string());

        // Build API URL theo format proxy.vn
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

    /// Parse TTL từ message tiếng Việt
    fn parse_ttl(message: &str) -> u64 {
        let re = Regex::new(r"(?i)(\d+)\s*(?:s|giây|giay|seconds?|sec)").unwrap();
        if let Some(captures) = re.captures(message) {
            if let Some(seconds_str) = captures.get(1) {
                if let Ok(seconds) = seconds_str.as_str().parse::<u64>() {
                    return seconds;
                }
            }
        }
        1800 // Default 30 minutes
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
                normalized == "101" || normalized.contains("wait")
            }
            serde_json::Value::Number(n) => {
                n.as_i64() == Some(101) || n.as_u64() == Some(101)
            }
            _ => false,
        }
    }

    /// Parse waiting error delay from error string
    fn parse_waiting_error_delay(error: &str) -> Option<u64> {
        let re = Regex::new(r"^\[WAITING_STATUS\](\d+)\|").ok()?;
        let captures = re.captures(error)?;
        captures.get(1)?.as_str().parse::<u64>().ok()
    }

    /// Single fetch attempt (internal)
    async fn fetch_proxy_once(&self) -> Result<CachedProxy, String> {
        info!("[ProxyVN] Sending API request to: {}", self.config.api_url);

        let response = self
            .base
            .client
            .get(&self.config.api_url)
            .send()
            .await
            .map_err(|e| {
                // Log chi tiết error để debug - kiểm tra xem là DNS, Connection, hay TLS error
                let error_msg = format!("{}", e);
                let error_source = e.source().map(|s| format!("{}", s)).unwrap_or_default();
                error!(
                    "[ProxyVN] HTTP request failed: {} | Source: {} | Is connect: {} | Is timeout: {}",
                    error_msg,
                    error_source,
                    e.is_connect(),
                    e.is_timeout()
                );
                format!("HTTP request failed: {} (source: {})", error_msg, error_source)
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            error!("[ProxyVN] API returned error status: {}, body: {}", status, body);
            return Err(format!("API returned error status: {}", status));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| {
                error!("[ProxyVN] Failed to read API response body: {}", e);
                format!("Failed to read API response body: {}", e)
            })?;

        debug!("[ProxyVN] Raw API response: {}", response_text);

        let proxy_response: RotationProxyResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                error!("[ProxyVN] Failed to parse API response: {} | body: {}", e, response_text);
                format!("Failed to parse API response: {} | body: {}", e, response_text)
            })?;

        debug!("[ProxyVN] Parsed response - status: {:?}, message: {}, http_proxy: {}, socks5_proxy: {}",
               proxy_response.status, proxy_response.message,
               proxy_response.proxy_http, proxy_response.proxy_socks5);

        if !Self::is_success_status(&proxy_response.status) {
            if Self::is_waiting_status(&proxy_response.status) {
                let wait_seconds = Self::parse_ttl(&proxy_response.message);
                warn!(
                    "[ProxyVN] Provider asks to wait {}s before rotating: {}",
                    wait_seconds, proxy_response.message
                );
                return Err(format!(
                    "[WAITING_STATUS]{}|{}",
                    wait_seconds, proxy_response.message
                ));
            }

            error!(
                "[ProxyVN] API returned non-success status: {:?} | message: {}",
                proxy_response.status, proxy_response.message
            );
            return Err(format!(
                "API returned non-success status: {:?} | message: {}",
                proxy_response.status, proxy_response.message
            ));
        }

        // Parse TTL from message field (format: "Proxy mới sẽ thay đổi sau 462s")
        let ttl_seconds = Self::parse_ttl(&proxy_response.message);

        // Apply safety margin (subtract 30s to pre-emptively rotate)
        let effective_ttl = if ttl_seconds > 30 {
            ttl_seconds - 30
        } else {
            ttl_seconds
        };

        let now = Instant::now();
        let mut cached = CachedProxy {
            http_proxy: proxy_response.proxy_http.clone(),
            socks5_proxy: proxy_response.proxy_socks5.clone(),
            real_ip: proxy_response.ip.clone(),
            expires_at: now + Duration::from_secs(effective_ttl),
            ttl_seconds: effective_ttl,
            created_at: now,
        };

        // Nếu provider không trả về IP, thử lấy qua external API
        if cached.real_ip.is_empty() {
            debug!("[ProxyVN] Provider didn't return IP, attempting to fetch via external API");
            if let Err(e) = self.base.fetch_real_ip_if_missing(&mut cached).await {
                warn!("[ProxyVN] Failed to fetch real IP: {}", e);
            }
        }

        info!(
            "[ProxyVN] Fetched new proxy - HTTP: {}, SOCKS5: {}, Real IP: {}, TTL: {}s (effective: {}s)",
            cached.http_proxy, cached.socks5_proxy, cached.real_ip, ttl_seconds, effective_ttl
        );

        Ok(cached)
    }
}

#[async_trait]
impl ProxyProvider for ProxyVNProvider {
    fn provider_id(&self) -> &'static str {
        "proxy_vn"
    }

    fn provider_name(&self) -> String {
        "Proxy.vn / ProxyXoay".to_string()
    }

    fn can_handle(&self, url: &str) -> bool {
        url.starts_with("rotation://")
            && (url.contains("proxyxoay.shop") || url.contains("proxy.vn"))
    }

    async fn fetch_proxy(&self) -> Result<CachedProxy, String> {
        info!("[ProxyVN] Starting proxy fetch with retry logic");
        let max_retries = 3;
        let mut last_error = String::new();

        for attempt in 0..max_retries {
            debug!("[ProxyVN] Fetch attempt {}/{}", attempt + 1, max_retries);
            match self.fetch_proxy_once().await {
                Ok(proxy) => {
                    info!("[ProxyVN] Fetch succeeded on attempt {}", attempt + 1);
                    return Ok(proxy);
                }
                Err(e) => {
                    last_error = e.clone();
                    warn!(
                        "[ProxyVN] Fetch attempt {} failed: {}",
                        attempt + 1, last_error
                    );

                    // Check if this is a cooldown error - return immediately, don't sleep
                    if let Some(wait_seconds) = Self::parse_waiting_error_delay(&last_error) {
                        warn!(
                            "[ProxyVN] Provider cooldown detected: {}s remaining. Returning immediately without blocking.",
                            wait_seconds
                        );
                        // Return error with [WAITING_STATUS] marker for initializer to parse
                        return Err(format!(
                            "[WAITING_STATUS]{}|Proxy rotation on cooldown. Please wait {} seconds before rotating again.",
                            wait_seconds, wait_seconds
                        ));
                    }

                    if attempt < max_retries - 1 {
                        // Use short exponential backoff for non-cooldown errors
                        let delay = Duration::from_secs(2u64.pow(attempt as u32).min(8));
                        info!(
                            "[ProxyVN] Retrying in {}s (attempt {}/{})",
                            delay.as_secs(), attempt + 1, max_retries
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        error!(
            "[ProxyVN] Failed to fetch proxy after {} attempts: {}",
            max_retries, last_error
        );
        Err(format!(
            "Failed to fetch proxy after {} attempts: {}",
            max_retries, last_error
        ))
    }

    async fn get_or_refresh(&self) -> Result<CachedProxy, String> {
        self.base.get_or_refresh_impl(|| self.fetch_proxy()).await
    }

    async fn force_rotate(&self) -> Result<CachedProxy, String> {
        // Don't clear cache before fetching - this prevents the infinite loop
        // when provider returns cooldown error (cache would be None, causing
        // immediate re-trigger of rotation on next TTL check)
        // Instead, fetch new proxy first, only clear cache on success
        let new_proxy = self.fetch_proxy().await?;
        
        // Only update cache after successful fetch
        let mut cache = self.base.cache.write().await;
        *cache = Some(new_proxy.clone());
        
        info!(
            "[ProxyVN] Force rotation complete, new proxy expires in {}s",
            new_proxy.ttl_seconds
        );
        
        Ok(new_proxy)
    }

    async fn get_cached(&self) -> Option<CachedProxy> {
        self.base.get_cached().await
    }

    async fn is_about_to_expire(&self, threshold_seconds: u64) -> bool {
        self.base.is_about_to_expire(threshold_seconds).await
    }

    async fn get_valid_proxy(&self, threshold_seconds: u64) -> Result<CachedProxy, String> {
        // First check with read lock - fast path for valid proxies
        {
            let cache = self.base.cache.read().await;
            if let Some(ref cached) = *cache {
                let expires_in = cached.expires_in_seconds();
                if expires_in > threshold_seconds {
                    debug!(
                        "[ProxyVN] Proxy valid, expires in {}s (threshold: {}s)",
                        expires_in, threshold_seconds
                    );
                    return Ok(cached.clone());
                }
                info!(
                    "[ProxyVN] Proxy expiring soon ({}s <= {}s), auto-rotating...",
                    expires_in, threshold_seconds
                );
            } else {
                info!("[ProxyVN] No cached proxy, fetching new one");
            }
        }

        // Proxy is expiring soon or doesn't exist - acquire write lock for rotation
        let mut cache = self.base.cache.write().await;

        // Double-check after acquiring write lock (another request might have rotated)
        if let Some(ref cached) = *cache {
            let expires_in = cached.expires_in_seconds();
            if expires_in > threshold_seconds {
                info!(
                    "[ProxyVN] Proxy rotated by another request, using new proxy (expires in {}s)",
                    expires_in
                );
                return Ok(cached.clone());
            }
        }

        // Perform auto-rotation
        info!(
            "[ProxyVN] Auto-rotating proxy (threshold: {}s)",
            threshold_seconds
        );
        let new_proxy = self.fetch_proxy().await?;
        *cache = Some(new_proxy.clone());

        info!(
            "[ProxyVN] Auto-rotation complete, new proxy expires in {}s",
            new_proxy.ttl_seconds
        );

        Ok(new_proxy)
    }

    async fn get_status(&self) -> RotationStatus {
        if let Some(cached) = self.get_cached().await {
            RotationStatus {
                active: true,
                provider_id: Some(self.provider_id().to_string()),
                provider_name: Some(self.provider_name()),
                current_proxy: Self::resolve_proxy_url(&cached).unwrap_or_default(),
                real_ip: cached.real_ip.clone(),
                expires_in_seconds: cached.expires_in_seconds(),
                ttl_seconds: cached.ttl_seconds,
            }
        } else {
            RotationStatus {
                active: true,
                provider_id: Some(self.provider_id().to_string()),
                provider_name: Some(self.provider_name()),
                ..RotationStatus::default()
            }
        }
    }
}

impl ProxyVNProvider {
    /// Create a new provider from a rotation:// URL
    /// Alias for from_url for backward compatibility
    pub fn new(url: &str) -> Result<Self, String> {
        Self::from_url(url)
    }

    /// Create a new provider from a rotation:// URL
    pub fn from_url(url: &str) -> Result<Self, String> {
        info!("[ProxyVN] Creating provider from URL: {}", url);

        let config = Self::parse_url(url)?;

        info!("[ProxyVN] Parsed config - API URL: {}, nhamang: {}, tinhthanh: {}",
              config.api_url, config.nhamang, config.tinhthanh);

        let base = ProxyProviderBase::new()?;

        Ok(Self { base, config })
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
            return Self::parse_proxy_string(&cached.socks5_proxy, "socks5");
        }

        if !cached.http_proxy.is_empty() && cached.http_proxy != ":" {
            return Self::parse_proxy_string(&cached.http_proxy, "http");
        }

        None
    }

    /// Get the API URL (for debugging)
    pub fn api_url(&self) -> &str {
        &self.config.api_url
    }
}
