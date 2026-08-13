function New-CodexPetSessionState {
    [pscustomobject]@{
        Path           = ''
        Offset         = 0L
        PendingText    = ''
        TurnActive     = $false
        WaitingForUser = $false
        TerminalState  = 'Idle'
    }
}

function Update-CodexPetSessionState($Session, [string[]]$Lines) {
    foreach ($line in $Lines) {
        if ([string]::IsNullOrWhiteSpace($line)) { continue }

        # Session records may contain multi-megabyte tool outputs. Extract only
        # the structural fields needed for state detection instead of decoding
        # the entire JSON object on WPF's UI thread.
        $payloadMatch = [regex]::Match(
            $line,
            '"payload"\s*:\s*\{\s*"type"\s*:\s*"(?<type>[^"]+)"',
            [Text.RegularExpressions.RegexOptions]::CultureInvariant
        )
        if (-not $payloadMatch.Success) { continue }
        $eventType = $payloadMatch.Groups['type'].Value

        switch -Regex ($eventType) {
            '^(task_started|turn_started)$' {
                $Session.TurnActive = $true
                $Session.WaitingForUser = $false
                $Session.TerminalState = 'Working'
                continue
            }
            '^(task_complete|turn_complete|turn_completed)$' {
                $Session.TurnActive = $false
                $Session.WaitingForUser = $false
                $Session.TerminalState = 'Ready'
                continue
            }
            '^(turn_aborted|error|failed)$' {
                $Session.TurnActive = $false
                $Session.WaitingForUser = $false
                $Session.TerminalState = 'Failed'
                continue
            }
            '^(user_message|user_input|approval_response|elicitation_response)$' {
                if ($Session.TurnActive) { $Session.WaitingForUser = $false }
                continue
            }
            'approval|request_user_input|elicitation' {
                if ($Session.TurnActive) { $Session.WaitingForUser = $true }
                continue
            }
            '^(function_call|custom_tool_call)$' {
                $nameMatch = [regex]::Match(
                    $line,
                    '"name"\s*:\s*"(?<name>[^"]+)"',
                    [Text.RegularExpressions.RegexOptions]::CultureInvariant
                )
                $callName = $(if ($nameMatch.Success) { $nameMatch.Groups['name'].Value } else { '' })
                if ($callName -match 'request_user_input|approval|elicitation') {
                    if ($Session.TurnActive) { $Session.WaitingForUser = $true }
                }
                continue
            }
            '^(function_call_output|custom_tool_call_output)$' {
                if ($Session.TurnActive -and $Session.WaitingForUser) {
                    $Session.WaitingForUser = $false
                }
                continue
            }
        }
    }

    if ($Session.WaitingForUser) { return 'Input' }
    if ($Session.TurnActive) { return 'Working' }
    return $Session.TerminalState
}

function Read-CodexPetSessionChanges($Session, [IO.FileInfo]$SessionFile) {
    if ($Session.Path -ne $SessionFile.FullName -or $SessionFile.Length -lt $Session.Offset) {
        $Session.Path = $SessionFile.FullName
        $Session.Offset = 0L
        $Session.PendingText = ''
        $Session.TurnActive = $false
        $Session.WaitingForUser = $false
        $Session.TerminalState = 'Idle'
    }
    if ($SessionFile.Length -eq $Session.Offset) {
        return $(if ($Session.WaitingForUser) { 'Input' } elseif ($Session.TurnActive) { 'Working' } else { $Session.TerminalState })
    }

    $stream = $null
    try {
        $stream = [IO.FileStream]::new(
            $SessionFile.FullName,
            [IO.FileMode]::Open,
            [IO.FileAccess]::Read,
            ([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete)
        )
        [void]$stream.Seek($Session.Offset, [IO.SeekOrigin]::Begin)
        $buffer = New-Object byte[] 65536
        $decoder = [Text.Encoding]::UTF8.GetDecoder()
        $chars = New-Object char[] ([Text.Encoding]::UTF8.GetMaxCharCount($buffer.Length))
        $text = New-Object Text.StringBuilder
        while (($read = $stream.Read($buffer, 0, $buffer.Length)) -gt 0) {
            $charCount = $decoder.GetChars($buffer, 0, $read, $chars, 0, $false)
            [void]$text.Append($chars, 0, $charCount)
        }
        $charCount = $decoder.GetChars([byte[]]@(), 0, 0, $chars, 0, $true)
        if ($charCount) { [void]$text.Append($chars, 0, $charCount) }
        $Session.Offset = $stream.Position
        $combined = $Session.PendingText + $text.ToString()
    } catch {
        return $(if ($Session.WaitingForUser) { 'Input' } elseif ($Session.TurnActive) { 'Working' } else { $Session.TerminalState })
    } finally {
        if ($stream) { $stream.Dispose() }
    }

    $parts = [regex]::Split($combined, '\r?\n')
    $Session.PendingText = $parts[-1]
    if ($parts.Count -le 1) { return $(if ($Session.WaitingForUser) { 'Input' } elseif ($Session.TurnActive) { 'Working' } else { $Session.TerminalState }) }
    return Update-CodexPetSessionState $Session $parts[0..($parts.Count - 2)]
}
