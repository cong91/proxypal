#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Alternative File Unlock Methods - When standard deletion fails
.DESCRIPTION
    Uses creative workarounds to unlock/delete locked files:
    - Rename-then-delete
    - Move to temp then delete
    - MoveFileEx with MOVEFILE_DELAY_UNTIL_REBOOT
    - Robocopy mirror empty folder
.PARAMETER Method
    Method to use: Rename, Move, DelayedDelete, Robocopy, All
#>

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("Rename", "Move", "DelayedDelete", "Robocopy", "All")]
    [string]$Method
)

$ErrorActionPreference = "Stop"
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class NativeMethods {
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    public static extern bool MoveFileEx(string lpExistingFileName, string lpNewFileName, int dwFlags);
    public const int MOVEFILE_DELAY_UNTIL_REBOOT = 0x4;
}
"@

$sidecarPath = Join-Path $PSScriptRoot "..\src-tauri\binaries\cli-proxy-api-x86_64-pc-windows-msvc.exe"
$sidecarPath = Resolve-Path $sidecarPath -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Path

if (-not $sidecarPath) {
    # Search for any sidecar exe
    $searchPath = Join-Path $PSScriptRoot "..\src-tauri\binaries"
    $found = Get-ChildItem $searchPath -Filter "*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($found) {
        $sidecarPath = $found.FullName
    } else {
        Write-Error "No sidecar executable found"
        exit 1
    }
}

function Write-Header($text) {
    Write-Host "`n=== $text ===" -ForegroundColor Cyan
}

function Write-Success($text) {
    Write-Host "✓ $text" -ForegroundColor Green
}

function Write-Error($text) {
    Write-Host "✗ $text" -ForegroundColor Red
}

function Test-FileLock($path) {
    try {
        $stream = [System.IO.File]::Open($path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::None)
        $stream.Close()
        $stream.Dispose()
        return $false
    } catch {
        return $true
    }
}

Write-Header "ALTERNATIVE UNLOCK METHOD"
Write-Host "Target: $sidecarPath"
Write-Host "Method: $Method"

$methods = if ($Method -eq "All") { @("Rename", "Move", "DelayedDelete", "Robocopy") } else { @($Method) }

foreach ($m in $methods) {
    Write-Header "ATTEMPTING: $m"
    
    switch ($m) {
        "Rename" {
            # Method: Rename to .old then delete
            try {
                $dir = Split-Path $sidecarPath -Parent
                $name = [System.IO.Path]::GetFileNameWithoutExtension($sidecarPath)
                $newName = "$name-$(Get-Random).old"
                $newPath = Join-Path $dir $newName
                
                Rename-Item $sidecarPath $newName -Force
                Write-Success "Renamed to: $newName"
                
                try {
                    Remove-Item $newPath -Force
                    Write-Success "Deleted renamed file"
                } catch {
                    Write-Error "Could not delete renamed file (will try on reboot): $_"
                    # Schedule for deletion on reboot
                    [NativeMethods]::MoveFileEx($newPath, $null, [NativeMethods]::MOVEFILE_DELAY_UNTIL_REBOOT) | Out-Null
                }
            } catch {
                Write-Error "Rename method failed: $_"
            }
        }
        
        "Move" {
            # Method: Move to temp folder then delete
            try {
                $tempDir = "$env:TEMP\sidecar_delete_$(Get-Random)"
                New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
                
                $fileName = Split-Path $sidecarPath -Leaf
                $tempPath = Join-Path $tempDir $fileName
                
                Move-Item $sidecarPath $tempPath -Force
                Write-Success "Moved to temp: $tempPath"
                
                try {
                    Remove-Item $tempPath -Force
                    Remove-Item $tempDir -Force
                    Write-Success "Deleted from temp"
                } catch {
                    Write-Error "Could not delete from temp (may succeed later): $_"
                }
            } catch {
                Write-Error "Move method failed: $_"
            }
        }
        
        "DelayedDelete" {
            # Method: Schedule deletion on next reboot using MoveFileEx
            try {
                $result = [NativeMethods]::MoveFileEx($sidecarPath, $null, [NativeMethods]::MOVEFILE_DELAY_UNTIL_REBOOT)
                if ($result) {
                    Write-Success "Scheduled for deletion on NEXT REBOOT"
                    Write-Host "⚠️  File will be deleted when you restart Windows"
                } else {
                    $err = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
                    Write-Error "MoveFileEx failed with error: $err"
                }
            } catch {
                Write-Error "Delayed delete method failed: $_"
            }
        }
        
        "Robocopy" {
            # Method: Mirror empty folder to delete contents
            try {
                $targetDir = Split-Path $sidecarPath -Parent
                $emptyDir = "$env:TEMP\empty_mirror_$(Get-Random)"
                New-Item -ItemType Directory -Path $emptyDir -Force | Out-Null
                
                # First try to rename the file to something random
                $fileName = Split-Path $sidecarPath -Leaf
                $renamedFile = "to_delete_$(Get-Random).tmp"
                $renamedPath = Join-Path $targetDir $renamedFile
                
                try {
                    Rename-Item $sidecarPath $renamedFile -Force
                    Write-Success "Renamed file to $renamedFile"
                } catch {
                    Write-Warn "Could not rename, trying direct robocopy..."
                }
                
                # Use robocopy to mirror empty folder
                Write-Host "Running robocopy mirror..."
                & robocopy $emptyDir $targetDir /MIR /R:2 /W:1 /NP /NFL /NDL 2>&1 | Out-Null
                
                Remove-Item $emptyDir -Force -Recurse -ErrorAction SilentlyContinue
                Write-Success "Robocopy mirror completed"
            } catch {
                Write-Error "Robocopy method failed: $_"
            }
        }
    }
    
    # Check if file is unlocked
    if (-not (Test-FileLock $sidecarPath)) {
        Write-Success "FILE IS NOW UNLOCKED!"
        break
    }
}

Write-Header "FINAL STATUS"
if (Test-Path $sidecarPath) {
    if (Test-FileLock $sidecarPath) {
        Write-Error "File is STILL LOCKED"
        Write-Host "`n💡 Recommendations:"
        Write-Host "   1. Try Method 'DelayedDelete' and restart Windows"
        Write-Host "   2. Use Sysinternals Process Explorer to find handle"
        Write-Host "   3. Restart computer"
    } else {
        Write-Success "File exists but is UNLOCKED - you can now build!"
    }
} else {
    Write-Success "File has been DELETED successfully!"
}
