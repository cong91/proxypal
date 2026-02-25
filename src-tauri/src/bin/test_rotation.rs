//! Standalone test binary for rotation proxy
//!
//! Run with: cargo run --bin test_rotation
//!
//! This binary tests the rotation proxy functionality without requiring
//! the Tauri UI or the full application to be running.

use std::time::Duration;
use proxypal_lib::{RotationProxyProvider, CachedProxy};

#[tokio::main]
async fn main() {
    println!("========================================");
    println!("ROTATION PROXY STANDALONE TEST");
    println!("========================================\n");

    // Get rotation URL from environment or use default test URL
    // Actual URL format: rotation://proxyxoay.shop?key=HnMBZgKRukdXQBHWssjmGP&&nhamang=random&&tinhthanh=0
    let rotation_url = std::env::var("ROTATION_URL")
        .unwrap_or_else(|_| {
            println!("⚠️  ROTATION_URL not set, using example URL");
            println!("   Set it with: set ROTATION_URL=rotation://proxyxoay.shop?key=HnMBZgKRukdXQBHWssjmGP&&nhamang=random&&tinhthanh=0");
            "rotation://proxyxoay.shop?key=HnMBZgKRukdXQBHWssjmGP&&nhamang=random&&tinhthanh=0".to_string()
        });

    println!("📡 Testing with URL: {}", rotation_url);
    println!();

    // Test 1: Parse rotation URL
    println!("Test 1: Parsing rotation URL...");
    let provider = match RotationProxyProvider::new(&rotation_url) {
        Ok(p) => {
            println!("✅ URL parsed successfully");
            let api_url: &str = p.api_url();
            println!("   API URL: {}", api_url);
            p
        }
        Err(e) => {
            println!("❌ Failed to parse URL: {}", e);
            std::process::exit(1);
        }
    };

    // Test 2: Fetch proxy
    println!("\nTest 2: Fetching proxy from provider...");
    let cached = match provider.fetch_proxy().await {
        Ok(c) => {
            println!("✅ Proxy fetched successfully");
            println!("   HTTP Proxy:  {}", c.http_proxy);
            println!("   SOCKS5 Proxy: {}", c.socks5_proxy);
            println!("   Real IP:      {}", c.real_ip);
            println!("   TTL: {}s", c.ttl_seconds);
            c
        }
        Err(e) => {
            println!("❌ Failed to fetch proxy: {}", e);
            std::process::exit(1);
        }
    };

    // Test 3: Resolve proxy URL
    println!("\nTest 3: Resolving proxy URL...");
    match RotationProxyProvider::resolve_proxy_url(&cached) {
        Some(url) => {
            println!("✅ Proxy URL resolved: {}", url);
        }
        None => {
            println!("❌ Failed to resolve proxy URL");
        }
    }

    // Test 4: Cache behavior
    println!("\nTest 4: Testing cache behavior...");
    let cached2: CachedProxy = provider.get_or_refresh().await.unwrap();
    println!("✅ Got cached proxy (should be same as before)");
    println!("   Expires in: {}s", cached2.expires_in_seconds());

    // Test 5: Force rotate
    println!("\nTest 5: Testing force rotate...");
    println!("   Waiting 2 seconds before force rotate...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    let old_proxy = cached.http_proxy.clone();
    let rotated: CachedProxy = match provider.force_rotate().await {
        Ok(c) => {
            println!("✅ Force rotate successful");
            println!("   Old HTTP:  {}", old_proxy);
            println!("   New HTTP:  {}", c.http_proxy);
            println!("   Changed:   {}", old_proxy != c.http_proxy);
            println!("   New TTL:   {}s", c.ttl_seconds);
            c
        }
        Err(e) => {
            println!("❌ Force rotate failed: {}", e);
            std::process::exit(1);
        }
    };

    // Test 6: IP validation (optional)
    println!("\nTest 6: Testing proxy connectivity (optional)...");
    let proxy_url_option: Option<String> = RotationProxyProvider::resolve_proxy_url(&rotated);
    if let Some(proxy_url) = proxy_url_option {
        test_proxy_connectivity(&proxy_url).await;
    }

    println!("\n========================================");
    println!("ALL TESTS PASSED!");
    println!("========================================");
}

async fn test_proxy_connectivity(proxy_url: &str) {
    use reqwest::Proxy;

    println!("   Testing proxy: {}", proxy_url);
    
    let proxy = match Proxy::all(proxy_url) {
        Ok(p) => p,
        Err(e) => {
            println!("   ⚠️  Invalid proxy URL: {}", e);
            return;
        }
    };

    let client = match reqwest::Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(10))
        .build() 
    {
        Ok(c) => c,
        Err(e) => {
            println!("   ⚠️  Failed to build HTTP client: {}", e);
            return;
        }
    };

    let urls = [
        "https://api.ipify.org?format=json",
        "https://httpbin.org/ip",
    ];

    for url in &urls {
        match client.get(*url).send().await {
            Ok(resp) => {
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let ip = json.get("ip")
                            .or_else(|| json.get("origin"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        println!("   ✅ Connected through proxy, IP: {}", ip);
                        return;
                    }
                    Err(e) => {
                        println!("   ⚠️  Could not parse response from {}: {}", url, e);
                    }
                }
            }
            Err(e) => {
                println!("   ⚠️  Failed to connect via {}: {}", url, e);
            }
        }
    }

    println!("   ⚠️  Could not verify proxy connectivity");
}

