use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStatus {
    pub running: bool,
    pub port: u16,
    pub endpoint: String,
}

impl Default for ProxyStatus {
    fn default() -> Self {
        Self {
            running: false,
            port: 8317,
            endpoint: "http://localhost:8317/v1".to_string(),
        }
    }
}

/// Response from rotation proxy provider API (e.g., proxyxoay.shop)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationProxyResponse {
    #[serde(default)]
    pub status: serde_json::Value,
    #[serde(default)]
    pub message: String,
    #[serde(default, rename = "proxyhttp")]
    pub proxy_http: String,
    #[serde(default, rename = "proxysocks5")]
    pub proxy_socks5: String,
    #[serde(default, rename = "Token expiration date")]
    pub token_expiration: String,
    /// Real external IP address (dynamic, changes on rotation)
    #[serde(default)]
    pub ip: String,
}

/// Cached proxy with expiration tracking
#[derive(Debug, Clone)]
pub struct CachedProxy {
    pub http_proxy: String,
    pub socks5_proxy: String,
    /// Real external IP address (dynamic, changes on rotation)
    pub real_ip: String,
    pub expires_at: Instant,
    pub ttl_seconds: u64,
    pub created_at: Instant,
}

impl CachedProxy {
    /// Check if the cached proxy has expired
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    /// Get remaining seconds until expiration
    pub fn expires_in_seconds(&self) -> u64 {
        let now = Instant::now();
        if now >= self.expires_at {
            0
        } else {
            self.expires_at.duration_since(now).as_secs()
        }
    }
}

/// Configuration for rotation proxy provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    /// Base API URL (e.g., https://proxyxoay.shop/api/get.php)
    pub api_url: String,
    /// API key for authentication
    pub api_key: String,
    /// Network type (e.g., "random", "viettel", "vinaphone")
    pub nhamang: String,
    /// Province/city filter (e.g., "0" for all)
    pub tinhthanh: String,
}

/// Status information for rotation proxy (returned to frontend)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RotationStatus {
    pub active: bool,
    pub current_proxy: String,
    /// Real external IP address (the dynamic IP that changes on rotation)
    pub real_ip: String,
    pub expires_in_seconds: u64,
    pub ttl_seconds: u64,
}

impl Default for RotationStatus {
    fn default() -> Self {
        Self {
            active: false,
            current_proxy: String::new(),
            real_ip: String::new(),
            expires_in_seconds: 0,
            ttl_seconds: 0,
        }
    }
}
