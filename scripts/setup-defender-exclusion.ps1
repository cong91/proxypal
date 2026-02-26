<#
.SYNOPSIS
    Setup Windows Defender exclusions for Tauri development

.DESCRIPTION
    Adds Windows Defender exclusions for:
    - src-tauri/target folder (build artifacts)
    - pnpm/node_modules (dependency scanning)
    - cargo registry/cache
    
    This prevents real-time scanning from locking files during builds.

.REQUIREMENTS
    Run as Administrator

.EXAMPLE
    .\setup-defender-exclusion.ps1
    
    .\setup-defender-exclusion.ps1 -DryRun (show what would be done)
#>

param(
    [switch]$DryRun,
    [switch]$Force
)

# Check if running as admin
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")

if (-not $isAdmin) {
    Write-Error "This script must be run as Administrator!`nRight-click PowerShell and select 'Run as Administrator'"
    exit 1
}

# Get project paths
$projectRoot = Resolve-Path "$PSScriptRoot\.."
$targetPath = Join-Path $projectRoot "src-tauri\target"
$nodeModulesPath = Join-Path $projectRoot "node_modules"
$cargoPath = "$env:USERPROFILE\.cargo"
$cargoRegistry = "$env:USERPROFILE\.cargo\registry"

Write-Host @"
╔════════════════════════════════════════════════════════════════╗
║     Windows Defender Exclusion Setup for Tauri Dev            ║
╚════════════════════════════════════════════════════════════════╝

Project Root: $projectRoot

Paths to exclude:
  1. $targetPath
  2. $nodeModulesPath
  3. $cargoRegistry

"@ -ForegroundColor Cyan

if ($DryRun) {
    Write-Host "[DRY RUN] No changes will be made" -ForegroundColor Yellow
    exit 0
}

# Confirm unless -Force
if (-not $Force) {
    $confirm = Read-Host "Continue? (y/N)"
    if ($confirm -ne 'y' -and $confirm -ne 'Y') {
        Write-Host "Aborted." -ForegroundColor Yellow
        exit 0
    }
}

# Function to add exclusion
function Add-DefenderExclusion {
    param(
        [string]$Path,
        [string]$Type = 'Path'
    )
    
    if (-not (Test-Path $Path)) {
        Write-Warn "Path does not exist: $Path"
        return
    }
    
    try {
        $existing = Get-MpPreference | Select-Object -ExpandProperty ExclusionPath
        if ($existing -contains $Path) {
            Write-Host "  [SKIP] Already excluded: $Path" -ForegroundColor DarkGray
            return
        }
        
        Add-MpPreference -ExclusionPath $Path
        Write-Host "  [OK] Added exclusion: $Path" -ForegroundColor Green
    }
    catch {
        Write-Error "Failed to add exclusion: $_"
    }
}

Write-Host "`nAdding exclusions..." -ForegroundColor Cyan

Add-DefenderExclusion -Path $targetPath
Add-DefenderExclusion -Path $nodeModulesPath
Add-DefenderExclusion -Path $cargoRegistry

Write-Host "`n✅ Done! Windows Defender will now skip these folders during real-time scanning." -ForegroundColor Green
Write-Host "`nNote: You may need to restart your terminal for changes to take full effect." -ForegroundColor Yellow
