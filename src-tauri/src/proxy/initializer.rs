//! Proxy Rotation Initializer
//!
//! Handles automatic initialization of proxy rotation on app startup.
//! Runs in background to avoid blocking UI thread.

use std::sync::Arc;
use std::time::{Duration, Instant};
use log::{info, warn, error};
use tauri::{AppHandle, Emitter, Manager};

use crate::config::AppConfig;
use crate::proxy::{ProxyProviderFactory, DynProxyProvider};
use crate::state::{AppState, RotationInitState};
use crate::types::proxy::{CachedProxy, RotationProxySettings};

/// Manages automatic initialization of proxy rotation
pub struct ProxyRotationInitializer {
    /// Singleflight guard to prevent concurrent fetches
    initialization_in_progress: Arc<tokio::sync::Mutex<bool>>,
    /// Retry configuration
    max_retries: u32,
    base_retry_delay_ms: u64,
}

impl ProxyRotationInitializer {
    pub fn new() -> Self {
        Self {
            initialization_in_progress: Arc::new(tokio::sync::Mutex::new(false)),
            max_retries: 3,
            base_retry_delay_ms: 2000,
        }
    }

    /// Initialize proxy rotation in background
    /// This function is non-blocking and returns immediately
    pub fn initialize(&self, app_handle: AppHandle) {
        let initializer = Arc::new(self.clone());
        
        tauri::async_runtime::spawn(async move {
            info!("[RotationInit] Starting background initialization");
            
            // Load config
            let config = crate::config::load_config();
            
            // Check if rotation is enabled
            if !Self::is_rotation_enabled(&config) {
                info!("[RotationInit] Rotation not enabled, skipping initialization");
                return;
            }
            
            // Update state to Initializing
            {
                let state = app_handle.state::<AppState>();
                if let Ok(mut init_state) = state.rotation_init_state.lock() {
                    *init_state = RotationInitState::Initializing;
                };
            }
            
            // Perform initialization with singleflight
            let result = initializer.initialize_with_singleflight(&app_handle, config).await;
            
            // Handle result
            match result {
                Ok(()) => {
                    info!("[RotationInit] Background initialization completed successfully");
                }
                Err(e) => {
                    error!("[RotationInit] Background initialization failed: {}", e);
                    
                    // Update state to Failed
                    let state = app_handle.state::<AppState>();
                    if let Ok(mut init_state) = state.rotation_init_state.lock() {
                        *init_state = RotationInitState::Failed {
                            error: e.clone(),
                            retry_after: Some(Instant::now() + Duration::from_secs(60)),
                        };
                    };
                    
                    // Emit failure event
                    let _ = app_handle.emit("rotation-proxy-initialized", serde_json::json!({
                        "success": false,
                        "error": e,
                    }));
                }
            }
        });
    }

    /// Check if rotation proxy is configured
    /// Returns true if URL starts with rotation:// regardless of rotation_settings
    fn is_rotation_enabled(config: &AppConfig) -> bool {
        if config.use_system_proxy {
            // System proxy doesn't support rotation
            return false;
        }
        
        // Enable rotation if URL starts with rotation://
        // rotation_settings can be parsed from URL if not present
        config.proxy_url.starts_with("rotation://")
    }

    /// Parse rotation settings from rotation:// URL
    /// Format: rotation://domain?key=xxx&nhamang=xxx&tinhthanh=xxx
    fn parse_rotation_settings_from_url(url: &str) -> Option<RotationProxySettings> {
        if !url.starts_with("rotation://") {
            return None;
        }

        // Remove rotation:// prefix
        let url_part = &url["rotation://".len()..];
        
        // Split domain and query params
        let parts: Vec<&str> = url_part.split('?').collect();
        if parts.is_empty() {
            return None;
        }
        
        let domain = parts[0].trim();
        
        // Parse query parameters
        let mut api_key = String::new();
        let mut network_type = "random".to_string();
        let mut location_filter = "0".to_string();
        
        if parts.len() > 1 {
            let query = parts[1];
            for param in query.split('&') {
                let kv: Vec<&str> = param.splitn(2, '=').collect();
                if kv.len() == 2 {
                    let key = kv[0];
                    let value = kv[1];
                    match key {
                        "key" => api_key = value.to_string(),
                        "nhamang" => network_type = value.to_string(),
                        "tinhthanh" => location_filter = value.to_string(),
                        _ => {}
                    }
                }
            }
        }
        
        // Detect provider from domain
        let provider_id = if domain.contains("proxyxoay.shop") || domain.contains("proxy.vn") {
            "proxy_vn"
        } else {
            "proxy_vn" // default
        };
        
        Some(RotationProxySettings {
            provider_id: provider_id.to_string(),
            api_key,
            network_type,
            location_filter,
            custom_api_url: None,
        })
    }

    /// Initialize with singleflight pattern (prevents duplicate requests)
    async fn initialize_with_singleflight(
        &self,
        app_handle: &AppHandle,
        config: AppConfig,
    ) -> Result<(), String> {
        // Try to acquire initialization lock
        let mut in_progress = self.initialization_in_progress.lock().await;
        
        if *in_progress {
            warn!("[RotationInit] Initialization already in progress, skipping");
            return Err("Initialization already in progress".to_string());
        }
        
        *in_progress = true;
        drop(in_progress); // Release lock before async operations
        
        let result = self.do_initialize(app_handle, config).await;
        
        // Reset flag
        let mut in_progress = self.initialization_in_progress.lock().await;
        *in_progress = false;
        
        result
    }

    /// Actual initialization logic with retry
    async fn do_initialize(
        &self,
        app_handle: &AppHandle,
        mut config: AppConfig,
    ) -> Result<(), String> {
        let effective_url = config.proxy_url.clone();
        
        // Parse rotation settings from URL if not present in config
        if config.rotation_settings.is_none() {
            if let Some(settings) = Self::parse_rotation_settings_from_url(&effective_url) {
                info!("[RotationInit] Parsed rotation settings from URL");
                config.rotation_settings = Some(settings);
            }
        }
        
        info!("[RotationInit] Creating provider for URL: {}", effective_url);
        
        // Create provider
        let provider = ProxyProviderFactory::create(&effective_url)
            .map_err(|e| format!("Failed to create provider: {}", e))?;
        
        // Store provider in state
        {
            let state = app_handle.state::<AppState>();
            if let Ok(mut rotation_provider) = state.rotation_provider.lock() {
                *rotation_provider = Some(provider.clone());
            };
        }
        
        // Fetch initial proxy with retry
        let cached = self.fetch_with_retry(&provider).await?;
        
        let proxy_url = crate::proxy::providers::ProxyVNProvider::resolve_proxy_url(&cached).unwrap_or_default();
        let real_ip = cached.real_ip.clone();
        let ttl_seconds = cached.ttl_seconds;
        let expires_in = cached.expires_in_seconds();
        let expires_at = Instant::now() + Duration::from_secs(expires_in);
        
        info!(
            "[RotationInit] Successfully initialized with proxy: {}, expires in {}s",
            proxy_url,
            expires_in
        );
        
        // Update state to Initialized
        {
            let state = app_handle.state::<AppState>();
            if let Ok(mut init_state) = state.rotation_init_state.lock() {
                *init_state = RotationInitState::Initialized {
                    proxy_url: proxy_url.clone(),
                    expires_at,
                };
            };
        }
        
        // Emit initialization event to frontend
        let _ = app_handle.emit("rotation-proxy-initialized", serde_json::json!({
            "success": true,
            "proxy": proxy_url,
            "realIp": real_ip,
            "ttl": ttl_seconds,
            "expiresInSeconds": expires_in,
        }));
        
        Ok(())
    }

    /// Fetch proxy with exponential backoff retry
    async fn fetch_with_retry(
        &self,
        provider: &DynProxyProvider,
    ) -> Result<CachedProxy, String> {
        let mut last_error = String::new();
        
        for attempt in 0..self.max_retries {
            match provider.get_or_refresh().await {
                Ok(cached) => {
                    info!("[RotationInit] Fetch succeeded on attempt {}", attempt + 1);
                    return Ok(cached);
                }
                Err(e) => {
                    last_error = e.clone();
                    error!(
                        "[RotationInit] Fetch attempt {} failed: {}",
                        attempt + 1,
                        e
                    );
                    
                    // Check if this is a cooldown error - don't retry immediately
                    if e.contains("[WAITING_STATUS]") {
                        warn!("[RotationInit] Provider on cooldown, will retry later");
                        // Parse wait time if possible
                        if let Some(wait_secs) = Self::parse_cooldown_seconds(&e) {
                            // Wait for cooldown + small buffer
                            let delay = Duration::from_secs(wait_secs + 5);
                            warn!("[RotationInit] Waiting {}s before retry", delay.as_secs());
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                    }
                    
                    if attempt < self.max_retries - 1 {
                        let delay = Duration::from_millis(
                            self.base_retry_delay_ms * 2_u64.pow(attempt)
                        );
                        warn!("[RotationInit] Retrying in {}ms", delay.as_millis());
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(format!(
            "Failed to fetch proxy after {} attempts: {}",
            self.max_retries,
            last_error
        ))
    }

    /// Parse cooldown seconds from error message
    fn parse_cooldown_seconds(error: &str) -> Option<u64> {
        // Format: [WAITING_STATUS]30|Message
        let re = regex::Regex::new(r"\[WAITING_STATUS\](\d+)\|").ok()?;
        re.captures(error)?
            .get(1)?
            .as_str()
            .parse::<u64>()
            .ok()
    }
}

impl Clone for ProxyRotationInitializer {
    fn clone(&self) -> Self {
        Self {
            initialization_in_progress: Arc::new(tokio::sync::Mutex::new(false)),
            max_retries: self.max_retries,
            base_retry_delay_ms: self.base_retry_delay_ms,
        }
    }
}
