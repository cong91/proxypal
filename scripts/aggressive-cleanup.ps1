#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Aggressive Cleanup Script - Nukes all build artifacts and locked files
.DESCRIPTION
    This script performs aggressive cleanup including:
    - Complete target folder deletion
    - Sidecar binary removal
    - Cargo clean with lock override
    - Temp file cleanup
.PARAMETER Nuke
    Complete destruction mode - kills all processes and deletes everything
.PARAMETER SkipConfirmation
    Skip all confirmation prompts
#>

param(
    [switch]$Nuke,
    [switch]$SkipConfirmation
)

$ErrorActionPreference = "Continue"
$projectRoot = Resolve-Path (Join-Path $PSScriptRoot "..") | Select-Object -ExpandProperty Path

function Write-Header($text) {
    Write-Host "`n========================================" -ForegroundColor Red
    Write-Host "  $text" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
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

function Write-Info($text) {
    Write-Host "ℹ $text" -ForegroundColor Cyan
}

Write-Header "AGGRESSIVE CLEANUP MODE"
Write-Host "Project Root: $projectRoot"
Write-Host "Nuke Mode: $Nuke"

if (-not $SkipConfirmation) {
    Write-Host "`n⚠️  WARNING: This will delete ALL build artifacts!" -ForegroundColor Red
    Write-Host "⚠️  Including: target/, binaries/*.exe, node_modules/.cache" -ForegroundColor Red
    if ($Nuke) {
        Write-Host "⚠️  NUKE MODE: Will also kill ALL related processes!" -ForegroundColor Magenta
    }
    
    $confirm = Read-Host "`nType 'YES' to continue"
    if ($confirm -ne "YES") {
        Write-Host "Cancelled."
        exit 0
    }
}

$successCount = 0
$failCount = 0

# ========== PHASE 1: Process Termination ==========
if ($Nuke) {
    Write-Header "PHASE 1: PROCESS TERMINATION"
    
    $targets = @(
        @{ Name = "cli-proxy-api*"; Desc = "Sidecar binary" },
        @{ Name = "proxypal*"; Desc = "Tauri app" },
        @{ Name = "tauri*"; Desc = "Tauri processes" },
        @{ Name = "cargo*"; Desc = "Cargo/rustc" },
        @{ Name = "rustc*"; Desc = "Rust compiler" },
        @{ Name = "node*"; Desc = "Node.js" }
    )
    
    foreach ($target in $targets) {
        $procs = Get-Process | Where-Object { $_.ProcessName -like $target.Name } -ErrorAction SilentlyContinue
        if ($procs) {
            Write-Warn "Found $($procs.Count) $($target.Desc) process(es)"
            foreach ($proc in $procs) {
                try {
                    Stop-Process -Id $proc.Id -Force -ErrorAction Stop
                    Write-Success "Killed: $($proc.ProcessName) (PID: $($proc.Id))"
                } catch {
                    Write-Error "Failed to kill PID $($proc.Id): $_"
                }
            }
        }
    }
    
    Start-Sleep -Seconds 2
}

# ========== PHASE 2: Target Folder Annihilation ==========
Write-Header "PHASE 2: TARGET FOLDER DESTRUCTION"

$targetPaths = @(
    "$projectRoot\src-tauri\target",
    "$projectRoot\src-tauri\binaries\*.exe"
)

foreach ($path in $targetPaths) {
    $items = Get-Item $path -ErrorAction SilentlyContinue
    if (-not $items) { 
        $items = Get-ChildItem $path -ErrorAction SilentlyContinue 
    }
    
    if ($items) {
        foreach ($item in $items) {
            Write-Host "Deleting: $($item.FullName)"
            try {
                if ($item.PSIsContainer) {
                    # Try multiple methods for directories
                    try {
                        Remove-Item $item.FullName -Recurse -Force -ErrorAction Stop
                    } catch {
                        # If normal delete fails, try robocopy mirror method
                        $emptyDir = "$env:TEMP\empty_for_delete"
                        New-Item -ItemType Directory -Path $emptyDir -Force | Out-Null
                        & robocopy $emptyDir $item.FullName /MIR /R:1 /W:1 | Out-Null
                        Remove-Item $item.FullName -Recurse -Force -ErrorAction SilentlyContinue
                        Remove-Item $emptyDir -Recurse -Force -ErrorAction SilentlyContinue
                    }
                } else {
                    # For files, try rename then delete method
                    $tempName = "$env:TEMP\to_delete_$(Get-Random).tmp"
                    try {
                        Move-Item $item.FullName $tempName -Force -ErrorAction Stop
                        Remove-Item $tempName -Force -ErrorAction Stop
                    } catch {
                        Remove-Item $item.FullName -Force -ErrorAction Stop
                    }
                }
                Write-Success "Deleted: $($item.Name)"
                $successCount++
            } catch {
                Write-Error "Failed to delete $($item.Name): $_"
                $failCount++
            }
        }
    } else {
        Write-Info "Not found: $path"
    }
}

# ========== PHASE 3: Cargo Clean ==========
Write-Header "PHASE 3: CARGO CLEAN"

Push-Location "$projectRoot\src-tauri"
try {
    Write-Host "Running cargo clean..."
    $cargoOutput = & cargo clean 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Cargo clean completed"
        $successCount++
    } else {
        Write-Error "Cargo clean failed: $cargoOutput"
        $failCount++
    }
} catch {
    Write-Error "Cargo clean exception: $_"
    $failCount++
} finally {
    Pop-Location
}

# ========== PHASE 4: Temp File Cleanup ==========
Write-Header "PHASE 4: TEMP FILE CLEANUP"

$tempPatterns = @(
    "$env:TEMP\cargo-*",
    "$env:TEMP\rust*",
    "$env:TEMP\tauri*",
    "$env:TEMP\*.tmp"
)

foreach ($pattern in $tempPatterns) {
    $files = Get-ChildItem $pattern -ErrorAction SilentlyContinue
    if ($files) {
        foreach ($file in $files) {
            try {
                Remove-Item $file.FullName -Recurse -Force -ErrorAction Stop
                Write-Success "Cleaned: $($file.Name)"
            } catch {
                Write-Warn "Could not delete: $($file.Name)"
            }
        }
    }
}

# ========== PHASE 5: Node/Frontend Cleanup ==========
Write-Header "PHASE 5: FRONTEND CLEANUP"

$frontendPaths = @(
    "$projectRoot\dist",
    "$projectRoot\node_modules\.cache",
    "$projectRoot\.vite"
)

foreach ($path in $frontendPaths) {
    if (Test-Path $path) {
        Write-Host "Deleting: $path"
        try {
            Remove-Item $path -Recurse -Force -ErrorAction Stop
            Write-Success "Deleted"
            $successCount++
        } catch {
            Write-Error "Failed: $_"
            $failCount++
        }
    }
}

# ========== SUMMARY ==========
Write-Header "CLEANUP SUMMARY"
Write-Host "Successful operations: $successCount"
Write-Host "Failed operations: $failCount"

if ($failCount -gt 0) {
    Write-Host "`n⚠️  Some operations failed. Try:" -ForegroundColor Yellow
    Write-Host "   1. Run as Administrator" -ForegroundColor Yellow
    Write-Host "   2. Run with -Nuke flag to kill all processes" -ForegroundColor Yellow
    Write-Host "   3. Download and use handle.exe from Sysinternals" -ForegroundColor Yellow
    Write-Host "   4. Restart computer if all else fails" -ForegroundColor Yellow
}
