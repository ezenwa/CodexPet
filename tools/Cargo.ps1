$ErrorActionPreference = 'Stop'
[string[]]$cargoArguments = @($args)
if (-not $cargoArguments.Count) { $cargoArguments = @('build') }
$root = Split-Path $PSScriptRoot -Parent
$localCargo = Join-Path $root '.build\cargo\bin\cargo.exe'
if (Test-Path -LiteralPath $localCargo) {
    $env:CARGO_HOME = Join-Path $root '.build\cargo'
    $env:RUSTUP_HOME = Join-Path $root '.build\rustup'
    $env:PATH = (Join-Path $env:CARGO_HOME 'bin') + ';' + $env:PATH
    $cargoExe = $localCargo
} else { $cargoExe = (Get-Command cargo -ErrorAction Stop).Source }
Push-Location $root
try {
    & $cargoExe @cargoArguments
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} finally { Pop-Location }
