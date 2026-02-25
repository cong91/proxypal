use std::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};
use tauri_plugin_shell::process::CommandChild;

use crate::types::{ProxyStatus, AuthStatus, OAuthState, CopilotStatus};
use crate::config::AppConfig;
use crate::proxy::RotationProxyProvider;

/// App state shared across all Tauri commands
pub struct AppState {
    pub proxy_status: Mutex<ProxyStatus>,
    pub auth_status: Mutex<AuthStatus>,
    pub config: Mutex<AppConfig>,
    pub pending_oauth: Mutex<Option<OAuthState>>,
    pub proxy_process: Mutex<Option<CommandChild>>,
    pub copilot_status: Mutex<CopilotStatus>,
    pub copilot_process: Mutex<Option<CommandChild>>,
    pub log_watcher_running: Arc<AtomicBool>,
    pub request_counter: Arc<AtomicU64>,
    /// Rotation proxy provider (when rotation:// URL is used)
    pub rotation_provider: Mutex<Option<Arc<RotationProxyProvider>>>,
    /// TTL monitor cancellation flag (separate from log_watcher for clean shutdown)
    pub ttl_monitor_running: Arc<AtomicBool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            proxy_status: Mutex::new(ProxyStatus::default()),
            auth_status: Mutex::new(AuthStatus::default()),
            config: Mutex::new(AppConfig::default()),
            pending_oauth: Mutex::new(None),
            proxy_process: Mutex::new(None),
            copilot_status: Mutex::new(CopilotStatus::default()),
            copilot_process: Mutex::new(None),
            log_watcher_running: Arc::new(AtomicBool::new(false)),
            request_counter: Arc::new(AtomicU64::new(0)),
            rotation_provider: Mutex::new(None),
            ttl_monitor_running: Arc::new(AtomicBool::new(false)),
        }
    }
}
