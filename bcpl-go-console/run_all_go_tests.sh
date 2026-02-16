#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GO_BIN="${GO_BIN:-go}"
if ! command -v "$GO_BIN" >/dev/null 2>&1; then
  if [[ -x "/mnt/c/Program Files/Go/bin/go.exe" ]]; then
    GO_BIN="/mnt/c/Program Files/Go/bin/go.exe"
  fi
fi

cd "$SCRIPT_DIR"

echo "===== TEST2 : direct run ====="
"$GO_BIN" run . TEST2
echo

for t in \
  test1_changeco.b \
  test3_callco.b \
  test4_multi.b \
  test5_resumeco.b \
  test6_deleteco.b
do
  echo "===== $t : compile + run ====="
  ./compile.sh "$t" --coro >/tmp/go_compile_${t}.log 2>&1 || {
    echo "Compile failed for $t"
    cat /tmp/go_compile_${t}.log
    exit 1
  }
  cat /tmp/go_compile_${t}.log
  echo
done

echo "All Go coroutine tests passed."
