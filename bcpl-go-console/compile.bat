@echo off
setlocal

if not exist "%SystemRoot%\System32\bash.exe" (
  echo Error: bash.exe not found. Install WSL or run compile.sh directly.
  exit /b 1
)

set "WSL_DIR=%~dp0"
set "WSL_DIR=%WSL_DIR:\=/%"
set "WSL_DIR=%WSL_DIR:C:=/mnt/c%"
if "%WSL_DIR:~-1%"=="/" set "WSL_DIR=%WSL_DIR:~0,-1%"

bash -lc "cd '%WSL_DIR%' && ./compile.sh %*"
exit /b %ERRORLEVEL%
