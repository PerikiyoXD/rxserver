#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Launches Cygwin xeyes against a running rxserver (DISPLAY 127.0.0.1:0).
#>

$ErrorActionPreference = "Stop"

$CygwinBash = "D:/cygwin64/bin/bash.exe"
$PidFile = Join-Path $PSScriptRoot ".xeyes.pid"
$LogFile = Join-Path $PSScriptRoot "xeyes.log"

if (-not (Test-Path $CygwinBash)) {
    throw "Cygwin bash not found at $CygwinBash"
}

if (Test-Path $PidFile) {
    $existingPid = Get-Content $PidFile -ErrorAction SilentlyContinue
    if ($existingPid -and (Get-Process -Id $existingPid -ErrorAction SilentlyContinue)) {
        Write-Host "xeyes is already running (PID $existingPid)."
        exit 0
    }
}

# Launch xeyes in Cygwin, capture its background PID via $!, and write that
# PID back out to a file Windows can read - Cygwin PIDs and Windows PIDs
# diverge, but xeyes forked directly off bash --login -c is 1:1 with the
# Windows process, so this round-trip is reliable.
$cygwinPidFile = "/tmp/rxserver_xeyes.pid"
$cmd = "DISPLAY=127.0.0.1:0 /usr/bin/xeyes > /tmp/rxserver_xeyes.log 2>&1 & echo `$! > $cygwinPidFile"
& $CygwinBash --login -c $cmd | Out-Null

Start-Sleep -Milliseconds 500

& $CygwinBash --login -c "cat /tmp/rxserver_xeyes.log 2>/dev/null" | Out-File -FilePath $LogFile -Encoding utf8

$xeyesProcess = Get-Process -Name "xeyes" -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $xeyesProcess) {
    throw "xeyes exited immediately after launch (likely rejected by the X server) - see $LogFile"
}

$xeyesProcess.Id | Out-File -FilePath $PidFile -Encoding ascii -NoNewline

Write-Host "xeyes started (PID $($xeyesProcess.Id))."
