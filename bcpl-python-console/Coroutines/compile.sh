#!/usr/bin/env bash
set -euo pipefail

# BCPL coroutine compiler/runner (Python version)
# Usage: ./compile.sh <source.b> [-iINPUT] [-oOUTPUT]

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PYTHON_BIN="${PYTHON_BIN:-python3}"
ICINT_CO="icint_co.py"

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <source.b> [-iINPUT] [-oOUTPUT]"
  echo "Example: $0 test3_callco.b"
  exit 1
fi

SRC="$1"
shift || true

if [[ ! -f "$SRC" && ! -f "$SCRIPT_DIR/$SRC" ]]; then
  echo "Error: Source file '$SRC' not found"
  exit 1
fi

if [[ -f "$SRC" ]]; then
  SRC_PATH="$SRC"
else
  SRC_PATH="$SCRIPT_DIR/$SRC"
fi

(
  cd "$SCRIPT_DIR"

  # Front-end pass
  cat syni trni > synitrni

  echo "--- Compiling $SRC to OCODE ---"
  "$PYTHON_BIN" "$ICINT_CO" synitrni -i"$SRC_PATH"

  # Back-end pass
  echo "--- Compiling OCODE to INTCODE ---"
  "$PYTHON_BIN" "$ICINT_CO" cgi -iOCODE

  # Link coroutine runtime
  echo "--- Linking with CORLIB ---"
  cat INTCODE CORLIB > RUNABLE

  # Run linked program
  echo "--- Running RUNABLE ---"
  "$PYTHON_BIN" "$ICINT_CO" RUNABLE "$@"
)
