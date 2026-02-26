#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Windows Defender/AV Workaround Script
.DESCRIPTION
    Manages Windows Defender exclusions and real-time protection settings
    to prevent file locking during Tauri builds.

.PARAMETER Action
    Action to perform: AddExclusion, RemoveExclusion, DisableRealtime, EnableRealtime, Status
.PARAMETER Path
    Path to exclude (default: project root)
#>

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("AddExclusion", "RemoveExclusion", "DisableRealtime", "EnableRealtime", "Status")]
    [string]$Action,
    
    [string]$Path = ""
)

$ErrorActionPreference = "Stop"

# Determine project path
if ([string]::IsNullOrEmpty($Path)) {
    $Path = Resolve-Path (Join-Path $PSScriptRoot "..") | Select-Object -ExpandProperty Path
}
$Path = Resolve-Path $Path | Select-Object -ExpandProperty Path

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

Write-Header "WINDOWS DEFENDER WORKAROUND"
Write-Host "Action: $Action"
Write-Host "Project Path: $Path"

# Check if Windows Defender is available
try {
    $defenderStatus = Get-MpComputerStatus -ErrorAction Stop
    if (-not $defenderStatus.AntivirusEnabled) {
        Write-Warn "Windows Defender Antivirus is not enabled"
    }
} catch {
    Write-Error "Windows Defender not available or access denied: $_"
    exit 1
}

switch ($Action) {
    "Status" {
        Write-Header "DEFENDER STATUS"
        
        $status = Get-MpComputerStatus
        Write-Host "Antivirus Enabled: $($status.AntivirusEnabled)"
        Write-Host "Real-time Protection: $($status.RealTimeProtectionEnabled)"
        Write-Host "Behavior Monitor: $($status.BehaviorMonitorEnabled)"
        Write-Host "On Access Protection: $($status.OnAccessProtectionEnabled)"
        
        Write-Host "`nCurrent Exclusions:"
        $exclusions = Get-MpPreference | Select-Object -ExpandProperty ExclusionPath
        if ($exclusions) {
            $exclusions | ForEach-Object { Write-Host "  - $_" }
        } else {
            Write-Host "  (none)"
        }
        
        Write-Host "`nExclusion Extensions:"
        $extExclusions = Get-MpPreference | Select-Object -ExpandProperty ExclusionExtension
        if ($extExclusions) {
            $extExclusions | ForEach-Object { Write-Host "  - $_" }
        } else {
            Write-Host "  (none)"
        }
    }
    
    "AddExclusion" {
        Write-Header "ADDING EXCLUSIONS"
        
        $exclusions = @(
            $Path,
            "$Path\src-tauri\target",
            "$Path\src-tauri\binaries",
            "$env:CARGO_HOME",
            "$env:RUSTUP_HOME"
        )
        
        # Filter out null paths
        $exclusions = $exclusions | Where-Object { -not [string]::IsNullOrEmpty($_) }
        
        foreach ($exclPath in $exclusions) {
            if (Test-Path $exclPath) {
                try {
                    Add-MpPreference -ExclusionPath $exclPath -ErrorAction Stop
                    Write-Success "Added exclusion: $exclPath"
                } catch {
                    Write-Error "Failed to add exclusion for $exclPath`: $_"
                }
            } else {
                Write-Warn "Path does not exist, skipping: $exclPath"
            }
        }
        
        # Also add extension exclusions
        $extExclusions = @(".exe", ".dll", ".tmp")
        foreach ($ext in $extExclusions) {
            try {
                Add-MpPreference -ExclusionExtension $ext -ErrorAction Stop
                Write-Success "Added extension exclusion: $ext"
            } catch {
                Write-Warn "Could not add extension exclusion $ext (may already exist)"
            }
        }
        
        Write-Host "`n✅ Exclusions added! Windows Defender will skip scanning these paths."
        Write-Host "⚠️  Note: This reduces security for these paths. Remove exclusions when done."
    }
    
    "RemoveExclusion" {
        Write-Header "REMOVING EXCLUSIONS"
        
        $exclusions = @(
            $Path,
            "$Path\src-tauri\target",
            "$Path\src-tauri\binaries"
        )
        
        foreach ($exclPath in $exclusions) {
            try {
                Remove-MpPreference -ExclusionPath $exclPath -ErrorAction Stop
                Write-Success "Removed exclusion: $exclPath"
            } catch {
                Write-Warn "Could not remove exclusion (may not exist): $exclPath"
            }
        }
        
        Write-Host "`n✅ Exclusions removed! Windows Defender will now scan these paths."
    }
    
    "DisableRealtime" {
        Write-Header "DISABLING REAL-TIME PROTECTION"
        Write-Warn "⚠️  This temporarily reduces system security!"
        Write-Warn "⚠️  Real-time protection will be disabled until you re-enable it or restart."
        
        $confirm = Read-Host "`nType 'DISABLE' to confirm"
        if ($confirm -ne "DISABLE") {
            Write-Host "Cancelled."
            exit 0
        }
        
        try {
            Set-MpPreference -DisableRealtimeMonitoring $true -ErrorAction Stop
            Write-Success "Real-time protection DISABLED"
            Write-Host "`n⚠️  IMPORTANT: Re-enable protection when build is complete!"
            Write-Host "   Run: .\scripts\defender-workaround.ps1 -Action EnableRealtime"
        } catch {
            Write-Error "Failed to disable real-time protection: $_"
            Write-Host "`n💡 This may be controlled by group policy or your organization's settings."
        }
    }
    
    "EnableRealtime" {
        Write-Header "ENABLING REAL-TIME PROTECTION"
        
        try {
            Set-MpPreference -DisableRealtimeMonitoring $false -ErrorAction Stop
            Write-Success "Real-time protection ENABLED"
        } catch {
            Write-Error "Failed to enable real-time protection: $_"
        }
    }
}

Write-Host "`n📋 Next Steps:"
Write-Host "   - Check status: .\scripts\defender-workaround.ps1 -Action Status"
Write-Host "   - Remove exclusions when done: .\scripts\defender-workaround.ps1 -Action RemoveExclusion"
