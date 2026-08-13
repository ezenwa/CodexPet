param([string]$Version = (Get-Content -LiteralPath (Join-Path $PSScriptRoot 'VERSION') -Raw).Trim())

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$issPath = Join-Path $root 'installer\CodexPet.iss'
$distDir = Join-Path $root 'dist'
$setupPath = Join-Path $distDir 'CodexPet-Setup.exe'
$setupZipPath = Join-Path $distDir 'CodexPet-Setup.zip'
$isccCandidates = @(
    (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe'),
    (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'),
    (Join-Path $env:ProgramFiles 'Inno Setup 6\ISCC.exe')
)
$iscc = $isccCandidates | Where-Object { $_ -and (Test-Path -LiteralPath $_) } | Select-Object -First 1
if (-not $iscc) { throw 'No se encontró ISCC.exe. Instala Inno Setup 6 antes de compilar.' }

foreach ($scriptPath in @(Get-ChildItem -LiteralPath $root -Filter '*.ps1' -File)) {
    $tokens = $null
    $errors = $null
    [Management.Automation.Language.Parser]::ParseFile($scriptPath.FullName, [ref]$tokens, [ref]$errors) | Out-Null
    if ($errors.Count) { throw "$($scriptPath.Name) contiene errores de sintaxis." }
}

$stateTests = Join-Path $root 'tests\CodexPet-State.Tests.ps1'
& powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $stateTests
if ($LASTEXITCODE -ne 0) { throw 'Fallaron las pruebas de estados de CodexPet.' }

New-Item -ItemType Directory -Path $distDir -Force | Out-Null
foreach ($oldArtifact in @($setupPath, $setupZipPath)) {
    if (Test-Path -LiteralPath $oldArtifact) { Remove-Item -LiteralPath $oldArtifact -Force }
}

& $iscc "/DMyAppVersion=$Version" $issPath
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $setupPath)) {
    throw "Inno Setup no pudo generar $setupPath (código $LASTEXITCODE)."
}

Compress-Archive -LiteralPath $setupPath -DestinationPath $setupZipPath -CompressionLevel Optimal -Force
Get-Item -LiteralPath $setupPath, $setupZipPath | Select-Object FullName, Length, LastWriteTime
