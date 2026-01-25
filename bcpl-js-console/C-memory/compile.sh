#!/bin/bash
set -e

# Compile and run BCPL programs using the C-memory interpreter (C and JS fallbacks)
# Announces building with extended memory functions

ROOT_DIR="$(cd ""$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$ROOT_DIR/.." && pwd)"

if [ -z "$1" ]; then
  echo "Usage: $0 <source.b> [-iINPUT] [-oOUTPUT]"
  echo "Example: $0 vec.b"
  exit 1
fi

SRC="$1"
shift

echo "Building with extended memory functions..."

# Locate helper files
SYN="$REPO_ROOT/syni"
TRN="$REPO_ROOT/trni"
CGI="$REPO_ROOT/cgi"

# Fallback lookups in parent dir
[ -f "$SYN" ] || SYN="$ROOT_DIR/../syni"
[ -f "$TRN" ] || TRN="$ROOT_DIR/../trni"
[ -f "$CGI" ] || CGI="$ROOT_DIR/../cgi"

# Prepare synitrni
cat "$SYN" "$TRN" > "$ROOT_DIR/synitrni"

# Try to compile the C interpreter (icint.c) if gcc/clang is available
ICINT_BIN=""
CC=\