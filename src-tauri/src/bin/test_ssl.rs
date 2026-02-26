//! Test binary để kiểm tra SSL/TLS connection đến proxyxoay.shop
//!
//! Chạy với: cargo run --bin test_ssl
//!
//! Binary này test riêng việc kết nối HTTPS đến proxyxoay.shop
//! để xác định xem lỗi có phải do SSL/TLS không.

use std::time::Duration;
use std::error::Error;

#[tokio::main]
async fn main() {
    println!("========================================");
    println!("SSL/TLS CONNECTION TEST");
    println!("========================================\n");

    let test_url = "https://proxyxoay.shop/api/get.php?key=HnMBZgKRukdXQBHWssjmGP&nhamang=random&tinhthanh=0";

    println!("📡 Testing URL: {}", test_url);
    println!();

    // Test 1: Client mặc định (có SSL verification)
    println!("Test 1: Client với SSL verification (default)...");
    match test_with_client(test_url, false).await {
        Ok(body) => {
            println!("✅ SUCCESS (SSL verification enabled)");
            println!("   Response preview: {}...", &body[..body.len().min(200)]);
        }
        Err(e) => {
            println!("❌ FAILED (SSL verification enabled)");
            print_error_details(e.as_ref());
        }
    }
    println!();

    // Test 2: Client không kiểm tra SSL (danger_accept_invalid_certs)
    println!("Test 2: Client với SSL verification DISABLED...");
    match test_with_client(test_url, true).await {
        Ok(body) => {
            println!("✅ SUCCESS (SSL verification disabled)");
            println!("   Response preview: {}...", &body[..body.len().min(200)]);
        }
        Err(e) => {
            println!("❌ FAILED (SSL verification disabled)");
            print_error_details(e.as_ref());
        }
    }
    println!();

    // Test 3: Thử với native-tls backend
    println!("Test 3: Thông tin hệ thống...");
    println!("   OS: {}", std::env::consts::OS);
    println!("   Arch: {}", std::env::consts::ARCH);
    println!();

    println!("========================================");
    println!("TEST COMPLETE");
    println!("========================================");
}

async fn test_with_client(url: &str, disable_ssl: bool) -> Result<String, Box<dyn Error>> {
    let client_builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(30));

    let client = if disable_ssl {
        client_builder.danger_accept_invalid_certs(true).build()?
    } else {
        client_builder.build()?
    };

    let response = client.get(url).send().await?;
    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        return Err(format!("HTTP error: {}", status).into());
    }

    Ok(body)
}

fn print_error_details(e: &(dyn Error + 'static)) {
    println!("   Error: {}", e);

    let mut current = e.source();
    let mut depth = 0;
    while let Some(source) = current {
        println!("   Source[{}]: {}", depth, source);
        current = source.source();
        depth += 1;
    }

    // In thêm thông tin nếu là reqwest error
    if let Some(reqwest_err) = e.downcast_ref::<reqwest::Error>() {
        println!("   Is connect error: {}", reqwest_err.is_connect());
        println!("   Is timeout: {}", reqwest_err.is_timeout());
        println!("   Is request error: {}", reqwest_err.is_request());
        if let Some(status) = reqwest_err.status() {
            println!("   Status: {}", status);
        }
    }
}
