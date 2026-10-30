$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$root = Split-Path $PSScriptRoot -Parent
$dist = Join-Path $root 'dist'
$zipPath = Join-Path $dist 'CodexPet-Portable.zip'
$portable = Join-Path $dist 'CodexPet'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [IO.Compression.ZipFile]::OpenRead($zipPath)
$verified = 0
try {
    foreach ($entry in $archive.Entries) {
        if (-not $entry.Name) { continue }
        $name = $entry.FullName.Replace('\','/')
        if (-not $name.StartsWith('CodexPet/')) { throw "Unexpected ZIP path: $name" }
        $relative = $name.Substring('CodexPet/'.Length)
        if ($relative -match '(^|/)\.\.?(/|$)' -or $relative.StartsWith('/')) { throw 'Unsafe ZIP path.' }
        $file = Join-Path $portable $relative
        if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "Unexpected ZIP entry: $name" }
        $stream = $entry.Open()
        $sha = [Security.Cryptography.SHA256]::Create()
        try { $hash = [BitConverter]::ToString($sha.ComputeHash($stream)).Replace('-','') }
        finally { $stream.Dispose(); $sha.Dispose() }
        if ($hash -ne (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash) { throw "ZIP hash mismatch: $name" }
        $verified++
    }
    $sourceCount = @(Get-ChildItem -LiteralPath $portable -Recurse -File).Count
    if ($verified -ne $sourceCount) { throw 'ZIP does not contain every portable file.' }
} finally { $archive.Dispose() }
$frameCount = @(Get-ChildItem -LiteralPath (Join-Path $portable 'assets') -Recurse -Filter 'frame_*.png' -File).Count
if ($frameCount -ne 576) { throw "Expected 576 frames, found $frameCount." }
$exeBytes = [IO.File]::ReadAllBytes((Join-Path $portable 'CodexPet.exe'))
$pe = [BitConverter]::ToInt32($exeBytes,0x3c)
if ([BitConverter]::ToUInt16($exeBytes,$pe + 24 + 68) -ne 2) { throw 'Executable must use the Windows GUI subsystem.' }
$setup = Get-Item -LiteralPath (Join-Path $dist 'CodexPet-Setup.exe')
$version = [regex]::Match((Get-Content -LiteralPath (Join-Path $root 'Cargo.toml') -Raw), '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
if ((Get-Item -LiteralPath (Join-Path $portable 'CodexPet.exe')).VersionInfo.ProductVersion.Trim() -ne $version) { throw 'Executable version resource mismatch.' }
if (-not ('CodexPetPackageIcon' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class CodexPetPackageIcon {
    [DllImport("shell32.dll", CharSet=CharSet.Unicode)]
    public static extern uint ExtractIconExW(string file, int index, IntPtr large, IntPtr small, uint count);
}
'@
}
if ([CodexPetPackageIcon]::ExtractIconExW((Join-Path $portable 'CodexPet.exe'),-1,[IntPtr]::Zero,[IntPtr]::Zero,0) -ne 1) { throw 'The executable must have one embedded application icon.' }
if ($setup.VersionInfo.ProductVersion.Trim() -ne $version) { throw 'Installer version mismatch.' }
"Package verified: $verified files, $frameCount frames, GUI executable, installer v$version."
# An extracted path with spaces catches packaging and executable argument regressions.
$tempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\')
$extracted = Join-Path $tempRoot ('CodexPet package verify ' + [guid]::NewGuid().ToString('N'))
try {
    Expand-Archive -LiteralPath $zipPath -DestinationPath $extracted -Force
    & (Join-Path $PSScriptRoot 'Verify-Native.ps1') -Executable (Join-Path $extracted 'CodexPet\CodexPet.exe')
} finally {
    $resolved = [IO.Path]::GetFullPath($extracted)
    if (-not $resolved.StartsWith($tempRoot + '\CodexPet package verify ', [StringComparison]::OrdinalIgnoreCase)) { throw 'Unexpected portable verification cleanup path.' }
    if (Test-Path -LiteralPath $resolved) { Remove-Item -LiteralPath $resolved -Recurse -Force }
}
