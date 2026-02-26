//! Rotation Proxy Provider Module
//!
//! This module implements a dynamic proxy rotation mechanism using Strategy Pattern.
//! It supports multiple proxy providers through a common trait interface.
//!
//! ## Architecture
//!
//! - `trait.rs`: Defines the [`ProxyProvider`] trait (Strategy Pattern)
//! - `base.rs`: Contains [`ProxyProviderBase`] with common caching logic
//! - `factory.rs`: Factory pattern for creating providers
//! - `providers/`: Concrete provider implementations
//!
//! ## Usage
//!
//! ```ignore
//! use proxypal_lib::proxy::{ProxyProviderFactory, ProxyProvider};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), String> {
//!     // Create provider using factory
//!     let provider = ProxyProviderFactory::create("rotation://proxyxoay.shop?key=XXX")?;
//!
//!     // Get or refresh cached proxy
//!     let cached = provider.get_or_refresh().await?;
//!
//!     // Resolve to standard proxy URL
//!     let proxy_url = proxypal_lib::proxy::providers::ProxyVNProvider::resolve_proxy_url(&cached);
//!     Ok(())
//! }
//! ```

pub mod trait_ext;
pub mod base;
pub mod factory;
pub mod providers;

// Re-export core types
pub use trait_ext::{ProxyProvider, DynProxyProvider};
pub use base::ProxyProviderBase;
pub use factory::ProxyProviderFactory;

// Re-export providers
pub use providers::ProxyVNProvider;

// Re-export ProviderInfo from types for consistency
pub use crate::types::proxy::ProviderInfo;

// Legacy re-export for backward compatibility
// This maintains compatibility with existing code that uses RotationProxyProvider
pub use providers::ProxyVNProvider as RotationProxyProvider;

// Legacy constants for backward compatibility
// These are kept to maintain API compatibility with existing code
pub use crate::types::proxy::{CachedProxy, RotationConfig, RotationProxyResponse, RotationStatus};
