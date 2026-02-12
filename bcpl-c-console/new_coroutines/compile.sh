#!/bin/sh
# Build and run script for new_coroutines
# Usage: ./compile.sh <source.b> [-iINPUT] [-oOUTPUT]
#
# First run builds icint from C source.
# Then compiles BCPL source to OCODE, OCODE to INTCODE, and runs INTCODE.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PARENT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Build the interpreter if not already built
if [ ! -f "$SCRIPT_DIR/icint" ]; then
    echo "Building icint interpreter..."
    gcc -O2 -o "$SCRIPT_DIR/icint" "$SCRIPT_DIR/icint.c" 2>&1
    echo "Built icint successfully."
fi

# Copy compiler toolchain files from parent if not present
for f in syni trni cgi; do
    if [ ! -f "$SCRIPT_DIR/$f" ]; then
        if [ -f "$PARENT_DIR/$f" ]; then
            cp "$PARENT_DIR/$f" "$SCRIPT_DIR/$f"
        else
            echo "Error: Cannot find $f in parent directory $PARENT_DIR"
            exit 1
        fi
    fi
done

if [ -z "$1" ]; then
    echo "Usage: $0 <source.b> [-iINPUT] [-oOUTPUT]"
    echo "Example: $0 test3_callco.b"
    exit 1
fi

if [ ! -f "$1" ]; then
    echo "Error: Source file '$1' not found"
    exit 1
fi

# Concatenate syni and trni for the compiler front-end
cat "$SCRIPT_DIR/syni" "$SCRIPT_DIR/trni" > "$SCRIPT_DIR/synitrni"

# Step 1: Compile BCPL to OCODE (NOTE: COROUTINE SUPPORT)
echo "--- Compiling $1 with COROUTINE SUPPORT ENABLED ---\n"
echo "First to OCODE ..."
"$SCRIPT_DIR/icint" "$SCRIPT_DIR/synitrni" -i"$1"

# Step 2: Compile OCODE to INTCODE
echo "--- Compiling OCODE to INTCODE ---"
"$SCRIPT_DIR/icint" "$SCRIPT_DIR/cgi" -iOCODE

# Step 2.5: Link with CORLIB for coroutine support
echo "--- Linking with CORLIB ---"
cat INTCODE CORLIB > RUNABLE

# Step 3: Run INTCODE (pass remaining args)
echo "--- Running RUNABLE ---"
"$SCRIPT_DIR/icint" RUNABLE $2 $3
echo ""
echo "--- Done ---"
