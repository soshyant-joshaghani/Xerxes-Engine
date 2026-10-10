@echo off
setlocal EnableDelayedExpansion
cd /d "%~dp0..\.."

if not exist letsencrypt mkdir letsencrypt
if not exist letsencrypt\acme.json type nul > letsencrypt\acme.json

if exist letsencrypt\acme.json (
  for %%I in (letsencrypt\acme.json) do if %%~zI GTR 0 (
    set "TS=%date:~-4%%date:~4,2%%date:~7,2%-%time:~0,2%%time:~3,2%%time:~6,2%"
    set "TS=!TS: =0!"
    copy /Y letsencrypt\acme.json "letsencrypt\acme.json.bak.!TS!" >nul
    echo Backed up SSL state to letsencrypt\acme.json.bak.!TS!
  )
)

echo Stopping production stack...
docker compose down --remove-orphans
if errorlevel 1 exit /b 1

echo Removing Postgres and Redis data volumes...
for %%V in (xerxes_db-data xerxes_redis-data) do (
  docker volume inspect %%V >nul 2>&1 && (
    docker volume rm %%V
    echo   removed %%V
  )
)

echo Removing application images (will rebuild on next start)...
for %%I in (xerxes-backend:prod) do (
  docker image inspect %%I >nul 2>&1 && (
    docker rmi %%I
    echo   removed %%I
  )
)

docker image prune -f >nul 2>&1

echo.
echo Reset complete.
echo   Kept: traefik:3.6, postgres:18, redis:8-alpine, adminer images
echo   Kept: .\letsencrypt\acme.json
echo   Wiped: db-data, redis-data, app :prod images
echo.
echo Start fresh: xerxes-ctrl.bat backend run prod
endlocal
