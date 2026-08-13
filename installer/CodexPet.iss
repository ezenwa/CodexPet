#ifndef MyAppVersion
  #define MyAppVersion "1.2.1"
#endif

#define MyAppName "CodexPet"
#define MyAppPublisher "ezenwa"
#define MyAppURL "https://github.com/ezenwa/CodexPet"

[Setup]
AppId={{A913EB65-C10B-4C1F-8845-627C4921488B}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases/latest
DefaultDirName={localappdata}\CodexPet
DefaultGroupName=CodexPet
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir=..\dist
OutputBaseFilename=CodexPet-Setup
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
SetupLogging=yes
CloseApplications=no
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
Source: "..\CodexPet.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\CodexPet-State.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\CodexPet-Watcher.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\Stop-CodexPet.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\Start-CodexPetWatcher.vbs"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\Start-CodexPet.cmd"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.en.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\VERSION"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\docs\*"; DestDir: "{app}\docs"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "language.en.txt"; DestDir: "{userappdata}\CodexPet"; DestName: "language.txt"; Languages: english; Flags: onlyifdoesntexist
Source: "language.es.txt"; DestDir: "{userappdata}\CodexPet"; DestName: "language.txt"; Languages: spanish; Flags: onlyifdoesntexist

[Icons]
Name: "{autodesktop}\CodexPet"; Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoLogo -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File ""{app}\CodexPet.ps1"" -Startup"; WorkingDir: "{app}"
Name: "{group}\CodexPet"; Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoLogo -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File ""{app}\CodexPet.ps1"" -Startup"; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,CodexPet}"; Filename: "{uninstallexe}"

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "CodexPetWatcher"; ValueData: """{sys}\wscript.exe"" //B //Nologo ""{app}\Start-CodexPetWatcher.vbs"""; Flags: uninsdeletevalue

[Run]
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoLogo -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File ""{app}\Stop-CodexPet.ps1"""; Flags: runhidden waituntilterminated
Filename: "{sys}\wscript.exe"; Parameters: "//B //Nologo ""{app}\Start-CodexPetWatcher.vbs"""; WorkingDir: "{app}"; Flags: nowait runhidden

[UninstallRun]
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoLogo -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File ""{app}\Stop-CodexPet.ps1"""; Flags: runhidden waituntilterminated; RunOnceId: "StopCodexPet"

[InstallDelete]
Type: files; Name: "{userdesktop}\Codex Pet.lnk"
Type: files; Name: "{userdesktop}\CodexPet.lnk"
Type: files; Name: "{userprograms}\Codex Pet.lnk"
Type: files; Name: "{userprograms}\CodexPet.lnk"
Type: files; Name: "{userprograms}\Desinstalar Codex Pet.lnk"
Type: files; Name: "{userprograms}\Desinstalar CodexPet.lnk"
Type: files; Name: "{userstartup}\Codex Pet.lnk"
Type: files; Name: "{userstartup}\CodexPet.lnk"
Type: files; Name: "{app}\Uninstall-CodexPet.ps1"

[UninstallDelete]
Type: files; Name: "{app}\watcher.log"
Type: files; Name: "{app}\codexpet.pid"
Type: files; Name: "{app}\close-with-codex.request"
