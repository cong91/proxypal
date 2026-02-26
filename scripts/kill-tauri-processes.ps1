<#
.SYNOPSIS
    Kill all Tauri/Cargo related processes to fix PermissionDenied OS error 5

.DESCRIPTION
    This script forcefully terminates all processes that may lock files during
    Tauri development, causing "Access is denied" errors on Windows.
    
    Processes killed:
    - cargo.exe (Rust build system)
    - tauri-app.exe (Tauri dev app)
    - proxypal.exe (Sidecar CLI proxy)
    - node.exe (Vite dev server)
    - rust-analyzer.exe (LSP - optional)

.EXAMPLE
    .\kill-tauri-processes.ps1
    
    .\kill-tauri-processes.ps1 -Verbose
    
    .\kill-tauri-processes.ps1 -IncludeRustAnalyzer

.NOTES
    Run with Administrator privileges for best results
#>

param(
    [switch]$IncludeRustAnalyzer,
    [switch]$CleanTarget,
    [switch]$RestartDev
)

$ErrorActionPreference = 'Continue'

function Write-Step {
    param([string]$Message)
    Write-Host "`n[STEP] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[OK] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# =============================================================================
# STEP 1: List processes before killing
# =============================================================================
Write-Step "Scanning for Tauri-related processes..."

$processNames = @(
    'cargo',
    'tauri-app',
    'proxypal',
    'node',
    'vite'
)

if ($IncludeRustAnalyzer) {
    $processNames += 'rust-analyzer'
}

$foundProcesses = @()
foreach ($name in $processNames) {
    $procs = Get-Process -Name $name -ErrorAction SilentlyContinue
    if ($procs) {
        $foundProcesses += $procs
        Write-Warn "Found: $name (PID: $($procs.Id -join ', '))"
    }
}

if ($foundProcesses.Count -eq 0) {
    Write-Success "No Tauri-related processes found running"
} else {
    Write-Host "Found $($foundProcesses.Count) process(es) to terminate" -ForegroundColor Yellow
}

# =============================================================================
# STEP 2: Kill processes with increasing force
# =============================================================================
Write-Step "Terminating processes..."

# Try graceful termination first, then force kill
$killList = @(
    @{ Name = 'tauri-app'; Desc = 'Tauri App' },
    @{ Name = 'proxypal'; Desc = 'ProxyPal Sidecar' },
    @{ Name = 'cargo'; Desc = 'Cargo Build' },
    @{ Name = 'node'; Desc = 'Node.js/Vite' },
    @{ Name = 'vite'; Desc = 'Vite Dev Server' }
)

if ($IncludeRustAnalyzer) {
    $killList += @{ Name = 'rust-analyzer'; Desc = 'Rust Analyzer' }
}

foreach ($item in $killList) {
    $procs = Get-Process -Name $item.Name -ErrorAction SilentlyContinue
    if ($procs) {
        Write-Host "  Killing $($item.Desc)..." -NoNewline
        try {
            # Try graceful stop first
            $procs | Stop-Process -Force -ErrorAction Stop
            Write-Host " DONE" -ForegroundColor Green
        }
        catch {
            # If that fails, try taskkill
            $procIds = $procs.Id -join ' '
            $result = Start-Process -FilePath 'taskkill.exe' -ArgumentList "/F /IM $($item.Name).exe" -Wait -WindowStyle Hidden -PassThru
            if ($result.ExitCode -eq 0) {
                Write-Host " DONE (forced)" -ForegroundColor Green
            } else {
                Write-Host " FAILED" -ForegroundColor Red
            }
        }
    }
}

# Wait for processes to fully terminate
Write-Host "  Waiting for processes to terminate..." -NoNewline
Start-Sleep -Seconds 2
Write-Host " DONE" -ForegroundColor Green

# =============================================================================
# STEP 3: Verify all processes are killed
# =============================================================================
Write-Step "Verifying process termination..."

$stillRunning = @()
foreach ($name in $processNames) {
    $procs = Get-Process -Name $name -ErrorAction SilentlyContinue
    if ($procs) {
        $stillRunning += $name
    }
}

if ($stillRunning.Count -eq 0) {
    Write-Success "All processes terminated successfully"
} else {
    Write-Error "Some processes still running: $($stillRunning -join ', ')"
    Write-Host "`nTrying nuclear option (taskkill /F /IM)..." -ForegroundColor Yellow
    
    foreach ($name in $stillRunning) {
        Start-Process -FilePath 'taskkill.exe' -ArgumentList "/F /IM $name.exe /T" -Wait -WindowStyle Hidden
    }
}

# =============================================================================
# STEP 4: Clean target/debug folder (optional)
# =============================================================================
if ($CleanTarget) {
    Write-Step "Cleaning target/debug folder..."
    
    $targetPaths = @(
        'src-tauri/target/debug',
        'src-tauri/target/release'
    )
    
    foreach ($path in $targetPaths) {
        if (Test-Path $path) {
            Write-Host "  Removing $path..." -NoNewline
            try {
                Remove-Item -Path $path -Recurse -Force -ErrorAction Stop
                Write-Host " DONE" -ForegroundColor Green
            }
            catch {
                Write-Host " FAILED" -ForegroundColor Red
                Write-Warn $_.Exception.Message
            }
        }
    }
}

# =============================================================================
# STEP 5: Check for file locks with handle.exe (if available)
# =============================================================================
Write-Step "Checking for file locks..."

$handlePath = 'C:\Sysinternals\handle.exe'
if (Test-Path $handlePath) {
    Write-Host "  Using Sysinternals Handle to find locks..." -ForegroundColor Cyan
    $locks = & $handlePath 'tauri-app' 2>$null
    if ($locks) {
        Write-Warn "Found file locks:"
        $locks | Select-Object -First 10 | ForEach-Object { Write-Host "    $_" }
    } else {
        Write-Success "No file locks detected"
    }
} else {
    Write-Host "  (Install Sysinternals Handle for detailed lock info)" -ForegroundColor DarkGray
    Write-Host "  Download: https://docs.microsoft.com/sysinternals/downloads/handle" -ForegroundColor DarkGray
}

# =============================================================================
# STEP 6: Summary and recommendations
# =============================================================================
Write-Step "Summary"

Write-Host @"

╔══════════════════════════════════════════════════════════════════════════════╗
║                     EMERGENCY FIX COMPLETED                                  ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  Next steps to resume development:                                           ║
║                                                                              ║
║  1. If you have VS Code terminal open:                                       ║
║     - Close all terminal tabs                                                ║
║     - OR run: pnpm tauri dev                                                 ║
║                                                                              ║
║  2. If using PowerShell directly:                                            ║
║     pnpm tauri dev                                                           ║
║                                                                              ║
║  3. To prevent future issues:                                                ║
║     - Always stop dev server with Ctrl+C before closing terminal             ║
║     - Run this script before: pnpm tauri build                               ║
║     - Consider adding Windows Defender exclusion (see scripts/README.md)     ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝

"@ -ForegroundColor Cyan

# =============================================================================
# STEP 7: Auto-restart dev server (optional)
# =============================================================================
if ($RestartDev) {
    Write-Step "Auto-restarting dev server in 3 seconds..."
    Start-Sleep -Seconds 3
    
    Write-Host "  Starting: pnpm tauri dev`n" -ForegroundColor Green
    & pnpm tauri dev
}
