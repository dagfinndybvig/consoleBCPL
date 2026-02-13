@echo off
setlocal EnableExtensions EnableDelayedExpansion

set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%"

set "PYTHON_EXE=%PYTHON_BIN%"
if "%PYTHON_EXE%"=="" set "PYTHON_EXE=%PYTHON%"
if "%PYTHON_EXE%"=="" set "PYTHON_EXE=python"

if not exist "icint_co.py" (
  echo Missing interpreter: %SCRIPT_DIR%icint_co.py
  exit /b 1
)

echo Running coroutine parity checks with %PYTHON_EXE% icint_co.py
echo.

echo ===== TEST2 : python icint_co =====
"%PYTHON_EXE%" icint_co.py TEST2
if errorlevel 1 (
  echo TEST2 failed.
  exit /b 1
)
echo.

for %%t in (test3_callco.b test4_multi.b test5_resumeco.b test6_deleteco.b) do (
  echo ===== %%t : compile =====
  call compile.bat %%t > "%TEMP%\compile_%%t.log" 2>&1
  if errorlevel 1 (
    echo COMPILE_FAIL %%t
    type "%TEMP%\compile_%%t.log"
    exit /b 1
  )

  echo ===== %%t : python icint_co ^(RUNABLE^) =====
  "%PYTHON_EXE%" icint_co.py RUNABLE > "%TEMP%\py_%%t.out" 2> "%TEMP%\py_%%t.err"
  if errorlevel 1 (
    echo PY_FAIL %%t
    type "%TEMP%\py_%%t.out"
    type "%TEMP%\py_%%t.err"
    exit /b 1
  )

  type "%TEMP%\py_%%t.out"
  echo.
)

echo All coroutine tests passed.
exit /b 0
