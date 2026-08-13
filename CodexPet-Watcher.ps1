param([string]$InstallDir = $PSScriptRoot)

$ErrorActionPreference = 'Stop'
$logFile = Join-Path $InstallDir 'watcher.log'
function Write-WatcherLog([string]$Message) {
    "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss.fff') $Message" |
        Add-Content -LiteralPath $logFile -Encoding utf8
}

function Test-PetRunning {
    $pidFile = Join-Path $InstallDir 'codexpet.pid'
    if (-not (Test-Path -LiteralPath $pidFile)) { return $false }

    $petProcessId = 0
    if (-not [int]::TryParse((Get-Content -LiteralPath $pidFile -Raw -ErrorAction SilentlyContinue).Trim(), [ref]$petProcessId)) {
        return $false
    }
    return $null -ne (Get-Process -Id $petProcessId -ErrorAction SilentlyContinue)
}

$createdNew = $false
$watcherMutex = $null

try {
    Write-WatcherLog "Watcher starting (PID=$PID)"
    # V2 avoids a stale mutex left by older watcher builds.
    $watcherMutex = New-Object System.Threading.Mutex($true, 'Local\CodexPet.Watcher.V2', [ref]$createdNew)
    if (-not $createdNew) {
        Write-WatcherLog 'Another watcher is already running; exiting.'
        exit 0
    }

    $systemPowerShell = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
    $powerShellExe = if (Test-Path -LiteralPath $systemPowerShell) {
        $systemPowerShell
    } else {
        (Get-Command powershell.exe -ErrorAction Stop).Source
    }
    $petScript = Join-Path $InstallDir 'CodexPet.ps1'
    while ($true) {
        # Codex can create short-lived codex.exe helper processes while a task is
        # running. Treat the process group collectively: transferring ownership
        # to every new PID makes the pet close and reopen when a helper exits.
        $codexProcesses = @(Get-Process -Name codex -ErrorAction SilentlyContinue)
        if ($codexProcesses.Count -gt 0 -and
            -not (Test-PetRunning) -and
            (Test-Path -LiteralPath $petScript)) {
            Write-WatcherLog "Codex active; starting pet."
            Start-Process -FilePath $powerShellExe `
                -ArgumentList @('-NoLogo','-NoProfile','-WindowStyle','Hidden','-ExecutionPolicy','Bypass','-File',$petScript,'-CloseWithCodex') `
                -WorkingDirectory $InstallDir -WindowStyle Hidden
        }
        Start-Sleep -Milliseconds 750
    }
} catch {
    Write-WatcherLog "ERROR: $($_.Exception.Message)"
    exit 1
} finally {
    if ($watcherMutex) {
        if ($createdNew) { $watcherMutex.ReleaseMutex() }
        $watcherMutex.Dispose()
    }
}
