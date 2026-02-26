#!/usr/bin/env node
/**
 * Test script for rotation proxy
 * 
 * This script tests the rotation proxy feature by:
 * 1. Getting current IP through local proxy
 * 2. Triggering rotation via Management API
 * 3. Getting new IP and comparing
 * 
 * Usage:
 *   node scripts/test-rotation.mjs [proxy_port] [management_key]
 * 
 * Environment variables:
 *   PROXY_PORT - Local proxy port (default: 8317)
 *   MGMT_KEY - Management API key (optional, reads from config if not provided)
 */

import { setTimeout } from 'timers/promises';

// Configuration
const PROXY_PORT = process.env.PROXY_PORT || process.argv[2] || 8317;
const MGMT_KEY = process.env.MGMT_KEY || process.argv[3] || '';
// Actual rotation URL for reference (uses && instead of & as separator)
// rotation://proxyxoay.shop?key=HnMBZgKRukdXQBHWssjmGP&&nhamang=random&&tinhthanh=0
const PROXY_URL = `http://127.0.0.1:${PROXY_PORT}`;
const MGMT_URL = `http://127.0.0.1:${PROXY_PORT}/v0/management`;

// Test targets for IP detection
const IP_CHECK_URLS = [
  'https://api.ipify.org?format=json',
  'https://httpbin.org/ip',
  'https://api64.ipify.org?format=json',
];

const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
};

function log(level, message) {
  const timestamp = new Date().toISOString().split('T')[1].split('.')[0];
  const color = {
    INFO: colors.cyan,
    WARN: colors.yellow,
    ERROR: colors.red,
    SUCCESS: colors.green,
  }[level] || colors.reset;
  
  console.log(`${color}[${timestamp}] [${level}]${colors.reset} ${message}`);
}

/**
 * Get current IP through the local proxy
 */
async function getCurrentIP() {
  for (const url of IP_CHECK_URLS) {
    try {
      const controller = new AbortController();
      const timeout = setTimeout(10000, () => controller.abort());
      
      const response = await fetch(url, {
        method: 'GET',
        signal: controller.signal,
        // Use local proxy
        dispatcher: new (await import('undici')).ProxyAgent(PROXY_URL),
      });
      
      clearTimeout(timeout);
      
      if (!response.ok) continue;
      
      const data = await response.json();
      // ipify returns { ip: "..." }, httpbin returns { origin: "..." }
      const ip = data.ip || data.origin;
      
      if (ip) {
        return { ip, source: url };
      }
    } catch (err) {
      // Try next URL
      continue;
    }
  }
  
  throw new Error('Could not determine current IP from any source');
}

/**
 * Get current proxy configuration via Management API
 */
async function getProxyConfig() {
  const headers = {};
  if (MGMT_KEY) {
    headers['X-Management-Key'] = MGMT_KEY;
  }
  
  try {
    const response = await fetch(`${MGMT_URL}/config`, { headers });
    if (response.ok) {
      return await response.json();
    }
  } catch (err) {
    log('WARN', `Could not get config: ${err.message}`);
  }
  return null;
}

/**
 * Trigger proxy rotation via Management API
 */
async function rotateProxy() {
  if (!MGMT_KEY) {
    throw new Error('Management key is required for rotation. Set MGMT_KEY env var or pass as argument.');
  }
  
  log('INFO', 'Triggering proxy rotation via Management API...');
  
  // First, we need to invalidate the rotation cache and get a new proxy
  // Since there's no direct "rotate" endpoint in the management API,
  // we'll update the proxy-url to trigger a re-fetch
  
  const response = await fetch(`${MGMT_URL}/proxy-url`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
      'X-Management-Key': MGMT_KEY,
    },
    body: JSON.stringify({ value: '' }), // Clear to force refresh
  });
  
  if (!response.ok) {
    throw new Error(`Management API error: ${response.status} ${response.statusText}`);
  }
  
  log('SUCCESS', 'Rotation triggered successfully');
  return await response.json();
}

/**
 * Test direct proxy fetch (without UI)
 */
async function testDirectRotation() {
  log('INFO', '========================================');
  log('INFO', 'ROTATION PROXY TEST');
  log('INFO', '========================================');
  log('INFO', `Proxy URL: ${PROXY_URL}`);
  log('INFO', `Management URL: ${MGMT_URL}`);
  log('INFO', '');
  
  // Check if proxy is running
  try {
    const healthCheck = await fetch(`${PROXY_URL}/v1/models`, {
      method: 'GET',
      signal: AbortSignal.timeout(5000),
    });
    log('SUCCESS', `Proxy is running (status: ${healthCheck.status})`);
  } catch (err) {
    log('ERROR', `Proxy is not responding: ${err.message}`);
    log('INFO', 'Make sure the backend is running with: cd src-tauri && cargo run');
    process.exit(1);
  }
  
  // Get initial IP
  log('INFO', 'Getting initial IP...');
  let initialIP;
  try {
    const result = await getCurrentIP();
    initialIP = result.ip;
    log('SUCCESS', `Initial IP: ${colors.yellow}${initialIP}${colors.reset} (from ${result.source})`);
  } catch (err) {
    log('ERROR', `Failed to get initial IP: ${err.message}`);
    process.exit(1);
  }
  
  // Get current config
  log('INFO', '');
  log('INFO', 'Checking current proxy configuration...');
  const config = await getProxyConfig();
  if (config) {
    log('INFO', `Current proxy-url: ${config['proxy-url'] || '(not set)'}`);
    log('INFO', `Port: ${config.port}`);
  }
  
  // Trigger rotation if management key is available
  if (MGMT_KEY) {
    log('INFO', '');
    log('INFO', 'Attempting to rotate proxy...');
    try {
      await rotateProxy();
    } catch (err) {
      log('ERROR', `Rotation failed: ${err.message}`);
      log('INFO', 'Note: Direct rotation requires Management API access.');
      log('INFO', 'To test rotation without UI, use the Rust test: cargo test debug_live_rotation_provider_e2e -- --nocapture');
    }
    
    // Wait for proxy to update
    log('INFO', 'Waiting 3 seconds for proxy update...');
    await setTimeout(3000);
    
    // Get new IP
    log('INFO', '');
    log('INFO', 'Getting new IP after rotation...');
    try {
      const result = await getCurrentIP();
      const newIP = result.ip;
      
      if (newIP !== initialIP) {
        log('SUCCESS', `${colors.green}IP changed!${colors.reset}`);
        log('SUCCESS', `New IP: ${colors.yellow}${newIP}${colors.reset}`);
        log('SUCCESS', `Old IP: ${colors.red}${initialIP}${colors.reset}`);
      } else {
        log('WARN', `${colors.yellow}IP did not change${colors.reset}`);
        log('INFO', `Current IP: ${newIP}`);
        log('INFO', 'Possible reasons:');
        log('INFO', '  - Rotation provider returned same IP');
        log('INFO', '  - Proxy cache not yet updated');
        log('INFO', '  - Management API key incorrect or missing');
      }
    } catch (err) {
      log('ERROR', `Failed to get new IP: ${err.message}`);
    }
  } else {
    log('WARN', '');
    log('WARN', 'Management key not provided, skipping rotation test');
    log('INFO', 'To test rotation, provide management key:');
    log('INFO', `  node scripts/test-rotation.mjs ${PROXY_PORT} <management_key>`);
    log('INFO', 'Or set environment variable:');
    log('INFO', '  MGMT_KEY=<key> node scripts/test-rotation.mjs');
  }
  
  log('INFO', '');
  log('INFO', '========================================');
  log('INFO', 'TEST COMPLETE');
  log('INFO', '========================================');
}

// Alternative test using curl-like approach via child process
async function testWithCurl() {
  log('INFO', '');
  log('INFO', 'Alternative test using system curl...');
  
  const { spawn } = await import('child_process');
  const { promisify } = await import('util');
  const exec = promisify((cmd, args, callback) => {
    const child = spawn(cmd, args, { stdio: 'pipe' });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (d) => stdout += d);
    child.stderr.on('data', (d) => stderr += d);
    child.on('close', (code) => callback(code ? new Error(stderr || `Exit ${code}`) : null, { stdout, stderr }));
  });
  
  try {
    const proxyUrl = `http://127.0.0.1:${PROXY_PORT}`;
    const result = await exec('curl', [
      '-s', '-x', proxyUrl,
      '--max-time', '10',
      'https://api.ipify.org?format=json'
    ]);
    const data = JSON.parse(result.stdout);
    log('SUCCESS', `curl test passed, IP: ${data.ip}`);
  } catch (err) {
    log('WARN', `curl test failed: ${err.message}`);
    log('INFO', 'Make sure curl is installed');
  }
}

// Run tests
testDirectRotation()
  .then(() => testWithCurl())
  .catch(err => {
    log('ERROR', `Unexpected error: ${err.message}`);
    console.error(err);
    process.exit(1);
  });
