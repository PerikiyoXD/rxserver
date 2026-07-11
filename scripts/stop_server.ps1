#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Stops rxserver.exe if it was started via run_server.ps1.
#>

$ErrorActionPreference = "Stop"

$PidFile = Join-Path $PSScriptRoot ".server.pid"

if (-not (Test-Path $PidFile)) {
    Write-Host "rxserver is not running (no PID file)."
    exit 0
}

$trackedPid = Get-Content $PidFile -ErrorAction SilentlyContinue
$process = if ($trackedPid) { Get-Process -Id $trackedPid -ErrorAction SilentlyContinue } else { $null }

if ($process -and $process.ProcessName -eq "rxserver") {
    Stop-Process -Id $trackedPid -Force -Confirm:$false
    Write-Host "Stopped rxserver (PID $trackedPid)."
}
else {
    Write-Host "rxserver was not running (stale PID file)."
}

Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
