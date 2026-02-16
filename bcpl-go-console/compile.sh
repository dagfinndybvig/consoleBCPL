#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GO_BIN="${GO_BIN:-go}"
ICINT_BIN="${ICINT_BIN:-$SCRIPT_DIR/icint}"
if ! command -v "$GO_BIN" >/dev/null 2>&1; then
  if [[ -x "/mnt/c/Program Files/Go/bin/go.exe" ]]; then
    GO_BIN="/mnt/c/Program Files/Go/bin/go.exe"
  fi
fi

if [[ $# -lt 1 ]]; then
  echo "Usage: ./compile.sh <source.b> [--coro] [runtime args]"
  exit 1
fi

SRC="$1"
shift || true
USE_CORO=0
if [[ "${1:-}" == "--coro" ]]; then
  USE_CORO=1
  shift || true
fi

if [[ ! -f "$SCRIPT_DIR/$SRC" && ! -f "$SRC" ]]; then
  echo "Error: Source file '$SRC' not found"
  exit 1
fi

cd "$SCRIPT_DIR"

cat syni trni > synitrni

echo "--- Compiling $SRC (BCPL -> OCODE) ---"
"$ICINT_BIN" synitrni -i"$SRC"

echo "--- Compiling OCODE (OCODE -> INTCODE) ---"
"$ICINT_BIN" cgi -iOCODE

if [[ $USE_CORO -eq 1 ]]; then
  echo "--- Linking coroutine library (INTCODE + CORLIB -> RUNABLE) ---"
  cat INTCODE CORLIB > RUNABLE
  echo "--- Running RUNABLE ---"
  "$GO_BIN" run . RUNABLE "$@"
else
  echo "--- Running INTCODE ---"
  "$GO_BIN" run . INTCODE "$@"
fi
