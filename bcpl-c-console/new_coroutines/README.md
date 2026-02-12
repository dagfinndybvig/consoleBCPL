# new_coroutines — Coroutine implementation for consoleBCPL (C version)

## Overview

This directory contains a from-scratch implementation of BCPL coroutines
for the C-based INTCODE interpreter.

NOTE:<br>
./compile.sh will now link automatically with CORLIB (intcode) to support coroutines<br>
The result is an intfile called RUNABLE<b>


## Files

| File | Purpose |
|------|---------|
| `icint.c` | Interpreter with CHANGECO debug tracing |
| `icint_co.py` | Coroutine-enabled Python INTCODE interpreter |
| `icint.h` | Compiler portability header |
| `LIBHDR` | BCPL standard library header |
| `compile.sh` | Build + run script |
| `run_all_python_tests.sh` | Runs TEST2 and tests 3-6 using `icint_co.py` |
| `CORO_LIB.b` | Coroutine runtime library (compiled to CORLIB for linking) |
| `test1_changeco.b` | Test: build pipeline + GETVEC |
| `test2_createco.b` | Test: CREATECO + suspend |
| `test3_callco.b` | Test: CALLCO/COWAIT round-trip |
| `test4_multi.b` | Test: multiple yields |
| `test5_resumeco.b` | Test: multiple coroutines |
| `test6_deleteco.b` | Test: DELETECO cleanup |

## Quick Start

```sh
cd bcpl-c-console/new_coroutines
chmod +x compile.sh
./compile.sh test1_changeco.b        # basic sanity check
./compile.sh test3_callco.b          # coroutine round-trip
./compile.sh test4_multi.b           # multiple yields
```

## Python Coroutine Interpreter

You can run precompiled coroutine intcode with the Python interpreter:

```sh
cd bcpl-c-console/new_coroutines
/bin/python3 icint_co.py TEST2
```

To run the full coroutine parity check (TEST2 + tests 3-6):

```sh
cd bcpl-c-console/new_coroutines
./run_all_python_tests.sh
```

Optional environment variables:

- `PYTHON_BIN` (default: `/bin/python3`)
- `TIMEOUT_SECS` (default: `20`)

## Debug Tracing

The interpreter prints CHANGECO traces to stderr. To see only program output:

```sh
./compile.sh test3_callco.b 2>/dev/null
```

## How It Works

See `../new_plan_coroutines.md` for the full design document.

The key insight: `CHANGECO` (K90) is already implemented in the interpreter.
It saves/restores `sp` and `pc` from coroutine control blocks. The BCPL-level
runtime (`CORO_LIB.b`) manages control block allocation, the coroutine entry
trampoline (`COROENTRY`), and the parent-chain protocol (`CALLCO`/`COWAIT`).
