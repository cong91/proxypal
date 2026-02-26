use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use log::{error, info, warn};
use crate::types::proxy::CachedProxy;

/// Base struct chứa common caching logic
/// Các concrete provider sẽ embed hoặc wrap struct này
#[derive(Debug)]
pub struct ProxyProviderBase {
    pub cache: Arc<RwLock<Option<CachedProxy>>>,
    pub client: reqwest::Client,
}

impl ProxyProviderBase {
    pub fn new() -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            // FIX: Bỏ qua xác minh chứng chỉ SSL để khắc phục lỗi TLS với proxyxoay.shop
            // trên Windows nơi chứng chỉ có thể không được trust store công nhận
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        info!("[ProxyProviderBase] HTTP client created with SSL verification disabled");

        Ok(Self {
            cache: Arc::new(RwLock::new(None)),
            client,
        })
    }

    /// Common double-checked caching logic
    pub async fn get_or_refresh_impl<F, Fut>(
        &self,
        fetch_fn: F,
    ) -> Result<CachedProxy, String>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<CachedProxy, String>>,
    {
        let call_started = Instant::now();
        info!("[ProxyProviderBase] get_or_refresh_impl start");

        // First check with read lock
        {
            let cache = self.cache.read().await;
            if let Some(ref cached) = *cache {
                if !cached.is_expired() {
                    info!(
                        "[ProxyProviderBase] Cache hit, expires in {}s (total={}ms)",
                        cached.expires_in_seconds(),
                        call_started.elapsed().as_millis()
                    );
                    return Ok(cached.clone());
                }
            }
        }

        // Acquire write lock và double-check
        let write_lock_started = Instant::now();
        let mut cache = self.cache.write().await;
        let write_lock_wait_ms = write_lock_started.elapsed().as_millis();
        info!(
            "[ProxyProviderBase] Write lock acquired after {}ms",
            write_lock_wait_ms
        );

        if let Some(ref cached) = *cache {
            if !cached.is_expired() {
                info!(
                    "[ProxyProviderBase] Cache updated by another thread, using cached (write_wait={}ms total={}ms)",
                    write_lock_wait_ms,
                    call_started.elapsed().as_millis()
                );
                return Ok(cached.clone());
            }
        }

        // Fetch new proxy while holding write lock (diagnostic)
        let fetch_started = Instant::now();
        info!(
            "[ProxyProviderBase] Cache miss/expired, fetching new proxy (write_wait={}ms)",
            write_lock_wait_ms
        );

        let new_proxy = match fetch_fn().await {
            Ok(proxy) => proxy,
            Err(e) => {
                error!(
                    "[ProxyProviderBase] Fetch failed while holding write lock (fetch={}ms total={}ms): {}",
                    fetch_started.elapsed().as_millis(),
                    call_started.elapsed().as_millis(),
                    e
                );
                return Err(e);
            }
        };

        *cache = Some(new_proxy.clone());
        info!(
            "[ProxyProviderBase] Cache updated with new proxy, expires in {}s (fetch={}ms total={}ms)",
            new_proxy.ttl_seconds,
            fetch_started.elapsed().as_millis(),
            call_started.elapsed().as_millis()
        );
        Ok(new_proxy)
    }

    pub async fn get_cached(&self) -> Option<CachedProxy> {
        let cache = self.cache.read().await;
        cache.clone()
    }

    pub async fn is_about_to_expire(&self, threshold_seconds: u64) -> bool {
        let cache = self.cache.read().await;
        if let Some(ref cached) = *cache {
            cached.expires_in_seconds() <= threshold_seconds
        } else {
            true
        }
    }

    /// Clear cache (used for force rotate)
    ///
    /// NOTE: Use with caution - clearing cache before fetching can cause
    /// infinite rotation loops if the fetch fails (e.g., provider cooldown).
    /// Prefer using force_rotate() which handles this properly.
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }
    
    /// Get direct access to cache for atomic operations
    /// This allows providers to implement safe rotation without race conditions
    pub fn cache_arc(&self) -> &Arc<RwLock<Option<CachedProxy>>> {
        &self.cache
    }

    /// Fetch real IP through the proxy if it's missing from provider response
    /// This is a fallback for providers that don't return IP in their API response
    pub async fn fetch_real_ip_if_missing(&self, cached: &mut CachedProxy) -> Result<(), String> {
        if !cached.real_ip.is_empty() {
            return Ok(());
        }

        // Sử dụng proxy vừa nhận để check external IP
        let proxy_url = if !cached.http_proxy.is_empty() {
            &cached.http_proxy
        } else if !cached.socks5_proxy.is_empty() {
            &cached.socks5_proxy
        } else {
            return Ok(());
        };

        let proxy = reqwest::Proxy::all(proxy_url)
            .map_err(|e| format!("Invalid proxy URL: {}", e))?;

        let client = reqwest::Client::builder()
            .proxy(proxy)
            .timeout(std::time::Duration::from_secs(10))
            // Bỏ qua SSL verification cho IP check
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        match client.get("https://api.ipify.org").send().await {
            Ok(resp) => {
                if let Ok(ip) = resp.text().await {
                    let ip = ip.trim().to_string();
                    if !ip.is_empty() {
                        info!("[ProxyProviderBase] Resolved real IP: {}", ip);
                        cached.real_ip = ip;
                    }
                }
            }
            Err(e) => {
                warn!("[ProxyProviderBase] Failed to resolve real IP: {}", e);
                // Non-fatal: real_ip remains empty
            }
        }

        Ok(())
    }
}
