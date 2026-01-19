#!/bin/bash
set -e

# BCPL Compiler Script (Node.js version)
# Compiles and runs a BCPL program using the Node.js INTCODE interpreter

if [ -z "$1" ]; then
	echo "Usage: $0 <source.b> [-iINPUT] [-oOUTPUT]"
	echo "Example: $0 test.b"
	exit 1
fi

if [ ! -f "$1" ]; then
	echo "Error: Source file '$1' not found"
	exit 1
fi

# Concatenate syni and trni
cat syni trni > synitrni

# Compile BCPL to OCODE
echo "Compiling $1 to OCODE..."
node icint.js synitrni -i$1

# Compile OCODE to INTCODE
echo "Compiling OCODE to INTCODE..."
node icint.js cgi -iOCODE

# Run INTCODE
echo "Running INTCODE..."
node icint.js INTCODE $2 $3
