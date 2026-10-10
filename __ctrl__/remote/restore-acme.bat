@echo off
setlocal
cd /d "%~dp0..\.."
for %%I in ("%~dp0..\..") do set "PROJECT=%%~nxI"
set "BACKUP_DIR=%~dp0..\..\..\.foxg-ssl-backups\%PROJECT%"
set "ACME=letsencrypt\acme.json"
set "BACKUP=%BACKUP_DIR%\acme.json"

if not exist "%BACKUP%" (
  echo No backup at %BACKUP%
  exit /b 1
)
for %%I in ("%BACKUP%") do if %%~zI EQU 0 (
  echo Backup is empty at %BACKUP%
  exit /b 1
)

if not exist letsencrypt mkdir letsencrypt
copy /Y "%BACKUP%" "%ACME%" >nul
echo Restored SSL from %BACKUP% to %ACME%
echo If proxy is running: docker compose up -d proxy
endlocal
