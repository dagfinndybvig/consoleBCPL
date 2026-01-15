#!/bin/bash
set -e

# BCPL Compiler Script (Rust version)
# Compiles and runs a BCPL program using the Rust INTCODE interpreter

if [ -z "$1" ]; then
    echo "Usage: $0 <source.b>"
    echo "Example: $0 test.b"
    exit 1
fi

if [ ! -f "$1" ]; then
    echo "Error: Source file '$1' not found"
    exit 1
fi

# Concatenate syni and trni (stripping Z from trni)
cat syni > synitrni
tail -n +4 trni >> synitrni

# Compile BCPL to OCODE
echo "Compiling $1 to OCODE..."
./target/release/icint synitrni -i$1

# Compile OCODE to INTCODE
echo "Compiling OCODE to INTCODE..."
./target/release/icint cgi -iOCODE

# Run INTCODE
echo "Running INTCODE..."
./target/release/icint INTCODE $2

