#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Advanced Sidecar File Unlock Script - Uses Sysinternals handle.exe to find and release file locks
.DESCRIPTION
    This script finds which processes are locking the sidecar binary and attempts to release them.
    It can use Sysinternals handle.exe for detailed handle information.
.PARAMETER KillProcesses
    Kill processes holding the file handle (including Explorer if necessary)
.PARAMETER StopServices
    Stop Windows services that might lock the file (Search, Defender)
.PARAMETER Force
    Force kill even system processes (USE WITH CAUTION)
#>

param(
    [switch]$KillProcesses,
    [switch]$StopServices,
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$sidecarPath = Join-Path $PSScriptRoot "..\src-tauri\binaries\cli-proxy-api-x86_64-pc-windows-msvc.exe"
$sidecarPath = Resolve-Path $sidecarPath -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Path
$sidecarName = "cli-proxy-api-x86_64-pc-windows-msvc.exe"

function Write-Header($text) {
    Write-Host "`n=== $text ===" -ForegroundColor Cyan
}

function Write-Success($text) {
    Write-Host "✓ $text" -ForegroundColor Green
}

function Write-Warn($text) {
    Write-Host "⚠ $text" -ForegroundColor Yellow
}

function Write-Error($text) {
    Write-Host "✗ $text" -ForegroundColor Red
}

Write-Header "ADVANCED SIDECAR FILE UNLOCK"
Write-Host "Target: $sidecarPath"
Write-Host "KillProcesses: $KillProcesses"
Write-Host "StopServices: $StopServices"
Write-Host "Force: $Force"

# ========== STEP 1: Check if file exists ==========
Write-Header "STEP 1: Checking File Existence"

if (-not (Test-Path $sidecarPath)) {
    Write-Warn "Sidecar file not found at expected path"
    # Search for it
    $searchPath = Join-Path $PSScriptRoot "..\src-tauri\binaries"
    $foundFiles = Get-ChildItem $searchPath -Filter "cli-proxy-api*.exe" -ErrorAction SilentlyContinue
    if ($foundFiles) {
        Write-Host "Found these sidecar files:"
        $foundFiles | ForEach-Object { Write-Host "  - $($_.FullName)" }
        $sidecarPath = $foundFiles[0].FullName
        Write-Host "Using: $sidecarPath"
    } else {
        Write-Error "No sidecar files found in $searchPath"
        exit 1
    }
} else {
    Write-Success "Sidecar file exists"
}

# ========== STEP 2: Method 1 - Using Resource Monitor (resmon) data ==========
Write-Header "STEP 2: Detecting File Locks via WMI/Handles"

# Try to find processes using the file using handle.exe if available
$handleExe = Get-Command "handle.exe" -ErrorAction SilentlyContinue
$handleExe = $handleExe ?? (Get-Command "$env:USERPROFILE\Downloads\handle.exe" -ErrorAction SilentlyContinue)
$handleExe = $handleExe ?? (Get-Command "C:\Sysinternals\handle.exe" -ErrorAction SilentlyContinue)
$handleExe = $handleExe ?? (Get-Command "$env:LOCALAPPDATA\Microsoft\WindowsApps\handle.exe" -ErrorAction SilentlyContinue)

if ($handleExe) {
    Write-Host "Found handle.exe at: $($handleExe.Source)"
    Write-Host "`nSearching for handles to sidecar binary..."
    
    & $handleExe.Source -a -u $sidecarName 2>&1 | Tee-Object -Variable handleOutput | Write-Host
    
    if ($handleOutput -match "pid:\s*(\d+)") {
        $processIds = $handleOutput | Select-String -Pattern "pid:\s*(\d+)" | ForEach-Object {
            [int]($_.Matches.Groups[1].Value)
        } | Sort-Object -Unique
        
        Write-Host "`nProcesses holding handles: $($processIds -join ', ')"
        
        if ($KillProcesses -and $processIds) {
            foreach ($procId in $processIds) {
                try {
                    $proc = Get-Process -Id $procId -ErrorAction SilentlyContinue
                    if ($proc) {
                        Write-Warn "Killing process: $($proc.ProcessName) (PID: $procId)"
                        Stop-Process -Id $procId -Force
                        Start-Sleep -Milliseconds 500
                    }
                } catch {
                    Write-Error "Failed to kill PID $procId`: $_"
                }
            }
        }
    }
} else {
    Write-Warn "handle.exe not found. Install from: https://docs.microsoft.com/sysinternals/downloads/handle"
    Write-Host "Attempting alternative methods..."
}

# ========== STEP 3: Method 2 - Find processes by name ==========
Write-Header "STEP 3: Searching for Related Processes"

$processPatterns = @(
    "cli-proxy-api*",
    "proxypal*",
    "tauri*",
    "cargo*",
    "rustc*"
)

foreach ($pattern in $processPatterns) {
    $procs = Get-Process | Where-Object { $_.ProcessName -like $pattern } -ErrorAction SilentlyContinue
    if ($procs) {
        Write-Host "Found '$pattern' processes:"
        $procs | ForEach-Object { 
            Write-Host "  - $($_.ProcessName) (PID: $($_.Id))"
            if ($KillProcesses) {
                try {
                    Stop-Process -Id $_.Id -Force
                    Write-Success "Killed PID $($_.Id)"
                } catch {
                    Write-Error "Failed to kill PID $($_.Id): $_"
                }
            }
        }
    }
}

# ========== STEP 4: Method 3 - Using OpenFiles ==========
Write-Header "STEP 4: Using OpenFiles Command"

$openFilesEnabled = (openfiles /local 2>&1) -match "ENABLED"
if (-not $openFilesEnabled) {
    Write-Warn "OpenFiles tracking not enabled. Run 'openfiles /local on' and restart to enable."
} else {
    $openFilesOutput = openfiles /query /fo csv 2>&1
    $lockedFiles = $openFilesOutput | Select-String -SimpleMatch $sidecarName
    if ($lockedFiles) {
        Write-Host "Found open handles:"
        $lockedFiles | ForEach-Object { Write-Host "  $_" }
    } else {
        Write-Host "No handles found via OpenFiles"
    }
}

# ========== STEP 5: Stop Services ==========
if ($StopServices) {
    Write-Header "STEP 5: Stopping Potentially Locking Services"
    
    $services = @(
        "WSearch",      # Windows Search
        "WinDefend",    # Windows Defender
        "MsMpSvc",      # Windows Defender Antimalware Service
        "WdNisSvc",     # Windows Defender Network Inspection
        "wscsvc"        # Windows Security Center
    )
    
    foreach ($svcName in $services) {
        $svc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
        if ($svc -and $svc.Status -eq "Running") {
            Write-Warn "Stopping service: $($svc.DisplayName)"
            try {
                Stop-Service -Name $svcName -Force -ErrorAction Stop
                Write-Success "Stopped $($svc.DisplayName)"
            } catch {
                Write-Error "Failed to stop $($svc.DisplayName): $_"
            }
        }
    }
}

# ========== STEP 6: Kill Explorer if necessary ==========
if ($Force) {
    Write-Header "STEP 6: Force Mode - Restarting Explorer"
    
    $explorerPid = (Get-Process explorer -ErrorAction SilentlyContinue).Id
    if ($explorerPid) {
        Write-Warn "Killing Explorer.exe (desktop will flicker)"
        Stop-Process -Name explorer -Force
        Start-Sleep 2
        Start-Process explorer
        Write-Success "Explorer restarted"
    }
}

# ========== STEP 7: Final Check ==========
Write-Header "STEP 7: Final File Lock Check"

try {
    $stream = [System.IO.File]::Open($sidecarPath, [System.IO.FileMode]::Open, [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::None)
    $stream.Close()
    $stream.Dispose()
    Write-Success "File is now UNLOCKED and accessible!"
} catch {
    Write-Error "File is STILL LOCKED: $_"
    Write-Host "`nRecommendations:"
    Write-Host "1. Run: handle.exe $sidecarName" -ForegroundColor Yellow
    Write-Host "2. Download handle.exe from: https://docs.microsoft.com/sysinternals/downloads/handle" -ForegroundColor Yellow
    Write-Host "3. Use Resource Monitor (resmon.exe) > CPU tab > Associated Handles search" -ForegroundColor Yellow
    Write-Host "4. Last resort: Restart computer" -ForegroundColor Yellow
}
