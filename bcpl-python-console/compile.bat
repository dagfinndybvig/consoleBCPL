@echo off
REM BCPL Compiler Script (Python version for Windows)
REM Compiles and runs a BCPL program using the Python INTCODE interpreter
REM
REM NOTE: For better performance, use PyPy instead of CPython:
REM   set PYTHON=pypy3
REM   compile.bat test.b

if "%PYTHON%"=="" set PYTHON=python

if "%1"=="" (
    echo Usage: %0 ^<source.b^>
    echo Example: %0 test.b
    echo For better performance: set PYTHON=pypy3 ^&^& %0 test.b
    exit /b 1
)

if not exist "%1" (
    echo Error: Source file '%1' not found
    exit /b 1
)

REM Concatenate syni and trni (stripping first 3 lines from trni)
copy /y syni synitrni >nul
more +3 trni >> synitrni

REM Compile BCPL to OCODE
echo Compiling %1 to OCODE...
%PYTHON% icint.py synitrni -i%1
if errorlevel 1 exit /b 1

REM Compile OCODE to INTCODE
echo Compiling OCODE to INTCODE...
%PYTHON% icint.py cgi -iOCODE
if errorlevel 1 exit /b 1

REM Run INTCODE
echo Running INTCODE...
%PYTHON% icint.py INTCODE %2 %3
