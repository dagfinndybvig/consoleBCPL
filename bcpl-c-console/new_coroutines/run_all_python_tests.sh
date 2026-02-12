#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PYTHON_BIN="${PYTHON_BIN:-/bin/python3}"
ICINT_PY="$SCRIPT_DIR/icint_co.py"
TIMEOUT_SECS="${TIMEOUT_SECS:-20}"

if [[ ! -f "$ICINT_PY" ]]; then
  echo "Missing interpreter: $ICINT_PY" >&2
  exit 1
fi

run_direct_test2() {
  echo "===== TEST2 : python icint_co ====="
  (
    cd "$SCRIPT_DIR"
    timeout "${TIMEOUT_SECS}s" "$PYTHON_BIN" "$ICINT_PY" TEST2
  )
  echo
}

run_compiled_test() {
  local src="$1"

  echo "===== $src : compile ====="
  (
    cd "$SCRIPT_DIR"
    ./compile.sh "$src" >/tmp/compile_${src}.log 2>&1
  ) || {
    echo "COMPILE_FAIL $src"
    cat "/tmp/compile_${src}.log"
    exit 1
  }

  echo "===== $src : python icint_co (RUNABLE) ====="
  (
    cd "$SCRIPT_DIR"
    timeout "${TIMEOUT_SECS}s" "$PYTHON_BIN" "$ICINT_PY" RUNABLE
  ) >"/tmp/py_${src}.out" 2>"/tmp/py_${src}.err" || {
    echo "PY_TIMEOUT_OR_FAIL $src"
    cat "/tmp/py_${src}.out" || true
    cat "/tmp/py_${src}.err" || true
    exit 1
  }

  cat "/tmp/py_${src}.out"
  echo
}

echo "Running coroutine parity checks with $ICINT_PY"
echo

run_direct_test2

for t in \
  test3_callco.b \
  test4_multi.b \
  test5_resumeco.b \
  test6_deleteco.b

do
  run_compiled_test "$t"
done

echo "All coroutine tests passed."
