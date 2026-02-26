use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use log::{debug, error, info, warn};
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
        // First check with read lock
        {
            let cache = self.cache.read().await;
            if let Some(ref cached) = *cache {
                if !cached.is_expired() {
                    debug!("[ProxyProviderBase] Cache hit, expires in {}s", cached.expires_in_seconds());
                    return Ok(cached.clone());
                }
            }
        }

        // Acquire write lock và double-check
        let mut cache = self.cache.write().await;
        if let Some(ref cached) = *cache {
            if !cached.is_expired() {
                debug!("[ProxyProviderBase] Cache updated by another thread, using cached");
                return Ok(cached.clone());
            }
        }

        // Fetch new proxy
        let new_proxy = fetch_fn().await?;
        *cache = Some(new_proxy.clone());
        info!("[ProxyProviderBase] Cache updated with new proxy, expires in {}s", new_proxy.ttl_seconds);
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
}
