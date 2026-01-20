@echo off
setlocal

REM BCPL Compiler Script (Rust version)
REM Compiles and runs a BCPL program using the Rust INTCODE interpreter

if "%~1"=="" (
    echo Usage: %~nx0 ^<source.b^>
    echo Example: %~nx0 test.b
    exit /b 1
)

if not exist "%~1" (
    echo Error: Source file "%~1" not found
    exit /b 1
)

REM Concatenate syni and trni
copy /b syni+trni synitrni >nul

REM Compile BCPL to OCODE
echo Compiling %~1 to OCODE...
."\target\x86_64-pc-windows-gnu\release\icint.exe" synitrni -i%~1

REM Compile OCODE to INTCODE
echo Compiling OCODE to INTCODE...
."\target\x86_64-pc-windows-gnu\release\icint.exe" cgi -iOCODE

REM Run INTCODE
echo Running INTCODE...
."\target\x86_64-pc-windows-gnu\release\icint.exe" INTCODE %2 %3

endlocal
