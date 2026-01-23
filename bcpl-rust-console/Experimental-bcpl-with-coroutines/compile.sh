#!/bin/bash
set -e

# BCPL Compiler Script (Rust coroutines version)
# Compiles and runs a BCPL program using the coroutine-enabled Rust INTCODE interpreter

if [ -z "$1" ]; then
    echo "Usage: $0 <source.b>"
    echo "Example: $0 test.b"
    exit 1
fi

if [ ! -f "$1" ]; then
    echo "Error: Source file '$1' not found"
    exit 1
fi

ROOT_DIR=$(cd "$(dirname "$0")" && pwd)
ICINT="$ROOT_DIR/target/release/icint"
TMP_DIR="$ROOT_DIR/.crlf-build"

SYN="$ROOT_DIR/syni"
TRN="$ROOT_DIR/trni"
CGI="$ROOT_DIR/cgi"

if [ ! -f "$SYN" ]; then
    SYN="$ROOT_DIR/../syni"
fi
if [ ! -f "$TRN" ]; then
    TRN="$ROOT_DIR/../trni"
fi
if [ ! -f "$CGI" ]; then
    CGI="$ROOT_DIR/../cgi"
fi

mkdir -p "$TMP_DIR"

to_crlf() {
    awk '{printf "%s\r", $0}' "$1" > "$2"
}

to_crlf "$ROOT_DIR/libhdr" "$TMP_DIR/libhdr"
if [ -f "$ROOT_DIR/coroutines" ]; then
    to_crlf "$ROOT_DIR/coroutines" "$TMP_DIR/coroutines"
fi

to_crlf "$1" "$TMP_DIR/source.b"

if [ -f "$ROOT_DIR/input.txt" ]; then
    cp "$ROOT_DIR/input.txt" "$TMP_DIR/input.txt"
fi

pushd "$TMP_DIR" >/dev/null

# Concatenate syni and trni
cat "$SYN" "$TRN" > "synitrni"

# Compile BCPL to OCODE
echo "Compiling $1 to OCODE..."
"$ICINT" "$ROOT_DIR/synitrni" -i"$TMP_DIR/source.b"

# Compile OCODE to INTCODE
echo "Compiling OCODE to INTCODE..."
"$ICINT" "$CGI" -i"$TMP_DIR/OCODE"

# Run INTCODE
echo "Running INTCODE..."
timeout 10s "$ICINT" "$TMP_DIR/INTCODE" $2 $3

if [ -f "$TMP_DIR/output.txt" ]; then
    cp "$TMP_DIR/output.txt" "$ROOT_DIR/output.txt"
fi

popd >/dev/null
