use crate::proxy::{DynProxyProvider};
use crate::proxy::providers::proxy_vn::ProxyVNProvider;
use crate::types::proxy::ProviderInfo;
use std::collections::HashMap;
use lazy_static::lazy_static;

struct ProviderEntry {
    id: &'static str,
    name: &'static str,
    factory: fn(&str) -> Result<DynProxyProvider, String>,
}

lazy_static! {
    static ref PROVIDER_REGISTRY: HashMap<&'static str, ProviderEntry> = {
        let mut m = HashMap::new();
        m.insert("proxy_vn", ProviderEntry {
            id: "proxy_vn",
            name: "Proxy.vn / ProxyXoay",
            factory: |url| ProxyVNProvider::from_url(url).map(|p| std::sync::Arc::new(p) as DynProxyProvider),
        });
        // Thêm providers khác ở đây
        // m.insert("proxy_xoay", ProviderEntry { ... });
        m
    };
}

/// Factory để tạo proxy provider từ URL
pub struct ProxyProviderFactory;

impl ProxyProviderFactory {
    /// Tạo provider từ rotation URL
    /// Tự động detect provider type dựa trên URL
    pub fn create(url: &str) -> Result<DynProxyProvider, String> {
        // Try auto-detect based on URL patterns
        if url.contains("proxyxoay.shop") || url.contains("proxy.vn") {
            return (PROVIDER_REGISTRY.get("proxy_vn").unwrap().factory)(url);
        }

        // Fallback: try all registered providers
        for (_, entry) in PROVIDER_REGISTRY.iter() {
            if let Ok(provider) = (entry.factory)(url) {
                return Ok(provider);
            }
        }

        Err(format!("No provider found for URL: {}", url))
    }

    /// Tạo provider cụ thể bằng ID
    pub fn create_by_id(provider_id: &str, url: &str) -> Result<DynProxyProvider, String> {
        PROVIDER_REGISTRY
            .get(provider_id)
            .ok_or_else(|| format!("Unknown provider: {}", provider_id))
            .and_then(|entry| (entry.factory)(url))
    }

    /// Lấy danh sách available providers (cho UI)
    pub fn available_providers() -> Vec<ProviderInfo> {
        PROVIDER_REGISTRY
            .values()
            .map(|entry| ProviderInfo {
                id: entry.id.to_string(),
                name: entry.name.to_string(),
            })
            .collect()
    }

    /// Lấy tên provider từ ID
    pub fn get_provider_name(provider_id: &str) -> Option<String> {
        PROVIDER_REGISTRY
            .get(provider_id)
            .map(|entry| entry.name.to_string())
    }
}
