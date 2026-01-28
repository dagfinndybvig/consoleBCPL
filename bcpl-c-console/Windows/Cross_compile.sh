#!/usr/bin/env bash
set -euo pipefail

# Default cross-compiler (64-bit Windows). Override with CC env var.
CC="${CC:-x86_64-w64-mingw32-gcc}"
# Resolve script directory and source path reliably
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC="$SCRIPT_DIR/../icint.c"
OUT="icint.exe"

if [ ! -f "$SRC" ]; then
  echo "Error: source $SRC not found. Run this from the bcpl-c-console directory."
  exit 1
fi

if ! command -v "$CC" >/dev/null 2>&1; then
  echo "Error: cross-compiler '$CC' not found in PATH."
  echo "Install mingw-w64 (package names: mingw-w64 or gcc-mingw-w64) or set CC to a valid cross-compiler."
  exit 1
fi

echo "Using CC=$CC"
"$CC" -O2 -o "$OUT" "$SRC"

echo "Built $OUT"
