#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Reports whether rxserver.exe (started via run_server.ps1) is running.
    Exits 0 and prints the PID if running, exits 1 otherwise.
#>

$ErrorActionPreference = "Stop"

$PidFile = Join-Path $PSScriptRoot ".server.pid"

if (-not (Test-Path $PidFile)) {
    Write-Host "rxserver is not running (no PID file)."
    exit 1
}

$trackedPid = Get-Content $PidFile -ErrorAction SilentlyContinue
$process = if ($trackedPid) { Get-Process -Id $trackedPid -ErrorAction SilentlyContinue } else { $null }

if ($process -and $process.ProcessName -eq "rxserver") {
    Write-Host "rxserver is running (PID $trackedPid)."
    exit 0
}
else {
    Write-Host "rxserver is not running (stale PID file)."
    exit 1
}
