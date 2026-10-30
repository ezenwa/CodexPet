# Upgrade helper only. Normal operation uses CodexPet.exe --stop.
param([Parameter(Mandatory = $true)][string]$InstallDir)
$ErrorActionPreference = 'Stop'
$installPath = [IO.Path]::GetFullPath($InstallDir)
$petScript = Join-Path $installPath 'CodexPet.ps1'
$watcherScript = Join-Path $installPath 'CodexPet-Watcher.ps1'
# Verify both the interpreter and script path. Never terminate an unrelated reused PID.
$legacy = @(Get-CimInstance Win32_Process | Where-Object {
    $_.ProcessId -ne $PID -and $_.Name -in @('powershell.exe','pwsh.exe') -and
    ($_.CommandLine -match [regex]::Escape($petScript) -or $_.CommandLine -match [regex]::Escape($watcherScript))
})
foreach ($process in $legacy) { Stop-Process -Id $process.ProcessId -Force -ErrorAction SilentlyContinue }
$native = Join-Path $installPath 'CodexPet.exe'
if (Test-Path -LiteralPath $native) {
    $stopper = Start-Process -FilePath $native -ArgumentList '--stop' -WindowStyle Hidden -PassThru
    if (-not $stopper.WaitForExit(10000)) { throw 'CodexPet stop command timed out.' }
    Start-Sleep -Milliseconds 1000
}
