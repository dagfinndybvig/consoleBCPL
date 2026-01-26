#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "Building C interpreter..."
gcc -O2 -std=c99 -Wall -Wextra -o C_icint icint.c

FILES=(fact.b coro_cps.b corns.b)
FAIL=0
for f in "${FILES[@]}"; do
  echo "\n=== Testing $f ==="

  # Prepare files
  cat syni trni > synitrni

  # Run C interpreter steps directly, capture only the final runtime output
  ./C_icint synitrni -i"$f" >/tmp/c_step1.txt 2>&1 || true
  ./C_icint cgi -iOCODE >/tmp/c_step2.txt 2>&1 || true
  ./C_icint INTCODE > /tmp/out_c.txt 2>&1 || true

  # Run Node interpreter steps and capture runtime output
  node icint.js synitrni -i"$f" >/tmp/js_step1.txt 2>&1 || true
  node icint.js cgi -iOCODE >/tmp/js_step2.txt 2>&1 || true
  node icint.js INTCODE > /tmp/out_js.txt 2>&1 || true

  if diff -u /tmp/out_c.txt /tmp/out_js.txt >/dev/null; then
    echo "[OK] $f: outputs match"
  else
    echo "[FAIL] $f: outputs differ"
    echo "--- C output ---"
    sed -n '1,200p' /tmp/out_c.txt
    echo "--- JS output ---"
    sed -n '1,200p' /tmp/out_js.txt
    FAIL=1
  fi
done

if [ "$FAIL" -ne 0 ]; then
  echo "Some tests failed"
  exit 2
else
  echo "All tests passed"
fi
