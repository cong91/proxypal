#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Master Sidecar Fix Script - Ultimate solution for PermissionDenied errors
.DESCRIPTION
    Comprehensive script that tries multiple methods in order:
    1. Process termination (Tauri, cargo, sidecar)
    2. File unlock using handle.exe
    3. Aggressive cleanup
    4. Alternative unlock methods
    5. Windows Defender exclusions
    
.PARAMETER Nuclear
    Maximum aggression mode - kills explorer, disables services
.PARAMETER Quick
    Quick mode - only essential steps
.PARAMETER SkipBuild
    Skip rebuild at the end
#>

param(
    [switch]$Nuclear,
    [switch]$Quick,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Continue"
$projectRoot = Resolve-Path (Join-Path $PSScriptRoot "..") | Select-Object -ExpandProperty Path

function Write-Color($text, $color) {
    Write-Host $text -ForegroundColor $color
}

function Write-Header($text) {
    Write-Host "`n========================================" -ForegroundColor Magenta
    Write-Host "  $text" -ForegroundColor Magenta
    Write-Host "========================================" -ForegroundColor Magenta
}

function Write-Success($text) { Write-Host "✓ $text" -ForegroundColor Green }
function Write-Error($text) { Write-Host "✗ $text" -ForegroundColor Red }
function Write-Warn($text) { Write-Host "⚠ $text" -ForegroundColor Yellow }
function Write-Info($text) { Write-Host "ℹ $text" -ForegroundColor Cyan }

Write-Color @"
   ____  _  _  ___   ___   ___  _  _  ___   ___ 
  |_  / | || |/ _ \ / _ \ / _ \| \| |/ __| | _ 
   / /  | __ | (_) | (_) | (_) | .  | (_ | |   /
  /___| |_||_|\___/ \___/ \___/|_|\_|\___| |_|_\\
"@ Magenta

Write-Host "`n  Master Sidecar Fix Tool" -ForegroundColor White
Write-Host "  Project: $projectRoot`n" -ForegroundColor Gray

if (-not $Quick) {
    Write-Host "Press Ctrl+C to cancel, or wait 3 seconds to continue..." -ForegroundColor Yellow
    Start-Sleep -Seconds 3
}

$stepsCompleted = 0
$totalSteps = if ($Nuclear) { 8 } else { 6 }

function Step($name, $action) {
    $script:stepsCompleted++
    Write-Header "STEP $stepsCompleted/$totalSteps`: $name"
    try {
        & $action
        return $true
    } catch {
        Write-Error "Step failed: $_"
        return $false
    }
}

# ========== STEP 1: Kill Processes ==========
Step "Kill Related Processes" {
    $targets = @(
        @{ Name = "cli-proxy-api*"; Desc = "Sidecar" },
        @{ Name = "proxypal*"; Desc = "App" },
        @{ Name = "tauri*"; Desc = "Tauri" },
        @{ Name = "cargo*"; Desc = "Cargo" }
    )
    
    foreach ($t in $targets) {
        Get-Process | Where-Object { $_.ProcessName -like $t.Name } | ForEach-Object {
            try {
                Stop-Process -Id $_.Id -Force
                Write-Success "Killed $($t.Desc): PID $($_.Id)"
            } catch {
                Write-Warn "Could not kill $($_.ProcessName)"
            }
        }
    }
    Start-Sleep -Seconds 2
}

# ========== STEP 2: Try handle.exe ==========
if (-not $Quick) {
    Step "Using Sysinternals Handle" {
        $handlePaths = @(
            "handle.exe",
            "$env:USERPROFILE\Downloads\handle.exe",
            "C:\Sysinternals\handle.exe"
        )
        
        $handleExe = $handlePaths | Where-Object { Test-Path $_ } | Select-Object -First 1
        
        if ($handleExe) {
            Write-Info "Found handle.exe at: $handleExe"
            $output = & $handleExe "cli-proxy-api" -a 2>&1
            $output | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
            
            # Parse and kill PIDs
            $pids = $output | Select-String -Pattern "pid:\s*(\d+)" | ForEach-Object { $_.Matches.Groups[1].Value } | Sort-Object -Unique
            $pids | ForEach-Object {
                try {
                    Stop-Process -Id $_ -Force
                    Write-Success "Killed process holding handle: PID $_"
                } catch {}
            }
        } else {
            Write-Warn "handle.exe not found. Download from: https://docs.microsoft.com/sysinternals/downloads/handle"
        }
    }
}

# ========== STEP 3: Aggressive Cleanup ==========
Step "Aggressive Cleanup" {
    $paths = @(
        "$projectRoot\src-tauri\target",
        "$projectRoot\src-tauri\binaries\*.exe"
    )
    
    foreach ($p in $paths) {
        Get-Item $p -ErrorAction SilentlyContinue | ForEach-Object {
            try {
                Remove-Item $_.FullName -Recurse -Force -ErrorAction Stop
                Write-Success "Deleted: $($_.Name)"
            } catch {
                Write-Warn "Could not delete: $($_.Name) - trying alternative..."
                # Try robocopy method
                $empty = "$env:TEMP\empty$([Guid]::NewGuid())"
                New-Item -ItemType Directory $empty -Force | Out-Null
                & robocopy $empty $_.FullName /MIR /R:1 /W:1 2>&1 | Out-Null
                Remove-Item $empty -Force -Recurse
            }
        }
    }
    
    # Cargo clean
    Push-Location "$projectRoot\src-tauri"
    & cargo clean 2>&1 | Out-Null
    Pop-Location
    Write-Success "Cargo clean completed"
}

# ========== STEP 4: Alternative Methods ==========
if (-not $Quick) {
    Step "Alternative Unlock Methods" {
        $sidecar = "$projectRoot\src-tauri\binaries\cli-proxy-api-x86_64-pc-windows-msvc.exe"
        
        if (Test-Path $sidecar) {
            try {
                # Try rename-then-delete
                $newName = "to_delete_$(Get-Random).tmp"
                Rename-Item $sidecar $newName -Force
                Remove-Item "$projectRoot\src-tauri\binaries\$newName" -Force
                Write-Success "Renamed and deleted"
            } catch {
                # Schedule for reboot deletion
                Add-Type -TypeDefinition "using System; using System.Runtime.InteropServices; public class X { [DllImport(\"kernel32.dll\",CharSet=CharSet.Unicode)] public static extern bool MoveFileEx(string a,string b,int c); public const int Y=0x4; }" -ErrorAction SilentlyContinue
                if ($?) {
                    [X]::MoveFileEx($sidecar, $null, [X]::Y) | Out-Null
                    Write-Warn "Scheduled for deletion on REBOOT"
                }
            }
        }
    }
}

# ========== STEP 5: Windows Defender (Optional) ==========
if (-not $Quick) {
    Step "Windows Defender Exclusions" {
        try {
            Add-MpPreference -ExclusionPath "$projectRoot" -ErrorAction Stop
            Add-MpPreference -ExclusionPath "$projectRoot\src-tauri\target" -ErrorAction SilentlyContinue
            Add-MpPreference -ExclusionExtension ".exe" -ErrorAction SilentlyContinue
            Write-Success "Added Defender exclusions"
            Write-Warn "Remember to remove exclusions when done!"
        } catch {
            Write-Warn "Could not add exclusions (may need manual action)"
        }
    }
}

# ========== STEP 6: Nuclear Options ==========
if ($Nuclear) {
    Step "NUCLEAR: Stop Services" {
        $services = @("WSearch", "WinDefend")
        $services | ForEach-Object {
            $svc = Get-Service $_ -ErrorAction SilentlyContinue
            if ($svc -and $svc.Status -eq "Running") {
                Stop-Service $_ -Force -ErrorAction SilentlyContinue
                Write-Success "Stopped: $_"
            }
        }
    }
    
    Step "NUCLEAR: Restart Explorer" {
        Stop-Process -Name explorer -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
        Start-Process explorer
        Write-Success "Explorer restarted"
    }
}

# ========== FINAL: Verification ==========
Write-Header "VERIFICATION"

$sidecarPath = "$projectRoot\src-tauri\binaries\cli-proxy-api-x86_64-pc-windows-msvc.exe"
if (Test-Path $sidecarPath) {
    try {
        $s = [System.IO.File]::Open($sidecarPath, [System.IO.FileMode]::Open, [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::None)
        $s.Close()
        $s.Dispose()
        Write-Success "File is UNLOCKED"
    } catch {
        Write-Error "File is STILL LOCKED"
        Write-Host "`nTry running with -Nuclear flag or restart Windows"
    }
} else {
    Write-Success "File not found (was deleted) - ready to rebuild"
}

# ========== REBUILD ==========
if (-not $SkipBuild) {
    Write-Header "REBUILD"
    Write-Host "Choose build option:" -ForegroundColor Cyan
    Write-Host "1. pnpm tauri dev (development)"
    Write-Host "2. pnpm tauri build (production)"
    Write-Host "3. Skip (manual build)"
    
    $choice = Read-Host "`nEnter 1, 2, or 3"
    
    switch ($choice) {
        "1" {
            Write-Host "`nStarting development build...`n" -ForegroundColor Green
            Set-Location $projectRoot
            & pnpm tauri dev
        }
        "2" {
            Write-Host "`nStarting production build...`n" -ForegroundColor Green
            Set-Location $projectRoot
            & pnpm tauri build
        }
        default {
            Write-Host "`nSkipping build. Run manually:" -ForegroundColor Yellow
            Write-Host "  cd $projectRoot"
            Write-Host "  pnpm tauri dev"
        }
    }
}

Write-Header "COMPLETE"
Write-Host "Fix process completed with $stepsCompleted steps`n" -ForegroundColor Green
