<#
.SYNOPSIS
    Pre-dev hook to prevent PermissionDenied errors

.DESCRIPTION
    Runs automatically before pnpm tauri dev to ensure no locked processes
    or files that would cause build failures.

.USAGE
    Add to package.json scripts:
    "predev": "powershell -ExecutionPolicy Bypass -File scripts/pre-dev.ps1",
    "dev": "pnpm predev && pnpm tauri dev"
#>

param([switch]$Verbose)

$ErrorActionPreference = 'SilentlyContinue'

# Quick check for running processes
$processes = @('tauri-app', 'proxypal', 'cargo')
$found = $false

foreach ($proc in $processes) {
    if (Get-Process -Name $proc -ErrorAction SilentlyContinue) {
        $found = $true
        break
    }
}

if ($found) {
    Write-Host "[pre-dev] Detected locked processes, running cleanup..." -ForegroundColor Yellow
    & "$PSScriptRoot\kill-tauri-processes.ps1"
    Start-Sleep -Seconds 1
} else {
    if ($Verbose) {
        Write-Host "[pre-dev] No locked processes detected" -ForegroundColor DarkGray
    }
}

# Check if target/debug/tauri-app.exe exists and is writable
$exePath = 'src-tauri/target/debug/tauri-app.exe'
if (Test-Path $exePath) {
    try {
        $stream = [System.IO.File]::Open($exePath, 'Open', 'Write')
        $stream.Close()
        if ($Verbose) {
            Write-Host "[pre-dev] Target executable is writable" -ForegroundColor DarkGray
        }
    }
    catch {
        Write-Host "[pre-dev] Target executable is locked! Running cleanup..." -ForegroundColor Red
        & "$PSScriptRoot\kill-tauri-processes.ps1"
    }
}

exit 0
