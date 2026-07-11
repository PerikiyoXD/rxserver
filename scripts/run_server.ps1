#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Builds and starts rxserver.exe in the background, tracking its PID.
#>

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$PidFile = Join-Path $PSScriptRoot ".server.pid"
$LogFile = Join-Path $PSScriptRoot "server_trace.log"
$ExePath = Join-Path $RepoRoot "target\debug\rxserver.exe"

if (Test-Path $PidFile) {
    $existingPid = Get-Content $PidFile -ErrorAction SilentlyContinue
    if ($existingPid -and (Get-Process -Id $existingPid -ErrorAction SilentlyContinue)) {
        Write-Host "rxserver is already running (PID $existingPid)."
        exit 0
    }
}

Push-Location $RepoRoot
try {
    cargo build
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}

if (-not (Test-Path $ExePath)) {
    throw "rxserver.exe not found at $ExePath after build"
}

$env:RUST_LOG = if ($env:RUST_LOG) { $env:RUST_LOG } else { "trace" }

$process = Start-Process -FilePath $ExePath `
    -WorkingDirectory $RepoRoot `
    -RedirectStandardOutput $LogFile `
    -RedirectStandardError "$LogFile.err" `
    -PassThru `
    -WindowStyle Hidden

$process.Id | Out-File -FilePath $PidFile -Encoding ascii -NoNewline

Write-Host "rxserver started (PID $($process.Id)). Logs: $LogFile"
