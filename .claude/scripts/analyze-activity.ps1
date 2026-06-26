# analyze-activity.ps1 — mine Claude Code transcripts for the self-improvement
# flywheel (rtk-free). Aggregates the agent's own tool activity into:
#   - Toil:        most-repeated actions (frequency)
#   - Token hogs:  actions by total result size (~chars/4 = tokens)
#   - Friction:    actions whose results contained errors
#
# Reads ~/.claude/projects/<project>/*.jsonl. Output is plain text for the
# /self-improve command to interpret into a proposal report.
[CmdletBinding()]
param(
    [string]$ProjectGlob = '*RAcingtycoon*',
    [int]$Days = 30,
    [int]$TopN = 15
)

$root = Join-Path $env:USERPROFILE '.claude\projects'
if (-not (Test-Path $root)) { Write-Output "No Claude projects dir at $root"; exit 0 }

$cutoff = (Get-Date).AddDays(-$Days)
$files = Get-ChildItem $root -Directory |
    Where-Object { $_.Name -like $ProjectGlob } |
    ForEach-Object { Get-ChildItem $_.FullName -Filter *.jsonl -ErrorAction SilentlyContinue } |
    Where-Object { $_.LastWriteTime -ge $cutoff }

if (-not $files) { Write-Output "No transcripts in the last $Days days for '$ProjectGlob'."; exit 0 }

$freq = @{}; $chars = @{}; $errs = @{}; $idToSig = @{}
$sessions = 0; $totalTools = 0

function Get-Signature([string]$tool, [string]$cmd) {
    if (($tool -eq 'Bash' -or $tool -eq 'PowerShell') -and $cmd) {
        $firstLine = ($cmd -split "`n")[0]
        $toks = $firstLine -split '\s+' | Where-Object {
            $_ -and ($_ -notmatch '^-') -and ($_ -notmatch '[\\/:]') -and ($_ -notmatch '^[''"$({]')
        }
        $sig = ($toks | Select-Object -First 2) -join ' '
        if (-not $sig) { $sig = $tool }
        return "sh: $sig"
    }
    return $tool
}

foreach ($f in $files) {
    $sessions++
    foreach ($line in [System.IO.File]::ReadLines($f.FullName)) {
        if (-not $line) { continue }
        try { $o = $line | ConvertFrom-Json } catch { continue }
        if ($o.type -eq 'assistant' -and $o.message.content) {
            foreach ($c in $o.message.content) {
                if ($c.type -ne 'tool_use') { continue }
                $cmd = $null
                if ($c.input -and ($c.input.PSObject.Properties.Name -contains 'command')) { $cmd = $c.input.command }
                $sig = Get-Signature $c.name $cmd
                $freq[$sig] = [int]$freq[$sig] + 1
                $idToSig[$c.id] = $sig
                $totalTools++
            }
        } elseif ($o.type -eq 'user' -and $o.message.content) {
            foreach ($c in $o.message.content) {
                if ($c.type -ne 'tool_result') { continue }
                $sig = $idToSig[$c.tool_use_id]
                if (-not $sig) { continue }
                $txt = if ($c.content -is [string]) { $c.content } else { ($c.content | ForEach-Object { $_.text }) -join '' }
                $chars[$sig] = [int]$chars[$sig] + $txt.Length
                if ($c.is_error -eq $true -or $txt -match 'error\[|error:|FAILED|panicked|fatal:') {
                    $errs[$sig] = [int]$errs[$sig] + 1
                }
            }
        }
    }
}

Write-Output "Activity analysis - '$ProjectGlob' - last $Days days"
Write-Output "Sessions: $sessions   Tool calls: $totalTools"
Write-Output ""
Write-Output "## Toil - most-repeated actions (frequency)"
$freq.GetEnumerator() | Sort-Object Value -Descending | Select-Object -First $TopN |
    ForEach-Object { "{0,5}x  {1}" -f $_.Value, $_.Key }
Write-Output ""
Write-Output "## Token hogs - by total result size (~chars/4 = tokens)"
$chars.GetEnumerator() | Sort-Object Value -Descending | Select-Object -First $TopN |
    ForEach-Object { "{0,10:n0} chars (~{1,8:n0} tok)  {2}" -f $_.Value, ($_.Value / 4), $_.Key }
Write-Output ""
Write-Output "## Friction - actions whose results contained errors"
$errs.GetEnumerator() | Sort-Object Value -Descending | Select-Object -First $TopN |
    ForEach-Object { "{0,5} errs  {1}" -f $_.Value, $_.Key }
