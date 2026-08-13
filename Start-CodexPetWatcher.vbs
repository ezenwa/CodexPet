Option Explicit

Dim shell, fileSystem, installDir, powerShellExe, watcherScript, command
Set shell = CreateObject("WScript.Shell")
Set fileSystem = CreateObject("Scripting.FileSystemObject")

installDir = fileSystem.GetParentFolderName(WScript.ScriptFullName)
powerShellExe = shell.ExpandEnvironmentStrings("%SystemRoot%") & "\System32\WindowsPowerShell\v1.0\powershell.exe"
watcherScript = installDir & "\CodexPet-Watcher.ps1"
command = """" & powerShellExe & """ -NoLogo -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File """ & watcherScript & """ -InstallDir """ & installDir & """"

shell.Run command, 0, False
