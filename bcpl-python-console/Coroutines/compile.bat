@echo off
setlocal
REM BCPL coroutine compiler/runner (Python version for Windows)
REM Usage: compile.bat <source.b> [-iINPUT] [-oOUTPUT]

set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%"

set "PYTHON_EXE=%PYTHON_BIN%"
if "%PYTHON_EXE%"=="" set "PYTHON_EXE=%PYTHON%"
if "%PYTHON_EXE%"=="" set "PYTHON_EXE=python"

if "%~1"=="" (
    echo Usage: %~nx0 ^<source.b^> [-iINPUT] [-oOUTPUT]
    echo Example: %~nx0 test3_callco.b
    exit /b 1
)

set "SRC=%~1"
set "SRC_PATH=%SRC%"
if not exist "%SRC_PATH%" set "SRC_PATH=%SCRIPT_DIR%%SRC%"

if not exist "%SRC_PATH%" (
    echo Error: Source file '%SRC%' not found
    exit /b 1
)

REM Front-end pass
copy /y /b syni+trni synitrni >nul

echo --- Compiling %SRC% to OCODE ---
"%PYTHON_EXE%" icint_co.py synitrni -i"%SRC_PATH%"
if errorlevel 1 exit /b 1

REM Back-end pass
echo --- Compiling OCODE to INTCODE ---
"%PYTHON_EXE%" icint_co.py cgi -iOCODE
if errorlevel 1 exit /b 1

REM Link coroutine runtime
echo --- Linking with CORLIB ---
copy /y /b INTCODE+CORLIB RUNABLE >nul
if errorlevel 1 exit /b 1

REM Run linked program
echo --- Running RUNABLE ---
"%PYTHON_EXE%" icint_co.py RUNABLE %2 %3

endlocal
