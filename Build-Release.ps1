param([string]$Version = '1.1.0')

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$installerDir = Join-Path $root 'installer'
$distDir = Join-Path $root 'dist'
$buildDir = Join-Path $root '.build'
$payloadPath = Join-Path $installerDir 'CodexPet-Payload.zip'
$setupPath = Join-Path $distDir 'CodexPet-Setup.exe'
$setupZipPath = Join-Path $distDir 'CodexPet-Setup.zip'

New-Item -ItemType Directory -Path $installerDir, $distDir, $buildDir -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $root 'Install-CodexPet.ps1') -Destination (Join-Path $installerDir 'Install-CodexPet.ps1') -Force
Copy-Item -LiteralPath (Join-Path $root 'Install.cmd') -Destination (Join-Path $installerDir 'Install.cmd') -Force

$payloadItems = @(
    (Join-Path $root 'assets'),
    (Join-Path $root 'docs'),
    (Join-Path $root 'CodexPet.ps1'),
    (Join-Path $root 'CodexPet-Watcher.ps1'),
    (Join-Path $root 'Start-CodexPetWatcher.vbs'),
    (Join-Path $root 'Start-CodexPet.cmd'),
    (Join-Path $root 'Uninstall-CodexPet.ps1'),
    (Join-Path $root 'README.md'),
    (Join-Path $root 'CHANGELOG.md')
)
Compress-Archive -LiteralPath $payloadItems -DestinationPath $payloadPath -CompressionLevel Optimal -Force

$sedPath = Join-Path $buildDir 'CodexPet-Setup.sed'
$sed = @"
[Version]
Class=IEXPRESS
SEDVersion=3

[Options]
PackagePurpose=InstallApp
ShowInstallProgramWindow=0
HideExtractAnimation=1
UseLongFileName=1
InsideCompressed=0
CAB_FixedSize=0
CAB_ResvCodeSigning=0
RebootMode=N
InstallPrompt=
DisplayLicense=
FinishMessage=
TargetName=$setupPath
FriendlyName=CodexPet $Version Setup
AppLaunched=Install.cmd
PostInstallCmd=<None>
AdminQuietInstCmd=Install.cmd
UserQuietInstCmd=Install.cmd
SourceFiles=SourceFiles

[Strings]
FILE0="Install.cmd"
FILE1="Install-CodexPet.ps1"
FILE2="CodexPet-Payload.zip"

[SourceFiles]
SourceFiles0=$installerDir\

[SourceFiles0]
%FILE0%=
%FILE1%=
%FILE2%=
"@
$sed | Set-Content -LiteralPath $sedPath -Encoding ascii

[pscustomobject]@{
    Version = $Version
    Sed     = $sedPath
    Payload = $payloadPath
    Setup   = $setupPath
    Zip     = $setupZipPath
}
