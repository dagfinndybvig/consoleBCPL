# Coroutines — self-contained BCPL coroutine toolchain (Python)

## Overview

This folder is a standalone coroutine-enabled BCPL environment.
You can compile and run BCPL coroutine programs entirely from here.

`compile.sh` uses `icint_co.py` for all stages:
1. source `.b` → `OCODE`
2. `OCODE` → `INTCODE`
3. link `INTCODE + CORLIB` → `RUNABLE`
4. run `RUNABLE`


## Files

| File | Purpose |
|------|---------|
| `icint.py` | Standard Python INTCODE interpreter (non-coroutine variant) |
| `icint_co.py` | Coroutine-enabled Python INTCODE interpreter |
| `syni`, `trni`, `cgi` | BCPL compiler passes (INTCODE) |
| `LIBHDR`, `CORHDR` | BCPL headers |
| `CORLIB`, `CORO_LIB.b` | Coroutine runtime library (INTCODE + source) |
| `compile.sh` | Build + run script |
| `compile.bat` | Windows build + run script |
| `run_all_python_tests.sh` | Runs TEST2 and tests 3-6 using `icint_co.py` |
| `run_all_python_tests.bat` | Windows parity test runner (TEST2 + tests 3-6) |
| `test1_changeco.b` | Test: build pipeline + GETVEC |
| `test2_createco.b` | Test: CREATECO + suspend |
| `test3_callco.b` | Test: CALLCO/COWAIT round-trip |
| `test4_multi.b` | Test: multiple yields |
| `test5_resumeco.b` | Test: multiple coroutines |
| `test6_deleteco.b` | Test: DELETECO cleanup |

## Quick Start

```sh
cd bcpl-python-console/Coroutines
chmod +x compile.sh
./compile.sh test1_changeco.b        # basic sanity check
./compile.sh test3_callco.b          # coroutine round-trip
./compile.sh test4_multi.b           # multiple yields
```

You can pass BCPL runtime I/O arguments through `compile.sh`:

```sh
./compile.sh your_program.b -iINPUT -oOUTPUT
```

Windows:

```bat
cd bcpl-python-console\Coroutines
compile.bat test3_callco.b
compile.bat your_program.b -iINPUT -oOUTPUT
```

## Python Coroutine Interpreter

You can run precompiled coroutine intcode with the Python interpreter:

```sh
cd bcpl-python-console/Coroutines
/bin/python3 icint_co.py TEST2
```

To run the full coroutine parity check (TEST2 + tests 3-6):

```sh
cd bcpl-python-console/Coroutines
./run_all_python_tests.sh
```

Windows:

```bat
cd bcpl-python-console\Coroutines
run_all_python_tests.bat
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

`CHANGECO` (K90) in `icint_co.py` switches execution between coroutine stacks.
`CORLIB` provides BCPL-level coroutine runtime support (`CREATECO`, `CALLCO`,
`COWAIT`, `RESUMECO`, `DELETECO`) and is linked into `RUNABLE`.
