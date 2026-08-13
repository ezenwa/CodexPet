function New-CodexPetSessionState {
    [pscustomobject]@{
        Path           = ''
        Offset         = 0L
        PendingText    = ''
        TurnActive     = $false
        WaitingForUser = $false
        TerminalState  = 'Idle'
        TerminalUtc    = [DateTime]::MinValue
    }
}

if (-not ('CodexPetNativeSqlite' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class CodexPetNativeSqlite {
    [DllImport("winsqlite3.dll", CallingConvention=CallingConvention.Cdecl)]
    static extern int sqlite3_open_v2([MarshalAs(UnmanagedType.LPStr)] string filename, out IntPtr database, int flags, IntPtr vfs);
    [DllImport("winsqlite3.dll", CallingConvention=CallingConvention.Cdecl)]
    static extern int sqlite3_prepare_v2(IntPtr database, [MarshalAs(UnmanagedType.LPStr)] string sql, int bytes, out IntPtr statement, IntPtr tail);
    [DllImport("winsqlite3.dll", CallingConvention=CallingConvention.Cdecl)] static extern int sqlite3_step(IntPtr statement);
    [DllImport("winsqlite3.dll", CallingConvention=CallingConvention.Cdecl)] static extern long sqlite3_column_int64(IntPtr statement, int column);
    [DllImport("winsqlite3.dll", CallingConvention=CallingConvention.Cdecl)] static extern int sqlite3_finalize(IntPtr statement);
    [DllImport("winsqlite3.dll", CallingConvention=CallingConvention.Cdecl)] static extern int sqlite3_close(IntPtr database);
    public static long ScalarInt64(string path, string sql) {
        IntPtr database;
        if (sqlite3_open_v2(path, out database, 1, IntPtr.Zero) != 0) return -1;
        try {
            IntPtr statement;
            if (sqlite3_prepare_v2(database, sql, -1, out statement, IntPtr.Zero) != 0) return -2;
            try { return sqlite3_step(statement) == 100 ? sqlite3_column_int64(statement, 0) : -3; }
            finally { sqlite3_finalize(statement); }
        } finally { sqlite3_close(database); }
    }
}
'@
}

function Test-CodexPetApprovalPending([string]$DatabasePath) {
    if (-not (Test-Path -LiteralPath $DatabasePath)) { return $false }
    $requestSql = "SELECT id FROM logs WHERE target='codex_core::stream_events_utils' AND feedback_log_body LIKE '%handle_output_item_done: ToolCall:%' AND feedback_log_body LIKE '%sandbox_permissions%require_escalated%' ORDER BY id DESC LIMIT 1"
    $resolutionSql = "SELECT id FROM logs WHERE (target='codex_core::session::handlers' AND feedback_log_body LIKE '%op: ExecApproval {%') OR (target='codex_core::tools::parallel' AND feedback_log_body LIKE '%tool call completed%') ORDER BY id DESC LIMIT 1"
    try {
        $request = [CodexPetNativeSqlite]::ScalarInt64($DatabasePath, $requestSql)
        $resolution = [CodexPetNativeSqlite]::ScalarInt64($DatabasePath, $resolutionSql)
        return $request -gt 0 -and $request -gt $resolution
    } catch { return $false }
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
        $timestampMatch = [regex]::Match($line, '"timestamp"\s*:\s*"(?<timestamp>[^"]+)"')
        $eventUtc = [DateTime]::UtcNow
        if ($timestampMatch.Success) {
            [DateTime]::TryParse(
                $timestampMatch.Groups['timestamp'].Value,
                [Globalization.CultureInfo]::InvariantCulture,
                [Globalization.DateTimeStyles]::AdjustToUniversal,
                [ref]$eventUtc
            ) | Out-Null
        }

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
                $Session.TerminalUtc = $eventUtc.ToUniversalTime()
                continue
            }
            '^(turn_aborted|error|failed)$' {
                $Session.TurnActive = $false
                $Session.WaitingForUser = $false
                $Session.TerminalState = 'Failed'
                $Session.TerminalUtc = $eventUtc.ToUniversalTime()
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
        $Session.TerminalUtc = [DateTime]::MinValue
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
