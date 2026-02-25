use std::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::Instant;
use tauri_plugin_shell::process::CommandChild;

use crate::types::{ProxyStatus, AuthStatus, OAuthState, CopilotStatus};
use crate::config::AppConfig;
use crate::proxy::DynProxyProvider;

/// Tracks the initialization state of rotation proxy
#[derive(Debug, Clone)]
pub enum RotationInitState {
    /// Not initialized or not configured
    Idle,
    /// Initialization in progress
    Initializing,
    /// Successfully initialized with cached proxy
    Initialized {
        proxy_url: String,
        expires_at: Instant,
    },
    /// Failed to initialize
    Failed {
        error: String,
        retry_after: Option<Instant>,
    },
}

impl Default for RotationInitState {
    fn default() -> Self {
        RotationInitState::Idle
    }
}

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
    /// Rotation proxy provider (dynamic trait object)
    pub rotation_provider: Mutex<Option<DynProxyProvider>>,
    /// TTL monitor cancellation flag (separate from log_watcher for clean shutdown)
    pub ttl_monitor_running: Arc<AtomicBool>,
    /// Rotation provider initialization state
    pub rotation_init_state: Mutex<RotationInitState>,
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
            rotation_init_state: Mutex::new(RotationInitState::default()),
        }
    }
}
