@echo off
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0Install-CodexPet.ps1" -PayloadPath "%~dp0CodexPet-Payload.zip"
