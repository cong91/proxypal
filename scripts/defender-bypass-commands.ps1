# Windows Defender Bypass - One-liner Commands
# Copy-paste từng dòng vào PowerShell (Run as Administrator)
# Không chạy file này trực tiếp để tránh AV detection

# ============================================
# 1️⃣ TẮT WINDOWS DEFENDER REAL-TIME (Tạm thờ)
# ============================================
Set-MpPreference -DisableRealtimeMonitoring $true; Write-Host "Defender Real-time: OFF" -ForegroundColor Green

# ⚠️ Để bật lại sau khi xong:
# Set-MpPreference -DisableRealtimeMonitoring $false

# ============================================
# 2️⃣ KILL PROCESS BỊ LOCK
# ============================================
# Kill theo tên process
Get-Process | Where-Object { $_.ProcessName -match "proxypal|tauri|sidecar|node" } | Stop-Process -Force -ErrorAction SilentlyContinue; Write-Host "Processes killed" -ForegroundColor Green

# Hoặc kill theo port 3030
Get-NetTCPConnection -LocalPort 3030 -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess | ForEach-Object { Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue }; Write-Host "Port 3030 processes killed" -ForegroundColor Green

# ============================================
# 3️⃣ XÓA FILE SIDECAR
# ============================================
$sidecarPath = "src-tauri\binaries\cli-proxy-api-management-x86_64-pc-windows-msvc.exe"; if (Test-Path $sidecarPath) { Remove-Item $sidecarPath -Force -ErrorAction SilentlyContinue; Write-Host "Sidecar deleted" -ForegroundColor Green } else { Write-Host "Sidecar not found" -ForegroundColor Yellow }

# ============================================
# 4️⃣ TẢI LẠI SIDECAR
# ============================================
# Cách 1: Dùng script có sẵn
powershell -ExecutionPolicy Bypass -Command "Set-Location src-tauri; .\scripts\download-binaries.ps1"

# Cách 2: Tải trực tiếp bằng curl/Invoke-WebRequest
$outFile = "src-tauri\binaries\cli-proxy-api-management-x86_64-pc-windows-msvc.exe"; $url = "https://github.com/thanhnguyen2187/proxypal/releases/latest/download/cli-proxy-api-management-x86_64-pc-windows-msvc.exe"; New-Item -ItemType Directory -Force -Path (Split-Path $outFile) | Out-Null; Invoke-WebRequest -Uri $url -OutFile $outFile -UseBasicParsing; Write-Host "Sidecar downloaded" -ForegroundColor Green

# ============================================
# 5️⃣ CHẠY DEV
# ============================================
pnpm tauri dev

# ============================================
# 🔓 BYPASS EXECUTION POLICY (Nếu cần)
# ============================================
# Cách 1: Mở PowerShell với Bypass
powershell -ExecutionPolicy Bypass

# Cách 2: Set policy permanently (chạy 1 lần)
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force

# ============================================
# 🛡️ BẬT LẠI DEFENDER (Sau khi xong)
# ============================================
Set-MpPreference -DisableRealtimeMonitoring $false; Write-Host "Defender Real-time: ON" -ForegroundColor Green

# ============================================
# 🚀 FULL SEQUENCE (Copy từng dòng)
# ============================================
<#
Set-MpPreference -DisableRealtimeMonitoring $true
Get-Process | Where-Object { $_.ProcessName -match "proxypal|tauri|sidecar|node" } | Stop-Process -Force -ErrorAction SilentlyContinue
$sidecarPath = "src-tauri\binaries\cli-proxy-api-management-x86_64-pc-windows-msvc.exe"; if (Test-Path $sidecarPath) { Remove-Item $sidecarPath -Force }
cd src-tauri; powershell -ExecutionPolicy Bypass -Command ".\scripts\download-binaries.ps1"; cd ..
pnpm tauri dev
#>
