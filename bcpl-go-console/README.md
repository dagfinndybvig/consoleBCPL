# bcpl-go-console

Go port of the coroutine-enabled BCPL INTCODE interpreter.

This folder is now self-contained for normal BCPL compile/run and coroutine tests.

## First run (quick start)

From this directory:

```sh
# standard BCPL example
./compile.sh test.b

# coroutine flow (links CORLIB and runs RUNABLE)
./compile.sh test3_callco.b --coro

# full coroutine regression set
./run_all_go_tests.sh
```

## Included

- Go interpreter: `main.go`
- Compiler assets: `syni`, `trni`, `cgi`
- Headers/libraries: `LIBHDR`, `CORHDR`, `CORLIB`, `CORO_LIB.b`
- Coroutine tests: `TEST2`, `test1_changeco.b` ... `test6_deleteco.b`
- Example BCPL programs: `test.b`, `scan.b`, `cmpltest.b`, `hello.b`, `fact.b`, `queens.b`, `coins.b`

## Directory layout

- `main.go`, `go.mod` — Go interpreter and module definition
- `compile.sh`, `compile.bat` — compile BCPL source and run result
- `run_all_go_tests.sh`, `run_all_go_tests.bat` — coroutine test runner
- `syni`, `trni`, `cgi` — BCPL compiler passes (INTCODE)
- `LIBHDR`, `CORHDR`, `CORLIB`, `CORO_LIB.b` — headers/runtime libraries
- `TEST2`, `test1_changeco.b` ... `test6_deleteco.b` — coroutine test inputs
- `test.b`, `scan.b`, `cmpltest.b`, `hello.b`, `fact.b`, `queens.b`, `coins.b` — sample programs
- `icint` — known-good backend used for compile passes in scripts

## Usage

### Linux/macOS (bash)

```sh
cd bcpl-go-console
./compile.sh hello.b
./compile.sh test3_callco.b --coro
./run_all_go_tests.sh
```

### Windows (cmd)

```bat
cd bcpl-go-console
compile.bat hello.b
compile.bat test3_callco.b --coro
run_all_go_tests.bat
```

## Manual run

You can run the interpreter directly:

```sh
go run . TEST2
go run . RUNABLE
```

Input/output options are supported: `-iINPUT` and `-oOUTPUT`.

Use `--coro` with `compile.sh` / `compile.bat` for coroutine-linked runs (creates and runs `RUNABLE`).
