@echo off
setlocal
set "VERSION=%~1"
if not defined VERSION set "VERSION=1.1.0"

powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0Build-Release.ps1" -Version "%VERSION%"
if errorlevel 1 exit /b %errorlevel%

"%SystemRoot%\System32\iexpress.exe" /N "%~dp0.build\CodexPet-Setup.sed"
if errorlevel 1 exit /b %errorlevel%

powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -Command "Compress-Archive -LiteralPath '%~dp0dist\CodexPet-Setup.exe' -DestinationPath '%~dp0dist\CodexPet-Setup.zip' -CompressionLevel Optimal -Force"
if errorlevel 1 exit /b %errorlevel%

echo CodexPet %VERSION% listo en dist\CodexPet-Setup.exe y dist\CodexPet-Setup.zip
