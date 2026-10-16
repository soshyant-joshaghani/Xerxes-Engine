@echo off
setlocal
cd /d "%~dp0..\.."
call "%~dp0backup-acme.bat"
if errorlevel 1 echo No SSL to back up — continuing...
echo.
echo Pruning Docker build cache...
docker builder prune -af
if errorlevel 1 exit /b 1
echo.
echo Build cache cleared.
for %%I in ("%~dp0..\..") do set "PROJECT=%%~nxI"
echo SSL backup: ..\..\..\.foxg-ssl-backups\%PROJECT%\acme.json
echo Restore:    xerxes-ctrl.bat prod restore-acme
endlocal
