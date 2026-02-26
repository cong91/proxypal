# Sidecar PermissionDenied Fix - Complete Guide

## Problem
Windows locks the sidecar binary (`cli-proxy-api-x86_64-pc-windows-msvc.exe`) causing `PermissionDenied` errors during Tauri builds.

Common culprits:
- Windows Defender/Antivirus real-time scanning
- Windows Search indexer
- Windows Explorer holding handles
- Zombie processes from previous builds

---

## Quick Fix (Recommended First)

```powershell
# Run as Administrator in PowerShell
cd scripts
.\fix-sidecar-master.ps1
```

This runs the complete fix workflow automatically.

---

## Manual Methods (If Quick Fix Fails)

### Method 1: Advanced Process Kill
```powershell
.\unlock-sidecar-advanced.ps1 -KillProcesses
```

Uses `handle.exe` from Sysinternals to find and release file locks.

**Download handle.exe:**
```powershell
# Download from Microsoft
Invoke-WebRequest -Uri "https://download.sysinternals.com/files/Handle.zip" -OutFile "$env:TEMP\Handle.zip"
Expand-Archive "$env:TEMP\Handle.zip" -DestinationPath "$env:USERPROFILE\Downloads\Handle"
Copy-Item "$env:USERPROFILE\Downloads\Handle\handle.exe" "C:\Windows\System32\handle.exe"
```

### Method 2: Aggressive Cleanup
```powershell
# Standard cleanup
.\aggressive-cleanup.ps1

# Nuclear mode (kills all processes + deletes everything)
.\aggressive-cleanup.ps1 -Nuke
```

### Method 3: Alternative Unlock Workarounds
```powershell
# Try rename-then-delete
.\alternative-unlock.ps1 -Method Rename

# Move to temp folder then delete
.\alternative-unlock.ps1 -Method Move

# Schedule deletion on reboot
.\alternative-unlock.ps1 -Method DelayedDelete

# Try all methods
.\alternative-unlock.ps1 -Method All
```

### Method 4: Windows Defender Workarounds
```powershell
# Check current status
.\defender-workaround.ps1 -Action Status

# Add exclusions (recommended during development)
.\defender-workaround.ps1 -Action AddExclusion

# Temporarily disable real-time protection (not recommended long-term)
.\defender-workaround.ps1 -Action DisableRealtime

# Re-enable when done
.\defender-workaround.ps1 -Action EnableRealtime

# Remove exclusions
.\defender-workaround.ps1 -Action RemoveExclusion
```

---

## Nuclear Option (Last Resort)

```powershell
# Maximum aggression - kills services, restarts explorer
.\fix-sidecar-master.ps1 -Nuclear
```

---

## Step-by-Step Manual Fix

If scripts don't work, do this manually:

### Step 1: Kill all related processes
```powershell
# Run as Administrator
Get-Process | Where-Object { $_.ProcessName -match "cli-proxy-api|proxypal|tauri|cargo|rustc" } | Stop-Process -Force
```

### Step 2: Check what is locking the file
1. Open **Resource Monitor** (Win+R → `resmon.exe`)
2. Go to **CPU** tab
3. In **Associated Handles** search box, type: `cli-proxy-api`
4. Right-click on results → **End Process**

### Step 3: Delete the locked file

**Option A: Command line**
```powershell
# Navigate to binaries folder
cd src-tauri\binaries

# Try rename first
ren cli-proxy-api-x86_64-pc-windows-msvc.exe old.exe

# Then delete
del old.exe
```

**Option B: Safe Mode**
1. Restart Windows in Safe Mode
2. Delete the file manually
3. Restart normally

**Option C: Use LockHunter**
1. Download LockHunter from https://lockhunter.com/
2. Right-click the locked file → "What is locking this file?"
3. Select "Delete It!"

### Step 4: Clean build artifacts
```powershell
cd src-tauri
cargo clean
cd ..
Remove-Item -Recurse -Force .\src-tauri\target -ErrorAction SilentlyContinue
```

### Step 5: Rebuild
```powershell
pnpm tauri dev
# or
pnpm tauri build
```

---

## Prevent Future Locks

### 1. Add Windows Defender Exclusion
```powershell
Add-MpPreference -ExclusionPath "C:\path\to\proxypal\src-tauri\target"
Add-MpPreference -ExclusionPath "C:\path\to\proxypal\src-tauri\binaries"
```

### 2. Stop Windows Search from indexing
1. Settings → Search → Searching Windows
2. Add project folder to "Excluded Folders"

### 3. Configure IDE
If using VS Code:
- Add `src-tauri/target` to files.exclude
- Disable real-time problem checking for Rust if not needed

---

## Troubleshooting

### "Access Denied" when running scripts
- Run PowerShell as **Administrator**
- Run: `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser`

### handle.exe not found
Download from: https://docs.microsoft.com/sysinternals/downloads/handle

### File still locked after restart
- Check if the file is synced by OneDrive/Dropbox - pause sync
- Check Antivirus quarantine
- Boot into Safe Mode to delete

### Build still fails after file deleted
```powershell
# Force redownload of sidecar binary
cd src-tauri
.\scripts\download-binaries.ps1
```

---

## Script Reference

| Script | Purpose | When to use |
|--------|---------|-------------|
| `fix-sidecar-master.ps1` | Complete automated fix | First try |
| `unlock-sidecar-advanced.ps1` | Process killing with handle.exe | When file is locked by process |
| `aggressive-cleanup.ps1` | Delete all build artifacts | Corrupted build state |
| `alternative-unlock.ps1` | Creative unlock methods | Standard delete fails |
| `defender-workaround.ps1` | Manage AV exclusions | AV blocking files |

---

## Emergency: Immediate Build Bypass

If you need to build NOW and can't fix the lock:

```powershell
# Temporarily rename the binary
cd src-tauri\binaries
ren cli-proxy-api-x86_64-pc-windows-msvc.exe cli-proxy-api-x86_64-pc-windows-msvc.exe.bak

# Build (sidecar won't be included)
pnpm tauri build

# After build, restore the binary
ren cli-proxy-api-x86_64-pc-windows-msvc.exe.bak cli-proxy-api-x86_64-pc-windows-msvc.exe
```

**Note:** This builds without the sidecar. Features requiring the sidecar won't work.

---

## Contact

If none of these methods work, the issue may be:
- Group policy restricting execution
- Corporate security software
- Hardware-level security

Consider using WSL2 for development or contact IT support.
