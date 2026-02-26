@echo off
chcp 65001 >nul
title Tauri Process Killer - Emergency Fix
echo.
echo ╔══════════════════════════════════════════════════════════════╗
echo ║     TAURI PROCESS KILLER - Fix PermissionDenied Error 5      ║
echo ╚══════════════════════════════════════════════════════════════╝
echo.

echo [STEP 1] Scanning for Tauri-related processes...
echo.

echo   Checking for: cargo.exe, tauri-app.exe, proxypal.exe, node.exe
echo.

REM List processes before killing
tasklist /FI "IMAGENAME eq cargo.exe" 2>nul | find /I "cargo.exe" >nul && echo   [FOUND] cargo.exe
tasklist /FI "IMAGENAME eq tauri-app.exe" 2>nul | find /I "tauri-app.exe" >nul && echo   [FOUND] tauri-app.exe
tasklist /FI "IMAGENAME eq proxypal.exe" 2>nul | find /I "proxypal.exe" >nul && echo   [FOUND] proxypal.exe
tasklist /FI "IMAGENAME eq node.exe" 2>nul | find /I "node.exe" >nul && echo   [FOUND] node.exe
tasklist /FI "IMAGENAME eq vite.exe" 2>nul | find /I "vite.exe" >nul && echo   [FOUND] vite.exe

echo.
echo [STEP 2] Terminating processes with force kill...
echo.

REM Kill each process with /F (force) and /T (terminate tree)
echo   Killing tauri-app.exe...
taskkill /F /IM tauri-app.exe /T 2>nul && echo     OK || echo     Not running or access denied

echo   Killing proxypal.exe...
taskkill /F /IM proxypal.exe /T 2>nul && echo     OK || echo     Not running or access denied

echo   Killing cargo.exe...
taskkill /F /IM cargo.exe /T 2>nul && echo     OK || echo     Not running or access denied

echo   Killing node.exe...
taskkill /F /IM node.exe /T 2>nul && echo     OK || echo     Not running or access denied

echo   Killing rust-analyzer.exe...
taskkill /F /IM rust-analyzer.exe /T 2>nul && echo     OK || echo     Not running or access denied

echo.
echo [STEP 3] Waiting for processes to fully terminate...
timeout /T 2 /NOBREAK >nul
echo     Done
echo.

REM Optional: Clean target folder
if "%1"=="--clean" (
    echo [STEP 4] Cleaning target folders...
    if exist "src-tauri\target\debug" (
        echo   Removing src-tauri\target\debug...
        rmdir /S /Q "src-tauri\target\debug" 2>nul && echo     OK || echo     FAILED (may be locked)
    )
    if exist "src-tauri\target\release" (
        echo   Removing src-tauri\target\release...
        rmdir /S /Q "src-tauri\target\release" 2>nul && echo     OK || echo     FAILED (may be locked)
    )
    echo.
)

echo ╔══════════════════════════════════════════════════════════════╗
echo ║                     FIX COMPLETED                            ║
echo ╠══════════════════════════════════════════════════════════════╣
echo ║                                                              ║
echo ║  Next steps:                                                 ║
echo ║                                                              ║
echo ║  1. Close any VS Code terminal tabs                          ║
echo ║  2. Run: pnpm tauri dev                                      ║
echo ║                                                              ║
echo ║  Usage: kill-tauri-processes.bat --clean  (to also clean)    ║
echo ║                                                              ║
echo ╚══════════════════════════════════════════════════════════════╝
echo.

if "%1"=="--restart" (
    echo [AUTO-RESTART] Starting pnpm tauri dev in 3 seconds...
    timeout /T 3 /NOBREAK >nul
    pnpm tauri dev
)

pause
