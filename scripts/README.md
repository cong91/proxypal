# ProxyPal Development Scripts

## PermissionDenied Error 5 - Quick Fix

### Emergency Fix (When you see "Access is denied")

```powershell
# PowerShell (Recommended)
.\scripts\kill-tauri-processes.ps1

# Or with cleanup and auto-restart
.\scripts\kill-tauri-processes.ps1 -CleanTarget -RestartDev

# Batch (if PowerShell not available)
.\scripts\kill-tauri-processes.bat

# With cleanup
.\scripts\kill-tauri-processes.bat --clean
```

### Then restart development:
```bash
pnpm tauri dev
```

---

## Scripts Reference

### `kill-tauri-processes.ps1`
Forcefully terminates all Tauri-related processes that may lock files.

**Processes killed:**
- `cargo.exe` - Rust build system
- `tauri-app.exe` - Tauri dev executable
- `proxypal.exe` - Sidecar CLI proxy
- `node.exe` - Vite dev server
- `rust-analyzer.exe` - Rust LSP (optional with `-IncludeRustAnalyzer`)

**Parameters:**
```powershell
-IncludeRustAnalyzer    # Also kill rust-analyzer
-CleanTarget            # Remove target/debug and target/release
-RestartDev             # Auto-restart pnpm tauri dev after cleanup
```

### `kill-tauri-processes.bat`
Batch version for systems without PowerShell.

```batch
# Basic kill
scripts\kill-tauri-processes.bat

# With cleanup
scripts\kill-tauri-processes.bat --clean

# With auto-restart
scripts\kill-tauri-processes.bat --restart
```

### `setup-defender-exclusion.ps1`
Sets up Windows Defender exclusions to prevent file locking during builds.

**Run as Administrator:**
```powershell
# Preview changes
.\scripts\setup-defender-exclusion.ps1 -DryRun

# Apply exclusions
.\scripts\setup-defender-exclusion.ps1

# Skip confirmation
.\scripts\setup-defender-exclusion.ps1 -Force
```

**Excluded paths:**
- `src-tauri/target/` - Build artifacts
- `node_modules/` - Dependencies
- `~/.cargo/registry/` - Cargo cache

### `pre-dev.ps1`
Pre-development hook to prevent PermissionDenied errors.

Add to `package.json`:
```json
{
  "scripts": {
    "predev": "powershell -ExecutionPolicy Bypass -File scripts/pre-dev.ps1",
    "dev": "pnpm predev && pnpm tauri dev"
  }
}
```

---

## Root Cause Analysis

### Why does PermissionDenied Error 5 happen?

1. **Windows File Locking**
   - Windows keeps file handles open on running executables
   - `tauri-app.exe` from previous `pnpm tauri dev` is still running
   - Cargo cannot overwrite the locked executable

2. **Sidecar Process (`proxypal.exe`)**
   - CLI proxy sidecar runs as separate process
   - May not terminate properly when dev server stops
   - Keeps handle to logs/config files

3. **Windows Defender Real-time Scanning**
   - Scans newly built executables
   - Temporarily locks files during scan
   - Race condition with cargo writing files

4. **Proxy Provider State**
   - Recent changes to `ProxySettings`/`ProviderSelector`
   - Rotation provider may spawn background tasks
   - `ttl_monitor_running` flag not properly cleaned up

---

## Prevention Best Practices

### 1. Always stop gracefully
```bash
# Use Ctrl+C in terminal
# Wait for "Process finished" message
# Don't close terminal window directly
```

### 2. Use pre-dev hook
Add to your workflow:
```bash
pnpm predev  # Auto-cleanup before dev
pnpm tauri dev
```

### 3. Setup Defender exclusions (one-time)
```powershell
.\scripts\setup-defender-exclusion.ps1 -Force
```

### 4. VS Code settings
Add to `.vscode/settings.json`:
```json
{
  "rust-analyzer.checkOnSave": false,
  "rust-analyzer.cargo.buildScripts.enable": false
}
```

### 5. Before builds
Always run cleanup before production builds:
```bash
.\scripts\kill-tauri-processes.ps1 -CleanTarget
pnpm tauri build
```

---

## Troubleshooting

### Script says "Access denied"
Run PowerShell as Administrator:
1. Press Win+X
2. Select "Terminal (Admin)" or "PowerShell (Admin)"
3. Navigate to project: `cd d:\Work\Outsourcing\proxypal`
4. Run script: `.\scripts\kill-tauri-processes.ps1`

### Still getting PermissionDenied after script
1. Close all VS Code windows
2. Open Task Manager (Ctrl+Shift+Esc)
3. End any remaining: `tauri-app.exe`, `cargo.exe`, `proxypal.exe`
4. Delete `src-tauri/target/debug` folder manually
5. Restart VS Code

### File in use by another process
Use Sysinternals Handle:
```powershell
# Download from https://docs.microsoft.com/sysinternals/downloads/handle
handle.exe tauri-app.exe
handle.exe -p tauri-app.exe
```

---

## Quick Command Reference

```bash
# One-liner emergency kill
taskkill /F /IM tauri-app.exe /IM cargo.exe /IM proxypal.exe /IM node.exe 2>nul

# Check what's using the file (requires handle.exe)
handle.exe src-tauri\target\debug\tauri-app.exe

# Nuclear option - kill all related processes
taskkill /F /FI "IMAGENAME eq tauri-app.exe"
taskkill /F /FI "IMAGENAME eq cargo.exe"
taskkill /F /FI "IMAGENAME eq proxypal.exe"
```
