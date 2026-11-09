# Throwaway probe for footplate-ghen, taken out before the bead finishes.
#
# Does `footplate.exe --version` reach a console, as it does for a person at a
# prompt? A step's own output is a pipe, and the exe prints into a pipe with or
# without AttachConsole, so the log cannot tell. This runs the exe in a console
# of its own, once as a PowerShell prompt does and once as cmd does, and reads
# back what that console shows.
#
# Without -InConsole this is the launcher: it starts itself in a new console and
# prints the report that run leaves. With it, this is the run in the console.
# Nothing written to the console may hold the line that is looked for.
param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$Version,
    [string]$Out = (Join-Path $env:TEMP 'console-probe.txt'),
    [switch]$InConsole
)

$Exe = (Resolve-Path -LiteralPath $Exe).Path
$want = "footplate $Version"

if (-not $InConsole) {
    Remove-Item -LiteralPath $Out -ErrorAction SilentlyContinue
    $shell = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
    $line = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -InConsole -Exe `"$Exe`" -Version `"$Version`" -Out `"$Out`""
    $child = Start-Process -FilePath $shell -ArgumentList $line -WindowStyle Minimized -Wait -PassThru
    $result = 'fault'
    if (Test-Path -LiteralPath $Out) {
        $report = Get-Content -LiteralPath $Out
        $report
        $last = $report | Select-Object -Last 1
        if ($last -match '^RESULT (green|red)$') { $result = $Matches[1] }
    } else {
        'the console wrote no report'
    }
    "the console's PowerShell exited with $($child.ExitCode); the probe says $result"
    switch ($result) { 'green' { exit 0 } 'red' { exit 1 } default { exit 2 } }
}

$ErrorActionPreference = 'Stop'
$report = New-Object System.Collections.Generic.List[string]
function Note([string]$text) { $report.Add($text) }

# Every row the console shows, down to the cursor's, as text.
function Rows {
    $raw = $Host.UI.RawUI
    $width = $raw.BufferSize.Width
    $last = $raw.CursorPosition.Y
    $rect = New-Object System.Management.Automation.Host.Rectangle 0, 0, ($width - 1), $last
    $cells = $raw.GetBufferContents($rect)
    for ($y = 0; $y -le $last; $y++) {
        $chars = for ($x = 0; $x -lt $width; $x++) { $cells[$y, $x].Character }
        (-join $chars).TrimEnd()
    }
}

# How many rows say the line.
function Said { @(Rows | Where-Object { $_ -match [regex]::Escape($want) }).Count }

# Waits until the line shows `count` times, up to `seconds`; answers how many
# times it shows.
function Wait-Said([int]$count, [int]$seconds) {
    $until = (Get-Date).AddSeconds($seconds)
    do {
        Start-Sleep -Milliseconds 250
        $said = Said
    } while ($said -lt $count -and (Get-Date) -lt $until)
    $said
}

$code = 2
try {
    Note "PowerShell $($PSVersionTable.PSVersion) on $([Environment]::OSVersion.VersionString), host $($Host.Name), output redirected: $([Console]::IsOutputRedirected), buffer $($Host.UI.RawUI.BufferSize.Width)x$($Host.UI.RawUI.BufferSize.Height)"

    # The control: the console shows what is written to it, so that seeing
    # nothing of the line means the exe did not say it, and not that the
    # console cannot be read.
    $marker = 'console marker ' + [guid]::NewGuid().ToString('N').Substring(0, 8)
    Write-Host $marker
    $marked = @(Rows | Where-Object { $_ -eq $marker }).Count
    Note "the marker this probe wrote shows $marked time(s), wanted 1"
    if ($marked -ne 1) { throw 'the console cannot be read back' }

    # A PowerShell prompt.
    & $Exe --version
    $first = Wait-Said 1 15
    Note "after the PowerShell prompt the line shows $first time(s)"

    # A cmd prompt: a batch file stands for it, and stays alive a few seconds
    # after the exe has started, as the prompt does.
    $batch = Join-Path (Split-Path -Parent $Out) 'console-probe.cmd'
    Set-Content -LiteralPath $batch -Encoding ASCII -Value @('@echo off', '"%~1" --version', 'ping -n 4 127.0.0.1 >nul')
    & $batch $Exe
    $second = Wait-Said ($first + 1) 15
    Note "after the cmd prompt the line shows $second time(s)"

    Note 'the console, row by row:'
    Rows | ForEach-Object { Note "| $_" }

    $viaPowerShell = $first -ge 1
    $viaCmd = $second -gt $first
    Note "PowerShell prompt: $(if ($viaPowerShell) { 'green' } else { 'red' }); cmd prompt: $(if ($viaCmd) { 'green' } else { 'red' })"
    $code = if ($viaPowerShell -and $viaCmd) { 0 } else { 1 }
} catch {
    Note "FAULT: $($_ | Out-String)"
    $code = 2
}
Note ('RESULT ' + @('green', 'red', 'fault')[$code])
Set-Content -LiteralPath $Out -Value $report
exit $code
