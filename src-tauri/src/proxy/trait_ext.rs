use async_trait::async_trait;
use crate::types::proxy::{CachedProxy, RotationStatus};
use std::sync::Arc;

/// Trait định nghĩa interface chung cho tất cả proxy providers
/// Implement nguyên tắc Strategy Pattern
#[async_trait]
pub trait ProxyProvider: Send + Sync {
    /// Unique identifier cho provider (vd: "proxy_vn", "proxy_xoay")
    fn provider_id(&self) -> &'static str;

    /// Human-readable name cho UI
    fn provider_name(&self) -> String;

    /// Kiểm tra xem provider có thể xử lý URL này không
    fn can_handle(&self, url: &str) -> bool;

    /// Fetch proxy mới từ provider API
    async fn fetch_proxy(&self) -> Result<CachedProxy, String>;

    /// Get cached proxy hoặc fetch mới nếu expired
    async fn get_or_refresh(&self) -> Result<CachedProxy, String>;

    /// Force rotate proxy (xóa cache và fetch mới)
    async fn force_rotate(&self) -> Result<CachedProxy, String>;

    /// Get current cached proxy without fetching
    async fn get_cached(&self) -> Option<CachedProxy>;

    /// Check if proxy is about to expire
    async fn is_about_to_expire(&self, threshold_seconds: u64) -> bool;

    /// Get valid proxy with auto-rotation
    async fn get_valid_proxy(&self, threshold_seconds: u64) -> Result<CachedProxy, String>;

    /// Get rotation status for frontend
    async fn get_status(&self) -> RotationStatus;

    /// Parse proxy string từ format provider sang standard URL
    /// Provider format: `host:port:user:pass`
    /// Output: `http://user:pass@host:port` hoặc `socks5://user:pass@host:port`
    fn parse_proxy_string(proxy_string: &str, protocol: &str) -> Option<String> where Self: Sized {
        let parts: Vec<&str> = proxy_string.split(':').collect();

        // Some providers return host:port:: when auth is not required
        if parts.len() == 4 {
            let host = parts[0];
            let port = parts[1];
            let user = parts[2];
            let pass = parts[3];

            if host.is_empty() || port.is_empty() {
                log::warn!("[ProxyProvider] Empty host or port in proxy string: {}", proxy_string);
                return None;
            }

            if user.is_empty() && pass.is_empty() {
                let result = format!("{}://{}:{}", protocol, host, port);
                log::debug!("[ProxyProvider] Parsed {} (no auth): {}", protocol, result);
                return Some(result);
            }

            if user.is_empty() || pass.is_empty() {
                log::warn!("[ProxyProvider] Partial auth in proxy string: {}", proxy_string);
                return None;
            }

            let result = format!("{}://{}:{}@{}:{}", protocol, user, pass, host, port);
            log::debug!("[ProxyProvider] Parsed {} proxy: {}@{}:{}", protocol, user, host, port);
            return Some(result);
        }

        // Fallback support for host:port
        if parts.len() == 2 {
            let host = parts[0];
            let port = parts[1];

            if host.is_empty() || port.is_empty() {
                log::warn!("[ProxyProvider] Empty host or port in proxy string: {}", proxy_string);
                return None;
            }

            let result = format!("{}://{}:{}", protocol, host, port);
            log::debug!("[ProxyProvider] Parsed {} proxy: {}", protocol, result);
            return Some(result);
        }

        log::error!(
            "[ProxyProvider] Invalid proxy string format: expected host:port:user:pass or host:port::, got: {}",
            proxy_string
        );
        None
    }
}

/// Type alias cho dynamic dispatch
pub type DynProxyProvider = Arc<dyn ProxyProvider>;
