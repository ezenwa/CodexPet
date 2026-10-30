param([string]$Executable = (Join-Path (Split-Path $PSScriptRoot -Parent) 'target\release\CodexPet.exe'))
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$tempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\')
$workDir = Join-Path $tempRoot ('CodexPet-verify-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $workDir | Out-Null
function Invoke-NativeCheck([string[]]$NativeArguments) {
    $process = Start-Process -FilePath $Executable -ArgumentList $NativeArguments -WindowStyle Hidden -PassThru
    if (-not $process.WaitForExit(15000)) { $process.Kill(); throw 'Native check timed out.' }
    if ($process.ExitCode -ne 0) { throw "Native check failed with code $($process.ExitCode)." }
}
try {
    $marker = Join-Path $workDir 'smoke.ok'
    Invoke-NativeCheck @('--smoke-test', ('"' + $marker + '"'))
    if (-not (Test-Path -LiteralPath $marker)) { throw 'Native smoke test did not write its completion marker.' }
    Get-Content -LiteralPath $marker
    $previews = Join-Path $workDir 'preview'
    Invoke-NativeCheck @('--preview', ('"' + $previews + '"'))
    if (@(Get-ChildItem -LiteralPath $previews -Filter '*.png' -File).Count -ne 6) { throw 'Expected six state previews.' }
    'Six preview exports verified; temporary output will be removed.'
} finally {
    $resolved = [IO.Path]::GetFullPath($workDir)
    if (-not $resolved.StartsWith($tempRoot + '\CodexPet-verify-', [StringComparison]::OrdinalIgnoreCase)) { throw 'Unexpected verification cleanup path.' }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
