#ifndef MyAppVersion
  #define MyAppVersion "2.1.0"
#endif

[Setup]
AppId={{A913EB65-C10B-4C1F-8845-627C4921488B}
AppName=CodexPet
AppVersion={#MyAppVersion}
AppVerName=CodexPet {#MyAppVersion}
AppPublisher=ezenwa
AppPublisherURL=https://github.com/ezenwa/CodexPet
AppSupportURL=https://github.com/ezenwa/CodexPet/issues
AppUpdatesURL=https://github.com/ezenwa/CodexPet/releases/latest
DefaultDirName={localappdata}\CodexPet
DefaultGroupName=CodexPet
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\dist
OutputBaseFilename=CodexPet-Setup
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
SetupLogging=yes
CloseApplications=yes
CloseApplicationsFilter=CodexPet.exe
RestartApplications=no
UninstallDisplayName=CodexPet
VersionInfoVersion={#MyAppVersion}
VersionInfoDescription=CodexPet Setup
VersionInfoProductName=CodexPet
VersionInfoProductVersion={#MyAppVersion}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"

[Files]
Source: "..\dist\CodexPet\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "Stop-CodexPet.ps1"; Flags: dontcopy
Source: "language.en.txt"; DestDir: "{userappdata}\CodexPet"; DestName: "language.txt"; Languages: english; Flags: onlyifdoesntexist
Source: "language.es.txt"; DestDir: "{userappdata}\CodexPet"; DestName: "language.txt"; Languages: spanish; Flags: onlyifdoesntexist

[Icons]
Name: "{autodesktop}\CodexPet"; Filename: "{app}\CodexPet.exe"; WorkingDir: "{app}"
Name: "{group}\CodexPet"; Filename: "{app}\CodexPet.exe"; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,CodexPet}"; Filename: "{uninstallexe}"

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "CodexPetWatcher"; ValueData: """{app}\CodexPet.exe"" --watch"; Flags: uninsdeletevalue

[Run]
Filename: "{app}\CodexPet.exe"; Parameters: "--watch"; WorkingDir: "{app}"; Flags: nowait

[UninstallRun]
Filename: "{app}\CodexPet.exe"; Parameters: "--stop"; Flags: runhidden waituntilterminated; RunOnceId: "StopCodexPet"

[InstallDelete]
; Retire the old hashed asset layout before copying the current flat PNG folders.
Type: filesandordirs; Name: "{app}\assets\pets\codex\sha256-*"
Type: filesandordirs; Name: "{app}\assets\pets\bsod\sha256-*"
Type: filesandordirs; Name: "{app}\assets\pets\dewey\sha256-*"
Type: filesandordirs; Name: "{app}\assets\pets\fireball\sha256-*"
Type: filesandordirs; Name: "{app}\assets\pets\null-signal\sha256-*"
Type: filesandordirs; Name: "{app}\assets\pets\rocky\sha256-*"
Type: filesandordirs; Name: "{app}\assets\pets\seedy\sha256-*"
Type: filesandordirs; Name: "{app}\assets\pets\stacky\sha256-*"
Type: files; Name: "{app}\README.en.md"
Type: files; Name: "{userdesktop}\Codex Pet.lnk"
Type: files; Name: "{userprograms}\Codex Pet.lnk"
Type: files; Name: "{userstartup}\Codex Pet.lnk"
Type: files; Name: "{userstartup}\CodexPet.lnk"
Type: files; Name: "{app}\CodexPet.ps1"
Type: files; Name: "{app}\CodexPet-State.ps1"
Type: files; Name: "{app}\CodexPet-Watcher.ps1"
Type: files; Name: "{app}\Start-CodexPetWatcher.vbs"
Type: files; Name: "{app}\Start-CodexPet.cmd"
Type: files; Name: "{app}\Stop-CodexPet.ps1"
Type: files; Name: "{app}\Uninstall-CodexPet.ps1"
Type: files; Name: "{app}\codexpet.pid"
Type: files; Name: "{app}\close-with-codex.request"

[UninstallDelete]
Type: files; Name: "{app}\watcher.log"

[Code]
function PrepareToInstall(var NeedsRestart: Boolean): String;
var Code: Integer;
begin
  Result := '';
  if DirExists(ExpandConstant('{app}')) then begin
    ExtractTemporaryFile('Stop-CodexPet.ps1');
    if not Exec(ExpandConstant('{sys}\WindowsPowerShell\v1.0\powershell.exe'),
      '-NoLogo -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File "' + ExpandConstant('{tmp}\Stop-CodexPet.ps1') + '" -InstallDir "' + ExpandConstant('{app}') + '"',
      '', SW_HIDE, ewWaitUntilTerminated, Code) then
      Result := 'Could not stop the previous CodexPet installation.'
    else if Code <> 0 then
      Result := 'The previous CodexPet installation could not be stopped.';
  end;
end;
