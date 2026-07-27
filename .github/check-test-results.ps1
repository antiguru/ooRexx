# Decides whether a Windows test suite run passed.
#
# The suite's own exit code is not enough on its own. It reports new failures,
# errors and exceptions, but a handful of its tests cannot pass on a hosted
# runner for reasons that have nothing to do with the interpreter, so a bare
# non-zero exit would make the job permanently red and therefore ignored.
#
# So the failures are matched individually against known-test-failures.txt and
# only unlisted ones count. The one thing that is never excused is the suite
# failing to finish: an interpreter crash produces no summary, and that is
# treated as failure regardless of what the exit code happened to be.

param(
    [Parameter(Mandatory = $true)][string] $ResultsFile,
    # Several files, so a platform can be given common.txt plus its own.
    [Parameter(Mandatory = $true)][string[]] $KnownFailuresFile,
    [string] $ExitCodeFile,
    [string] $Platform = $env:RUNNER_OS
)

$ErrorActionPreference = 'Stop'

function Add-Summary([string] $line) {
    Write-Host $line
    if ($env:GITHUB_STEP_SUMMARY) { $line | Out-File -FilePath $env:GITHUB_STEP_SUMMARY -Append }
}

if (-not (Test-Path $ResultsFile)) {
    Add-Summary "## ooRexx test suite"
    Add-Summary ''
    Add-Summary 'FAILED: the suite produced no output file at all.'
    exit 1
}

$text = Get-Content $ResultsFile -Raw
$lines = Get-Content $ResultsFile

$suiteExit = if ($ExitCodeFile -and (Test-Path $ExitCodeFile)) {
    (Get-Content $ExitCodeFile -Raw).Trim()
} else { 'unknown' }

# The summary block is the proof that the suite ran to completion.
$ran = [regex]::Match($text, '(?m)^Tests ran:\s+(\d+)')
$assertions = [regex]::Match($text, '(?m)^Assertions:\s+(\d+)')
$failures = [regex]::Match($text, '(?m)^Failures:\s+(\d+)')
$errors = [regex]::Match($text, '(?m)^Errors:\s+(\d+)')

$label = if ($Platform) { $Platform } else { 'unknown platform' }
Add-Summary "## ooRexx test suite - $label"
Add-Summary ''

if (-not $ran.Success) {
    Add-Summary "FAILED: the suite did not reach its summary, so it did not finish."
    Add-Summary ''
    Add-Summary "testOORexx exit code: ``$suiteExit``"
    $lastContainer = $lines | Where-Object { $_ -match '^Executing ' } | Select-Object -Last 1
    if ($lastContainer) { Add-Summary "Last container started: ``$lastContainer``" }
    Add-Summary ''
    Add-Summary 'Exit code -1073741819 (0xC0000005) is an access violation on Windows.'
    Add-Summary 'On a Unix host the shell reports a fatal signal as 128+N, so 139 is SIGSEGV'
    Add-Summary 'and 134 is SIGABRT.'
    exit 1
}

Add-Summary "| | |"
Add-Summary "|---|---|"
Add-Summary "| Tests ran | $($ran.Groups[1].Value) |"
if ($assertions.Success) { Add-Summary "| Assertions | $($assertions.Groups[1].Value) |" }
if ($failures.Success)   { Add-Summary "| Failures | $($failures.Groups[1].Value) |" }
if ($errors.Success)     { Add-Summary "| Errors | $($errors.Groups[1].Value) |" }
Add-Summary "| Exit code | $suiteExit |"
Add-Summary ''

# Lines starting a failure or error record, followed by indented Test:/Class:
# fields. Pair them up so a failure can be identified as CLASS/TEST.
$known = @{}
foreach ($file in $KnownFailuresFile) {
    if (-not (Test-Path $file)) {
        Add-Summary "FAILED: known failures file '$file' does not exist."
        exit 1
    }
    foreach ($line in (Get-Content $file)) {
        $trimmed = $line.Trim()
        if ($trimmed -eq '' -or $trimmed.StartsWith('#')) { continue }
        $known[$trimmed.ToLowerInvariant()] = $true
    }
}

$records = @()
for ($i = 0; $i -lt $lines.Count; $i++) {
    # Written as -match rather than -notmatch because only -match is guaranteed
    # to leave the captured groups in $Matches.
    if ($lines[$i] -match '^\[(failure|error)\]') { $kind = $Matches[1] } else { continue }
    $testName = $null
    $className = $null
    # The fields sit in the few lines directly after the marker.
    for ($j = $i + 1; $j -lt [Math]::Min($i + 8, $lines.Count); $j++) {
        if ($lines[$j] -match '^\s+Test:\s+(.+?)\s*$')  { $testName = $Matches[1] }
        if ($lines[$j] -match '^\s+Class:\s+(.+?)\s*$') { $className = $Matches[1] }
        if ($testName -and $className) { break }
    }
    if (-not $testName -or -not $className) { continue }
    $records += [pscustomobject]@{
        Kind = $kind
        Id   = "$className/$testName"
    }
}

$unexpected = @($records | Where-Object { -not $known.ContainsKey($_.Id.ToLowerInvariant()) })
$expected = @($records | Where-Object { $known.ContainsKey($_.Id.ToLowerInvariant()) })

if ($expected.Count -gt 0) {
    Add-Summary "$($expected.Count) known environmental failure(s), ignored:"
    Add-Summary ''
    foreach ($r in $expected) { Add-Summary "- ``$($r.Id)``" }
    Add-Summary ''
}

if ($unexpected.Count -gt 0) {
    Add-Summary "**$($unexpected.Count) unexpected failure(s):**"
    Add-Summary ''
    foreach ($r in $unexpected) { Add-Summary "- [$($r.Kind)] ``$($r.Id)``" }
    Add-Summary ''
    Add-Summary 'Fix the interpreter, or if this really is a property of the runner rather'
    Add-Summary 'than of ooRexx, add it to .github/known-test-failures.txt with the reason.'
    exit 1
}

# A record the parser could not read is not the same as no failures. Compare
# what was counted against what the suite itself reported so a change in the
# report format cannot quietly turn into a green build.
$reportedTotal = 0
if ($failures.Success) { $reportedTotal += [int]$failures.Groups[1].Value }
if ($errors.Success)   { $reportedTotal += [int]$errors.Groups[1].Value }
if ($reportedTotal -ne $records.Count) {
    Add-Summary "FAILED: the suite reported $reportedTotal failure(s) and error(s) but only $($records.Count) record(s) could be parsed."
    exit 1
}

Add-Summary 'No unexpected failures.'
exit 0
