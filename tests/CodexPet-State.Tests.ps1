$ErrorActionPreference = 'Stop'
. (Join-Path (Split-Path $PSScriptRoot -Parent) 'CodexPet-State.ps1')

function Event([string]$Type, [string]$Name = '') {
    $payload = [ordered]@{ type = $Type }
    if ($Name) { $payload.name = $Name }
    return (@{ type='event_msg'; payload=$payload } | ConvertTo-Json -Compress)
}
function Assert-State([string]$Expected, [string]$Actual, [string]$Case) {
    if ($Expected -ne $Actual) { throw "$Case esperaba $Expected y devolvió $Actual" }
}

$session = New-CodexPetSessionState
Assert-State Idle (Update-CodexPetSessionState $session @()) 'sin turno'
Assert-State Working (Update-CodexPetSessionState $session @((Event task_started))) 'inicio'
Assert-State Input (Update-CodexPetSessionState $session @((Event function_call request_user_input))) 'solicitud de usuario'
Assert-State Working (Update-CodexPetSessionState $session @((Event function_call_output))) 'respuesta del usuario'
Assert-State Ready (Update-CodexPetSessionState $session @((Event task_complete))) 'finalización'
Assert-State Ready (Update-CodexPetSessionState $session @()) 'finalización persistente'
Assert-State Working (Update-CodexPetSessionState $session @((Event task_started))) 'segundo inicio'
Assert-State Failed (Update-CodexPetSessionState $session @((Event turn_aborted))) 'aborto'

$largeOutput = '{"type":"response_item","payload":{"type":"custom_tool_call_output","output":"' + ('x' * 1000000) + '"}}'
$performanceState = New-CodexPetSessionState
$performanceWatch = [Diagnostics.Stopwatch]::StartNew()
Assert-State Idle (Update-CodexPetSessionState $performanceState @($largeOutput)) 'salida grande'
$performanceWatch.Stop()
if ($performanceWatch.ElapsedMilliseconds -gt 500) { throw "El análisis de salida grande tardó $($performanceWatch.ElapsedMilliseconds) ms" }

$tempFile = Join-Path ([IO.Path]::GetTempPath()) ("CodexPet-State-{0}.jsonl" -f [guid]::NewGuid())
try {
    @((Event task_started), (Event task_complete)) | Set-Content -LiteralPath $tempFile -Encoding utf8
    $readerState = New-CodexPetSessionState
    Assert-State Ready (Read-CodexPetSessionChanges $readerState (Get-Item $tempFile)) 'lectura inicial completa'
    (Event task_started) | Add-Content -LiteralPath $tempFile -Encoding utf8
    Assert-State Working (Read-CodexPetSessionChanges $readerState (Get-Item $tempFile)) 'lectura incremental'
} finally {
    Remove-Item -LiteralPath $tempFile -Force -ErrorAction SilentlyContinue
}

'Todos los estados de CodexPet son correctos.'
