param([switch]$PortableOnly)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$root = Split-Path $PSScriptRoot -Parent
$cargo = Join-Path $root 'tools\Cargo.ps1'
$manifest = Get-Content -LiteralPath (Join-Path $root 'Cargo.toml') -Raw
$version = [regex]::Match($manifest, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
if (-not $version) { throw 'Cargo.toml must declare a version.' }
& $cargo test --locked
if ($LASTEXITCODE -ne 0) { throw 'Rust tests failed.' }
& $cargo build --release --locked
if ($LASTEXITCODE -ne 0) { throw 'Rust compilation failed.' }
$dist = Join-Path $root 'dist'
$portable = Join-Path $dist 'CodexPet'
if (Test-Path -LiteralPath $portable) {
    $resolvedPortable = [IO.Path]::GetFullPath($portable)
    if ($resolvedPortable -ne ([IO.Path]::GetFullPath($root).TrimEnd('\') + '\dist\CodexPet')) { throw 'Unexpected portable build destination.' }
    Remove-Item -LiteralPath $resolvedPortable -Recurse -Force
}
New-Item -ItemType Directory -Path $portable -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $root 'target\release\CodexPet.exe') -Destination $portable -Force
Copy-Item -LiteralPath (Join-Path $root 'assets') -Destination $portable -Recurse -Force
Copy-Item -LiteralPath (Join-Path $root 'docs') -Destination $portable -Recurse -Force
foreach ($file in @('README.md','README.es.md','CHANGELOG.md','CHANGELOG.es.md')) {
    Copy-Item -LiteralPath (Join-Path $root $file) -Destination $portable -Force
}
$version | Set-Content -LiteralPath (Join-Path $portable 'VERSION') -Encoding ascii
$portableZip = Join-Path $dist 'CodexPet-Portable.zip'
Compress-Archive -LiteralPath $portable -DestinationPath $portableZip -CompressionLevel Optimal -Force
if (-not $PortableOnly) {
    $candidates = @(
        (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe'),
        (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'),
        (Join-Path $env:ProgramFiles 'Inno Setup 6\ISCC.exe')
    )
    $iscc = $candidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
    if (-not $iscc) { throw 'Inno Setup 6 not found. The portable build is ready; install Inno Setup to build the installer.' }
    & $iscc '/Q' "/DMyAppVersion=$version" (Join-Path $root 'installer\CodexPet.iss')
    if ($LASTEXITCODE -ne 0) { throw 'Inno Setup failed.' }
}
Get-ChildItem -LiteralPath $dist -File | Select-Object Name,Length,LastWriteTime
