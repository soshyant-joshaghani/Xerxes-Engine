@echo off
setlocal
cd /d "%~dp0..\.."
docker compose down --remove-orphans
if errorlevel 1 exit /b 1
echo.
echo Production stack stopped.
echo Certificates: .\letsencrypt\acme.json (unchanged)
echo DB/Redis data: still on Docker volumes (use reset-prod.bat to wipe)
endlocal
