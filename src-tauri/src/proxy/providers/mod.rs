//! Proxy providers module
//!
//! Contains concrete implementations of the ProxyProvider trait
//! for different proxy rotation services.

pub mod proxy_vn;

pub use proxy_vn::ProxyVNProvider;
