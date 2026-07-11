#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Stops xeyes if it was started via run_xeyes.ps1.
#>

$ErrorActionPreference = "Stop"

$PidFile = Join-Path $PSScriptRoot ".xeyes.pid"

if (-not (Test-Path $PidFile)) {
    Write-Host "xeyes is not running (no PID file)."
    exit 0
}

$trackedPid = Get-Content $PidFile -ErrorAction SilentlyContinue
$process = if ($trackedPid) { Get-Process -Id $trackedPid -ErrorAction SilentlyContinue } else { $null }

if ($process -and $process.ProcessName -eq "xeyes") {
    Stop-Process -Id $trackedPid -Force -Confirm:$false
    Write-Host "Stopped xeyes (PID $trackedPid)."
}
else {
    Write-Host "xeyes was not running (stale PID file)."
}

Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
