@echo off
cd /d "%~dp0"
echo Starting local Rust API. Ensure the development PostgreSQL database is available.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\start-dev-api.ps1"
pause