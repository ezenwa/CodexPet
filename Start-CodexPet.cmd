@echo off
where pwsh.exe >nul 2>nul && (set "PSHOST=pwsh.exe") || (set "PSHOST=powershell.exe")
start "CodexPet" /min %PSHOST% -NoLogo -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File "%~dp0CodexPet.ps1" -CloseWithCodex
