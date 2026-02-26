<#
.SYNOPSIS
    Test rotation proxy functionality

.DESCRIPTION
    This script tests the rotation proxy by checking IP through local proxy
    and optionally triggering rotation via Management API.
    
    Actual rotation URL format (uses && instead of &):
    rotation://proxyxoay.shop?key=HnMBZgKRukdXQBHWssjmGP&&nhamang=random&&tinhthanh=0

.PARAMETER Port
    Local proxy port (default: 8317)

.PARAMETER ManagementKey
    Management API key for triggering rotation

.EXAMPLE
    .\test-rotation.ps1
    
    .\test-rotation.ps1 -Port 8080 -ManagementKey "your_key_here"
#>

param(
    [int]$Port = 8317,
    [string]$ManagementKey = ""
)

$ProxyUrl = "http://127.0.0.1:$Port"
$MgmtUrl = "http://127.0.0.1:$Port/v0/management"

function Write-Log {
    param(
        [string]$Level,
        [string]$Message
    )
    $timestamp = Get-Date -Format "HH:mm:ss"
    $colors = @{
        "INFO" = "Cyan"
        "WARN" = "Yellow"
        "ERROR" = "Red"
        "SUCCESS" = "Green"
    }
    $color = $colors[$Level] ?? "White"
    Write-Host "[$timestamp] [$Level] $Message" -ForegroundColor $color
}

function Get-CurrentIP {
    $urls = @(
        "https://api.ipify.org?format=json"
        "https://httpbin.org/ip"
        "https://api64.ipify.org?format=json"
    )

    foreach ($url in $urls) {
        try {
            # Use the local proxy
            $proxy = New-Object System.Net.WebProxy($ProxyUrl)
            $request = Invoke-RestMethod -Uri $url -Proxy $proxy -TimeoutSec 10 -ErrorAction Stop
            
            $ip = $request.ip ?? $request.origin
            if ($ip) {
                return @{ IP = $ip; Source = $url }
            }
        }
        catch {
            continue
        }
    }
    throw "Could not determine current IP"
}

function Test-ProxyConnection {
    try {
        $response = Invoke-RestMethod -Uri "$ProxyUrl/v1/models" -TimeoutSec 5
        return $true
    }
    catch {
        return $false
    }
}

# Main
Write-Log "INFO" "========================================"
Write-Log "INFO" "ROTATION PROXY TEST"
Write-Log "INFO" "========================================"
Write-Log "INFO" "Proxy URL: $ProxyUrl"
Write-Log "INFO" ""

# Check proxy
Write-Log "INFO" "Checking proxy connection..."
if (Test-ProxyConnection) {
    Write-Log "SUCCESS" "Proxy is running"
}
else {
    Write-Log "ERROR" "Proxy is not responding"
    Write-Log "INFO" "Make sure the backend is running with: cd src-tauri; cargo run"
    exit 1
}

# Get initial IP
Write-Log "INFO" "Getting initial IP..."
try {
    $initial = Get-CurrentIP
    $initialIP = $initial.IP
    Write-Log "SUCCESS" "Initial IP: $initialIP (from $($initial.Source))"
}
catch {
    Write-Log "ERROR" "Failed to get initial IP: $($_.Exception.Message)"
    exit 1
}

# Trigger rotation if key provided
if ($ManagementKey) {
    Write-Log "INFO" ""
    Write-Log "INFO" "Triggering rotation via Management API..."
    try {
        $headers = @{
            "X-Management-Key" = $ManagementKey
        }
        $body = @{ value = "" } | ConvertTo-Json
        
        $response = Invoke-RestMethod -Uri "$MgmtUrl/proxy-url" -Method PUT -Headers $headers -Body $body -ContentType "application/json"
        Write-Log "SUCCESS" "Rotation triggered"
        
        Write-Log "INFO" "Waiting 3 seconds for update..."
        Start-Sleep -Seconds 3
        
        # Get new IP
        Write-Log "INFO" ""
        Write-Log "INFO" "Getting new IP after rotation..."
        $new = Get-CurrentIP
        $newIP = $new.IP
        
        if ($newIP -ne $initialIP) {
            Write-Log "SUCCESS" "IP changed!"
            Write-Log "SUCCESS" "New IP: $newIP"
            Write-Log "SUCCESS" "Old IP: $initialIP"
        }
        else {
            Write-Log "WARN" "IP did not change"
            Write-Log "INFO" "Current IP: $newIP"
            Write-Log "INFO" "Possible reasons:"
            Write-Log "INFO" "  - Rotation provider returned same IP"
            Write-Log "INFO" "  - Proxy cache not yet updated"
            Write-Log "INFO" "  - Management API key incorrect"
        }
    }
    catch {
        Write-Log "ERROR" "Rotation failed: $($_.Exception.Message)"
    }
}
else {
    Write-Log "WARN" ""
    Write-Log "WARN" "Management key not provided, skipping rotation test"
    Write-Log "INFO" "To test rotation, run with ManagementKey parameter:"
    Write-Log "INFO" "  .\test-rotation.ps1 -ManagementKey 'your_key'"
}

Write-Log "INFO" ""
Write-Log "INFO" "========================================"
Write-Log "INFO" "TEST COMPLETE"
Write-Log "INFO" "========================================"
