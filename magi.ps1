<#
.SYNOPSIS
    MAGI System CLI Runner for Windows (Docker-powered)

.DESCRIPTION
    Executes the MAGI System CLI, NERV TUI, and Trinity multi-agent consensus engine
    inside Docker without requiring a local Rust/C++ toolchain installed on Windows.
    Automatically manages the SpacetimeDB container and persists reports and state
    directly to host disk.

.EXAMPLE
    .\magi.ps1 status
    .\magi.ps1 idea docs\IDEA.md
    .\magi.ps1 maintain client\src\main.rs -g docs\guides\OPERATIONS.md
    .\magi.ps1 triage "panic in connection pool"
    .\magi.ps1 history
    .\magi.ps1 console
    .\magi.ps1 up
    .\magi.ps1 down
#>

param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)

$ErrorActionPreference = "Stop"

# Always ensure working directory is the script root where docker-compose.yml resides
if ($PSScriptRoot) {
    Set-Location $PSScriptRoot
}

function Write-NervBanner {
    Write-Host "NERV // MAGI SYSTEM DOCKER ORCHESTRATOR" -ForegroundColor DarkYellow
}

function Ensure-SpacetimeDB {
    $running = docker ps --filter "name=magi-spacetimedb" --filter "status=running" -q
    if (-not $running) {
        Write-Host "▶ SpacetimeDB engine is not running. Starting container..." -ForegroundColor Cyan
        docker compose up -d spacetimedb | Out-Null
        Start-Sleep -Seconds 2
    }
}

# If no arguments provided, launch interactive session / help
if (-not $Arguments -or $Arguments.Length -eq 0) {
    Ensure-SpacetimeDB
    docker compose run --rm magi
    exit $LASTEXITCODE
}

$firstArg = $Arguments[0].ToLowerInvariant()

switch ($firstArg) {
    "up" {
        Write-Host "▶ Starting MAGI SpacetimeDB container in background..." -ForegroundColor Cyan
        docker compose up -d spacetimedb
        Start-Sleep -Seconds 2
        docker compose ps
        exit $LASTEXITCODE
    }

    "down" {
        Write-Host "▶ Stopping MAGI containers..." -ForegroundColor Yellow
        docker compose down
        exit $LASTEXITCODE
    }

    "restart" {
        Write-Host "▶ Restarting SpacetimeDB..." -ForegroundColor Yellow
        docker compose restart spacetimedb
        exit $LASTEXITCODE
    }

    "logs" {
        docker compose logs -f spacetimedb
        exit $LASTEXITCODE
    }

    "build" {
        Ensure-SpacetimeDB
        Write-Host "▶ Building MAGI client and server WASM inside container..." -ForegroundColor Cyan
        docker compose run --rm magi cargo build --bin magi
        docker compose run --rm magi cargo build -p magi-server --target wasm32-unknown-unknown
        Write-Host "▶ Publishing magi-server module to SpacetimeDB..." -ForegroundColor Cyan
        docker compose exec spacetimedb spacetime publish -y --server http://127.0.0.1:3000 --bin-path /target/wasm32-unknown-unknown/debug/magi_server.wasm magi-system
        exit $LASTEXITCODE
    }

    "check" {
        Write-Host "▶ Running cargo check inside container..." -ForegroundColor Cyan
        docker compose run --rm magi cargo check --workspace
        exit $LASTEXITCODE
    }

    "test" {
        Write-Host "▶ Running workspace tests inside container..." -ForegroundColor Cyan
        docker compose run --rm magi cargo test --workspace
        exit $LASTEXITCODE
    }

    "bash" {
        Ensure-SpacetimeDB
        docker compose run --rm magi bash
        exit $LASTEXITCODE
    }

    "sh" {
        Ensure-SpacetimeDB
        docker compose run --rm magi sh
        exit $LASTEXITCODE
    }

    "mcp" {
        docker compose run -i -T --rm magi mcp
        exit $LASTEXITCODE
    }

    default {
        Ensure-SpacetimeDB

        # Normalize Windows backslashes in paths for Linux container compatibility
        $normalizedArgs = @()
        foreach ($arg in $Arguments) {
            # If the argument looks like a relative or absolute path, convert \ to /
            if ($arg -match '\\' -and ($arg -match '\.(rs|md|toml|json|txt|ya?ml|log)$' -or (Test-Path $arg -ErrorAction SilentlyContinue))) {
                $normalizedArgs += $arg.Replace('\', '/')
            } else {
                $normalizedArgs += $arg
            }
        }

        docker compose run --rm magi @normalizedArgs
        exit $LASTEXITCODE
    }
}
