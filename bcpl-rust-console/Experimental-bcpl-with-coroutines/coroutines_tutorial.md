# Coroutines tutorial (current implementation) 🔧

This short tutorial documents the current coroutine syntax and semantics as implemented in `bcpl-with-coroutines` (interpreter + runtime). It describes the calling primitives, the runtime contract (including the starter-argument slot), a working example, and how to compile and run tests locally.

---

## Key primitives (BCPL runtime)

- `CREATECO(F, SIZE)` → allocates a coroutine control block and returns its address (vector). `F` is the entry procedure and `SIZE` is the stack size (in words).
- `CALLCO(CPTR, A)` → call `CPTR` as a child coroutine with argument `A` (parent must be 0 in coroutine block). Returns the value that the coroutine yields back.
- `COWAIT(A)` → used inside a coroutine to yield back to the parent, passing `A` as the return value for the parent. The return value from `COWAIT` is the next argument the coroutine receives when it is resumed.
- `RESUMECO(CPTR, A)` → resume `CPTR` from the current coroutine's point of view and pass the value `A`; used for cross-coroutine resume semantics.
- `DELETECO(CPTR)` → free a coroutine that has no parent (parent link must be 0).

Notes:
- The runtime also exposes `GETVEC(n)` / `FREEVEC(addr)` for heap allocation (used by `CREATECO`).
- The runtime assumes a fixed control-block layout (see below). The interpreter cooperates with the runtime to save/restore `sp` and `pc` on `CHANGECO`.

---

## Important runtime contract: starter-argument slot (`C!7`) ✨

There is a small but important convention added to make coroutine startup robust:

- When the interpreter performs `CHANGECO` (the low-level switch), it will stash the incoming *starter* argument into the coroutine control block at slot `C!7` (i.e., write to `m[cptr + 7]`) before switching frames.
- `COROENTRY` (the BCPL entry stub that runs the first time a coroutine starts) will check `C!7`:
  - If `C!7` != 0, it must consume the value (copy into a local `ARG` and set `C!7 := 0`) and use that as the first argument to the coroutine's main procedure.
  - Otherwise, `COROENTRY` falls back to `COWAIT(C)` to obtain its first argument.

This prevents the starter argument from being lost if the initial entry instruction sequence clobbers registers.

---

## Control block layout (summary)

The coroutine control block (`C`) is a small vector; the important slots are:

- `C!0` = saved `sp`
- `C!1` = saved `pc` (entry or return point)
- `C!2` = parent link
- `C!3` = coroutine list link
- `C!4` = main procedure pointer (`F`)
- `C!5` = requested size
- `C!6` = self pointer (C)
- `C!7` = **reserved starter-arg slot** (interpreter writes here during `CHANGECO`)

`CREATECO` initializes these fields and now also sets `C!7 := 0` to make the contract explicit.

---

## Working example (based on `test_coroutines_inline.b`) ✅

Use the test program `test_coroutines_inline.b` as a clear, working example of coroutine interaction. Save or open `test_coroutines_inline.b` under `bcpl-with-coroutines` and run it with `./compile.sh test_coroutines_inline.b`.

```bcpl
GET "LIBHDR"

GLOBAL $(
   CURRCO:500;
   COLIST:501
$)

LET ABORT(N) = STOP(N)

LET INITCO() BE
$( IF CURRCO=0 THEN
   $( LET C = GETVEC(7)
      IF C=0 DO ABORT(200)
      C!0 := LEVEL()
      C!1 := 0
      C!2 := 0
      C!3 := COLIST
      C!4 := 0
      C!5 := 0
      C!6 := C
      COLIST := C
      CURRCO := C
   $)
$)

AND COROENTRY() BE
$( LET C = CURRCO
   LET F = C!4
   LET ARG = 0
   IF C!7 NE 0 THEN $( LET SAVED = C!7 ; C!7 := 0 ; ARG := SAVED $) ELSE ARG := COWAIT(C)
   WHILE TRUE DO
   $( C := F(ARG)
      ARG := COWAIT(C)
   $)
$)

LET WORKER(ARG) = VALOF
$( LET V = 1
   WHILE TRUE DO
   $(
      WRITES("worker got ")
      WRITEN(V)
      NEWLINE()
      V := COWAIT(V+1)
   $)
$)

LET START() BE
$( LET C = 0
   LET V = 1
   INITCO()
   C := CREATECO(WORKER, 200)
   V := CALLCO(C, 1)
   WRITES("main got ")
   WRITEN(V)
   NEWLINE()
   V := CALLCO(C, 10)
   WRITES("main got ")
   WRITEN(V)
   NEWLINE()
   DELETECO(C)
$)
```

Walk-through (step-by-step):
- `INITCO()` sets up the global coroutine list and the initial `CURRCO`.
- `CREATECO(WORKER, 200)` allocates a coroutine `C` seeded to enter `COROENTRY` which will in turn call `WORKER`.
- `CALLCO(C, 1)` performs a switch to the coroutine and supplies starter-arg `1`. The interpreter stashes the starter-arg in `C!7` so `COROENTRY` consumes it and invokes `WORKER(ARG)`.
- `WORKER` prints `worker got 1` (its initial `V`), then executes `V := COWAIT(V+1)`. That `COWAIT(V+1)` yields `V+1` (2) back to the caller; therefore the parent receives `2` as the result of `CALLCO`.
- The parent prints `main got 2`.
- `CALLCO(C, 10)` resumes the worker and supplies `10` as the next argument: the suspended `COWAIT` returns `10` inside the worker, which assigns `V := 10` and then prints `worker got 10`, then `COWAIT(V+1)` yields `11` back to the parent. The parent prints `main got 11`.

Expected output when run:
1. `worker got 1`
2. `main got 2`
3. `worker got 10`
4. `main got 11`

This example demonstrates the start-up contract (starter-arg via `C!7`), `COWAIT` yielding semantics, and a simple call/resume cycle between the parent and a child coroutine.


---

## How to compile and run 🔨

From the `bcpl-with-coroutines` folder:

- Compile & run a single test:

  - `./compile.sh <program>.b` — compiles to OCODE → INTCODE → executes with the Rust `icint`.

- Enable coroutine debug traces (useful when a test hangs or misbehaves):

  - `BCPL_CO_DEBUG=1 timeout 120s ./compile.sh <program>.b` — prints `CHANGECO` traces and instruction steps to stderr.

Notes:
- The build pipeline uses `syni` and `trni` (concatenated as `synitrni`) then runs the built `icint` to produce/run `INTCODE`.
- The script uses CRLF translation to match the BCPL toolchain. Output and stderr are written to `.crlf-build/output.txt` / `.crlf-build/error.txt` during runs.

---

## Debugging tips & recommendations 💡

- If you see `UNKNOWN CALL`, `UNKNOWN EXEC`, or `BAD PC`, enable `BCPL_CO_DEBUG=1` and inspect the printed `CHANGECO` entries and memory dumps around `sp`/`pc`/frame base.
- To reproduce hangs, run with `timeout` and capture the last 200 lines of the error log for analysis.
- Consider adding a small test that fails fast (uses `STOP`) to isolate the startup path rather than long-running examples.

---

## Next steps (suggested)

- Stabilize `test_coroutines_delete.b` and `test_coroutines_resume_cross.b` (currently unstable/hanging in some cases).
- Add a regression test that asserts the `C!7` starter-arg behavior.
- Add a CI job that runs coroutine tests and fails on timeouts.

---

If you want, I can add this file to the project as `coroutines_tutorial.md` and add a short entry in the top-level `README.md` linking to it. Would you like me to do that? 🚀
