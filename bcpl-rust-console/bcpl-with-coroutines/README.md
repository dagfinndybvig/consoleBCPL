# WORK IN PROGRESS

## What has been done
- Added coroutine support in the Rust `icint` under bcpl-with-coroutines, including a new K‑code `CHANGECO` and a simple heap allocator that implements `GETVEC`/`FREEVEC`.
- Added a coroutine runtime library (`coroutines` / `coroutines.b`) and a coroutine test program (`test_coroutines_inline.b`).
- Added a coroutine-specific `libhdr` exposing `CHANGECO`, `GETVEC`, and `FREEVEC`.
- Added a coroutine build script (`compile.sh`) that compiles within the coroutine folder.

## Current problems
- The coroutine test program currently stalls or crashes at runtime (previously an `UNKNOWN EXEC` and then a panic due to out-of-bounds writes).
- The coroutine control block layout is still being stabilized. The runtime and interpreter have been updated to save/restore both `sp` and `pc`, but the initialization sequence is still fragile.
- The compiler is picky about syntax and declaration ordering; we had to adjust to `GLOBAL` for coroutine state and remove `REPEAT` syntax.

## What remains to be solved
- Make the coroutine control block layout and `CHANGECO` semantics consistent with the compiler’s calling convention.
- Verify the correctness of the initial coroutine stack frame so the first `CHANGECO` lands in the intended entry routine.
- Confirm `CALLCO`/`COWAIT`/`RESUMECO` parent-link invariants work without corrupting the stack.
- Add a minimal, reliable test that exercises `CALLCO`/`COWAIT` without triggering invalid instruction fetches.

## Ideas for a solution
- Use the standard BCPL `APTOVEC` calling convention to build a valid stack frame for coroutine entry, instead of manually seeding `sp`/`pc`.
- Define and document an explicit coroutine control block layout (e.g., `[sp, pc, parent, next, f, size, self]`) and use it consistently in both runtime and interpreter.
- Add sanity checks in `CHANGECO` to guard against invalid `sp`/`pc` values and abort cleanly on corruption.
- Create a step-by-step, single-yield coroutine example and trace `sp`/`pc` transitions under the interpreter to validate the model.
