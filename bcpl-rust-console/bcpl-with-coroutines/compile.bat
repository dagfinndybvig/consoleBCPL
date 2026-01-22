@echo off
setlocal
set "ROOT=%~dp0"
pushd "%ROOT%" >nul

REM BCPL Compiler Script (Rust coroutines version)
REM Compiles and runs a BCPL program using the coroutine-enabled Rust INTCODE interpreter

if "%~1"=="" (
    echo Usage: %~nx0 ^<source.b^>
    echo Example: %~nx0 test_coroutines_min.b
    exit /b 1
)

if not exist "%~1" (
    echo Error: Source file "%~1" not found
    exit /b 1
)

REM Concatenate syni and trni
set "ICINT=.\target\x86_64-pc-windows-gnu\release\icint.exe"
set "SYN=.\syni"
set "TRN=.\trni"
set "CGI=.\cgi"

if not exist "%SYN%" set "SYN=..\syni"
if not exist "%TRN%" set "TRN=..\trni"
if not exist "%CGI%" set "CGI=..\cgi"

if not exist "%ICINT%" (
    echo Error: icint.exe not found at "%ICINT%"
    exit /b 1
)
if not exist "%SYN%" (
    echo Error: syni not found
    exit /b 1
)
if not exist "%TRN%" (
    echo Error: trni not found
    exit /b 1
)
if not exist "%CGI%" (
    echo Error: cgi not found
    exit /b 1
)

REM Concatenate syni and trni
copy /b "%SYN%"+"%TRN%" synitrni >nul

REM Compile BCPL to OCODE
echo Compiling %~1 to OCODE...
"%ICINT%" synitrni -i%~1

REM Compile OCODE to INTCODE
echo Compiling OCODE to INTCODE...
"%ICINT%" "%CGI%" -iOCODE

REM Run INTCODE
echo Running INTCODE...
"%ICINT%" INTCODE %2 %3

popd >nul
endlocal
