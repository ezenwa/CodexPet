$ErrorActionPreference = 'SilentlyContinue'
$installDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$pidFile = Join-Path $installDir 'codexpet.pid'

if (Test-Path -LiteralPath $pidFile) {
    $petProcessId = 0
    if ([int]::TryParse((Get-Content -LiteralPath $pidFile -Raw).Trim(), [ref]$petProcessId) -and $petProcessId -gt 0) {
        Stop-Process -Id $petProcessId -Force
    }
}

Get-CimInstance Win32_Process |
    Where-Object {
        $_.ProcessId -ne $PID -and
        $_.Name -in @('powershell.exe', 'pwsh.exe') -and
        $_.CommandLine -match [regex]::Escape((Join-Path $installDir 'CodexPet-Watcher.ps1'))
    } |
    ForEach-Object { Stop-Process -Id $_.ProcessId -Force }

Remove-Item -LiteralPath $pidFile -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath (Join-Path $installDir 'close-with-codex.request') -Force -ErrorAction SilentlyContinue
