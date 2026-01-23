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

## Minimal working example (based on `test_coroutines_min.b`) ✅

Save this as `mini_coroutine.b` into the `bcpl-with-coroutines` folder (or copy from `test_coroutines_min.b`):

```bcpl
GET "LIBHDR"
GET "coroutines"

LET WORKER(ARG) = VALOF
$( WRITES("Coroutines work")
   NEWLINE()
   ARG := COWAIT(ARG+1)
   WRITES("Coroutines work")
   NEWLINE()
   ARG := COWAIT(ARG+1)
   RESULTIS ARG+1
$)

LET START() BE
$( INITCO()
   LET C = CREATECO(WORKER, 5000)
   (void) CALLCO(C, 0)
   (void) CALLCO(C, 10)
   WRITES("Coroutines work")
   NEWLINE()
   (void) CALLCO(C, 20)
   IF (CALLCO(C, 30) NE 31) DO STOP(999)
   WRITES("Lines: 5")
   NEWLINE()
   DELETECO(C)
$)
```

Walk-through:
- `INITCO()` initializes coroutine globals (`COLIST`, `CURRCO`, etc.).
- `CREATECO(WORKER, 5000)` allocates a coroutine whose entry is `WORKER`.
- `CALLCO(C, x)` sends `x` as an argument: the interpreter writes `x` into `C!7` and switches to the coroutine. On first entry `COROENTRY` consumes `C!7` and runs `WORKER(ARG)`.
- Inside `WORKER`, `COWAIT` yields back to the parent and supplies the next argument when resumed.

Expected output (in `output.txt`) for the full test: five "Coroutines work" lines and final "Lines: 5".

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
