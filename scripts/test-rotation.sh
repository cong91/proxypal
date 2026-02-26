#!/bin/bash
# Test script for rotation proxy
# Usage: ./test-rotation.sh [proxy_port] [management_key]
#
# Actual rotation URL format (uses && instead of &):
#   rotation://proxyxoay.shop?key=HnMBZgKRukdXQBHWssjmGP&&nhamang=random&&tinhthanh=0

set -e

PROXY_PORT="${1:-8317}"
MGMT_KEY="${2:-${MGMT_KEY:-}}"
PROXY_URL="http://127.0.0.1:${PROXY_PORT}"
MGMT_URL="http://127.0.0.1:${PROXY_PORT}/v0/management"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp=$(date '+%H:%M:%S')
    local color="${NC}"
    
    case "$level" in
        INFO) color="${CYAN}" ;;
        WARN) color="${YELLOW}" ;;
        ERROR) color="${RED}" ;;
        SUCCESS) color="${GREEN}" ;;
    esac
    
    echo -e "${color}[${timestamp}] [${level}]${NC} ${message}"
}

get_current_ip() {
    local urls=(
        "https://api.ipify.org?format=json"
        "https://httpbin.org/ip"
        "https://api64.ipify.org?format=json"
    )
    
    for url in "${urls[@]}"; do
        local result
        if result=$(curl -s -x "$PROXY_URL" --max-time 10 "$url" 2>/dev/null); then
            local ip=$(echo "$result" | grep -oP '"ip":\s*"\K[^"]+' || echo "$result" | grep -oP '"origin":\s*"\K[^"]+')
            if [ -n "$ip" ]; then
                echo "$ip|$url"
                return 0
            fi
        fi
    done
    return 1
}

# Main
log INFO "========================================"
log INFO "ROTATION PROXY TEST"
log INFO "========================================"
log INFO "Proxy URL: $PROXY_URL"
log INFO ""

# Check proxy
log INFO "Checking proxy connection..."
if curl -s --max-time 5 "$PROXY_URL/v1/models" > /dev/null 2>&1; then
    log SUCCESS "Proxy is running"
else
    log ERROR "Proxy is not responding"
    log INFO "Make sure the backend is running with: cd src-tauri && cargo run"
    exit 1
fi

# Get initial IP
log INFO "Getting initial IP..."
if initial_data=$(get_current_ip); then
    initial_ip=$(echo "$initial_data" | cut -d'|' -f1)
    source_url=$(echo "$initial_data" | cut -d'|' -f2)
    log SUCCESS "Initial IP: ${YELLOW}${initial_ip}${NC} (from ${source_url})"
else
    log ERROR "Failed to get initial IP"
    exit 1
fi

# Trigger rotation if key provided
if [ -n "$MGMT_KEY" ]; then
    log INFO ""
    log INFO "Triggering rotation via Management API..."
    if curl -s -X PUT \
        -H "Content-Type: application/json" \
        -H "X-Management-Key: $MGMT_KEY" \
        -d '{"value":""}' \
        "$MGMT_URL/proxy-url" > /dev/null 2>&1; then
        log SUCCESS "Rotation triggered"
        
        log INFO "Waiting 3 seconds for update..."
        sleep 3
        
        # Get new IP
        log INFO ""
        log INFO "Getting new IP after rotation..."
        if new_data=$(get_current_ip); then
            new_ip=$(echo "$new_data" | cut -d'|' -f1)
            
            if [ "$new_ip" != "$initial_ip" ]; then
                log SUCCESS "${GREEN}IP changed!${NC}"
                log SUCCESS "New IP: ${YELLOW}${new_ip}${NC}"
                log SUCCESS "Old IP: ${RED}${initial_ip}${NC}"
            else
                log WARN "${YELLOW}IP did not change${NC}"
                log INFO "Current IP: $new_ip"
                log INFO "Possible reasons:"
                log INFO "  - Rotation provider returned same IP"
                log INFO "  - Proxy cache not yet updated"
                log INFO "  - Management API key incorrect"
            fi
        else
            log ERROR "Failed to get new IP"
        fi
    else
        log ERROR "Failed to trigger rotation"
    fi
else
    log WARN ""
    log WARN "Management key not provided, skipping rotation test"
    log INFO "To test rotation, provide management key:"
    log INFO "  $0 ${PROXY_PORT} <management_key>"
    log INFO "Or set environment variable:"
    log INFO "  MGMT_KEY=<key> $0"
fi

log INFO ""
log INFO "========================================"
log INFO "TEST COMPLETE"
log INFO "========================================"
