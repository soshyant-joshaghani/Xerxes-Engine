@echo off
setlocal
cd /d "%~dp0..\.."
for %%I in ("%~dp0..\..") do set "PROJECT=%%~nxI"
set "BACKUP_DIR=%~dp0..\..\..\.foxg-ssl-backups\%PROJECT%"
set "ACME=letsencrypt\acme.json"
set "BACKUP=%BACKUP_DIR%\acme.json"

if not exist "%ACME%" (
  echo No local %ACME% — nothing to back up.
  exit /b 1
)
for %%I in ("%ACME%") do if %%~zI EQU 0 (
  echo Local acme.json is empty — skipping backup.
  exit /b 1
)

if not exist "%BACKUP_DIR%" mkdir "%BACKUP_DIR%"
copy /Y "%ACME%" "%BACKUP%" >nul
echo Backed up SSL to %BACKUP%
endlocal
