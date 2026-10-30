$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$root = Split-Path $PSScriptRoot -Parent
$env:CARGO_HOME = Join-Path $root '.build\cargo'
$env:RUSTUP_HOME = Join-Path $root '.build\rustup'
$installer = Join-Path $root '.build\rustup-init.exe'
New-Item -ItemType Directory -Path (Split-Path $installer -Parent) -Force | Out-Null
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
if (-not (Test-Path -LiteralPath $installer)) {
    Invoke-WebRequest -UseBasicParsing -Uri 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe' -OutFile $installer
}
& $installer -y --no-modify-path --profile minimal --component rustfmt --component clippy --default-host x86_64-pc-windows-msvc --default-toolchain stable
if ($LASTEXITCODE -ne 0) { throw 'Rust setup failed.' }
& (Join-Path $env:CARGO_HOME 'bin\rustc.exe') --version
