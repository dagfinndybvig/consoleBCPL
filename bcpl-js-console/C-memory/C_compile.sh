#!/bin/sh
# Simple wrapper to use the compiled C_icint executable
echo "Using C_icint as executable"

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
./C_icint synitrni -i$1

# Compile OCODE to INTCODE
echo "Compiling OCODE to INTCODE..."
./C_icint cgi -iOCODE

# Run INTCODE
echo "Running INTCODE..."
./C_icint INTCODE $2 $3