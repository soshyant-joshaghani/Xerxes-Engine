@echo off
setlocal
rem Usage: start-prod.bat [--slim]   (--slim: no Redis and no worker)
cd /d "%~dp0..\.."
set SLIM=0
if /i "%~1"=="--slim" set SLIM=1
if not exist .env copy /Y .env.example .env >nul 2>&1
if not exist letsencrypt mkdir letsencrypt
if not exist letsencrypt\acme.json type nul > letsencrypt\acme.json
docker network inspect traefik-public >nul 2>&1 || docker network create traefik-public
if "%SLIM%"=="1" (
  docker compose build backend
  if errorlevel 1 exit /b 1
  docker compose up -d --pull never --no-deps db backend adminer proxy
) else (
  docker compose up -d --build
)
if errorlevel 1 exit /b 1
echo.
if "%SLIM%"=="1" (echo xerxes production stack started ^(slim: no Redis, no worker^).) else (echo xerxes production stack started.)
echo Update DOMAIN in compose.yml, ACME email in compose.traefik.yml, and DNS before going live.
endlocal
