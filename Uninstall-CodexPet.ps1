$installDir = $PSScriptRoot
$powerShellExe = if (Get-Command pwsh.exe -ErrorAction SilentlyContinue) { (Get-Command pwsh.exe).Source } else { (Get-Command powershell.exe).Source }
$pidPath = Join-Path $installDir 'codexpet.pid'
if (Test-Path -LiteralPath $pidPath) {
    $petPid = [int](Get-Content -LiteralPath $pidPath -Raw)
    Stop-Process -Id $petPid -Force -ErrorAction SilentlyContinue
}
Get-CimInstance Win32_Process -Filter "Name = 'powershell.exe' OR Name = 'pwsh.exe'" -ErrorAction SilentlyContinue |
    Where-Object { $_.CommandLine -like '*CodexPet-Watcher.ps1*' } |
    ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
$startupShortcut = Join-Path ([Environment]::GetFolderPath('Startup')) 'Codex Pet.lnk'
$programsDir = [Environment]::GetFolderPath('Programs')
$desktopDir = [Environment]::GetFolderPath('Desktop')
foreach ($path in @($startupShortcut, (Join-Path $programsDir 'Codex Pet.lnk'), (Join-Path $programsDir 'Desinstalar Codex Pet.lnk'), (Join-Path $desktopDir 'Codex Pet.lnk'))) {
    Remove-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue
}
Remove-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'CodexPetWatcher' -ErrorAction SilentlyContinue
$cleanup = "Start-Sleep -Milliseconds 800; Remove-Item -LiteralPath '$($installDir.Replace("'","''"))' -Recurse -Force"
Start-Process -FilePath $powerShellExe -ArgumentList @('-NoLogo','-NoProfile','-WindowStyle','Hidden','-Command',$cleanup) -WorkingDirectory $env:TEMP -WindowStyle Hidden
Add-Type -AssemblyName PresentationFramework
[System.Windows.MessageBox]::Show('Codex Pet se desinstaló correctamente.','Codex Pet') | Out-Null
