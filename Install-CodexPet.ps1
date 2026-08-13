param([string]$PayloadPath = (Join-Path $PSScriptRoot 'CodexPet-Payload.zip'))
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName PresentationFramework
$installDir = Join-Path $env:LOCALAPPDATA 'CodexPet'
$programsDir = [Environment]::GetFolderPath('Programs')
$desktopDir = [Environment]::GetFolderPath('Desktop')
$powerShellExe = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
$wscriptExe = Join-Path $env:SystemRoot 'System32\wscript.exe'
$startupShortcut = Join-Path ([Environment]::GetFolderPath('Startup')) 'CodexPet.lnk'
$legacyStartupShortcut = Join-Path ([Environment]::GetFolderPath('Startup')) 'Codex Pet.lnk'

if (Test-Path -LiteralPath (Join-Path $installDir 'codexpet.pid')) {
    $oldPid = [int](Get-Content -LiteralPath (Join-Path $installDir 'codexpet.pid') -Raw)
    Stop-Process -Id $oldPid -Force -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Path $installDir -Force | Out-Null
Expand-Archive -LiteralPath $PayloadPath -DestinationPath $installDir -Force

$shell = New-Object -ComObject WScript.Shell
foreach ($shortcutPath in @((Join-Path $desktopDir 'CodexPet.lnk'), (Join-Path $programsDir 'CodexPet.lnk'))) {
    $shortcut = $shell.CreateShortcut($shortcutPath)
    $shortcut.TargetPath = $powerShellExe
    $shortcut.Arguments = "-NoLogo -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$(Join-Path $installDir 'CodexPet.ps1')`" -Startup"
    $shortcut.WorkingDirectory = $installDir
    $shortcut.Save()
}
$uninstallShortcut = $shell.CreateShortcut((Join-Path $programsDir 'Desinstalar CodexPet.lnk'))
$uninstallShortcut.TargetPath = $powerShellExe
$uninstallShortcut.Arguments = "-NoLogo -NoProfile -ExecutionPolicy Bypass -File `"$(Join-Path $installDir 'Uninstall-CodexPet.ps1')`""
$uninstallShortcut.WorkingDirectory = $installDir
$uninstallShortcut.Save()

foreach ($obsoleteShortcut in @($startupShortcut, $legacyStartupShortcut, (Join-Path $desktopDir 'Codex Pet.lnk'), (Join-Path $programsDir 'Codex Pet.lnk'), (Join-Path $programsDir 'Desinstalar Codex Pet.lnk'))) {
    if (Test-Path -LiteralPath $obsoleteShortcut) { Remove-Item -LiteralPath $obsoleteShortcut -Force }
}

$runKeyPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$watcherLauncher = Join-Path $installDir 'Start-CodexPetWatcher.vbs'
$watcherCommand = "`"$wscriptExe`" //B //Nologo `"$watcherLauncher`""
New-Item -Path $runKeyPath -Force | Out-Null
Set-ItemProperty -Path $runKeyPath -Name 'CodexPetWatcher' -Value $watcherCommand

Start-Process -FilePath $wscriptExe -ArgumentList @('//B','//Nologo',$watcherLauncher) -WorkingDirectory $installDir -WindowStyle Hidden
[System.Windows.MessageBox]::Show('CodexPet se instaló correctamente.','CodexPet') | Out-Null
