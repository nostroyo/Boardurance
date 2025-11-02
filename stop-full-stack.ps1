#!/usr/bin/env pwsh

# Full Stack Stop Script - Web3 Racing Game
# Stops React Frontend, Rust Backend, and optionally Docker containers

param(
    [switch]$KeepDocker,
    [switch]$Verbose
)

Write-Host "🛑 Stopping Web3 Racing Game Full Stack" -ForegroundColor Red
Write-Host "=======================================" -ForegroundColor Red

$ErrorActionPreference = "Continue"

try {
    # Step 1: Stop Frontend (Node/npm processes)
    Write-Host "`n⚛️  Stopping React Frontend..." -ForegroundColor Yellow
    $nodeProcesses = Get-Process | Where-Object {$_.ProcessName -eq "node" -or $_.ProcessName -eq "npm"}
    if ($nodeProcesses) {
        $nodeProcesses | ForEach-Object {
            if ($Verbose) {
                Write-Host "   Stopping process: $($_.ProcessName) (PID: $($_.Id))" -ForegroundColor Gray
            }
            Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
        }
        Write-Host "✅ Frontend processes stopped" -ForegroundColor Green
    } else {
        Write-Host "ℹ️  No frontend processes found" -ForegroundColor Gray
    }

    # Step 2: Stop Backend (Cargo/Rust processes)
    Write-Host "`n🦀 Stopping Rust Backend..." -ForegroundColor Yellow
    $cargoProcesses = Get-Process | Where-Object {$_.ProcessName -eq "cargo" -or $_.ProcessName -eq "rust-backend"}
    if ($cargoProcesses) {
        $cargoProcesses | ForEach-Object {
            if ($Verbose) {
                Write-Host "   Stopping process: $($_.ProcessName) (PID: $($_.Id))" -ForegroundColor Gray
            }
            Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
        }
        Write-Host "✅ Backend processes stopped" -ForegroundColor Green
    } else {
        Write-Host "ℹ️  No backend processes found" -ForegroundColor Gray
    }

    # Step 3: Stop Docker containers (optional)
    if (-not $KeepDocker) {
        Write-Host "`n🐳 Stopping Docker containers..." -ForegroundColor Yellow
        
        try {
            # Check if Docker is running
            docker version | Out-Null
            
            # Stop MongoDB containers
            Set-Location "rust-backend"
            & .\scripts\stop-mongodb.ps1
            Set-Location ".."
            
            Write-Host "✅ Docker containers stopped" -ForegroundColor Green
        } catch {
            Write-Host "ℹ️  Docker not running or containers already stopped" -ForegroundColor Gray
        }
    } else {
        Write-Host "⏭️  Keeping Docker containers running (--KeepDocker flag)" -ForegroundColor Gray
    }

    # Step 4: Clean up any remaining processes on our ports
    Write-Host "`n🧹 Cleaning up ports..." -ForegroundColor Yellow
    
    # Check and kill processes on port 3000 (backend)
    try {
        $port3000 = netstat -ano | Select-String ":3000.*LISTENING"
        if ($port3000) {
            $pid = ($port3000 -split '\s+')[-1]
            if ($pid -and $pid -ne "0") {
                Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue
                Write-Host "   Freed port 3000" -ForegroundColor Gray
            }
        }
    } catch { }
    
    # Check and kill processes on port 5173 (frontend)
    try {
        $port5173 = netstat -ano | Select-String ":5173.*LISTENING"
        if ($port5173) {
            $pid = ($port5173 -split '\s+')[-1]
            if ($pid -and $pid -ne "0") {
                Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue
                Write-Host "   Freed port 5173" -ForegroundColor Gray
            }
        }
    } catch { }

    Write-Host "✅ Port cleanup complete" -ForegroundColor Green

    # Success Summary
    Write-Host "`n✅ Full Stack Stop Complete!" -ForegroundColor Green
    Write-Host "============================" -ForegroundColor Green
    Write-Host ""
    Write-Host "🔗 Stopped Services:" -ForegroundColor Cyan
    Write-Host "   ⚛️  React Frontend (port 5173)" -ForegroundColor White
    Write-Host "   🦀 Rust Backend (port 3000)" -ForegroundColor White
    if (-not $KeepDocker) {
        Write-Host "   🐳 Docker containers (MongoDB)" -ForegroundColor White
    }
    Write-Host ""
    Write-Host "🚀 To restart everything:" -ForegroundColor Yellow
    Write-Host "   .\start-full-stack.ps1" -ForegroundColor White
    Write-Host ""
    Write-Host "🐳 To restart with existing Docker containers:" -ForegroundColor Yellow
    Write-Host "   .\start-full-stack.ps1 -SkipDocker" -ForegroundColor White

} catch {
    Write-Host "`n❌ Stop failed: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "💡 You may need to manually stop processes or restart your terminal" -ForegroundColor Yellow
    exit 1
}

Write-Host "`n🎯 All services stopped successfully!" -ForegroundColor Green