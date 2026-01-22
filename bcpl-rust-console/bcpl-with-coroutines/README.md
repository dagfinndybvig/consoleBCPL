# WORK IN PROGRESS

## What has been done
- Added coroutine support in the Rust `icint` under bcpl-with-coroutines, including a new K‑code `CHANGECO` and a simple heap allocator that implements `GETVEC`/`FREEVEC` in [bcpl-with-coroutines/src/main.rs](bcpl-with-coroutines/src/main.rs).
- Added a coroutine runtime library [bcpl-with-coroutines/coroutines](bcpl-with-coroutines/coroutines) and [bcpl-with-coroutines/coroutines.b](bcpl-with-coroutines/coroutines.b) plus a coroutine test program [bcpl-with-coroutines/test_coroutines_inline.b](bcpl-with-coroutines/test_coroutines_inline.b).
- Added a coroutine-specific header exposing `CHANGECO`, `GETVEC`, and `FREEVEC` in [bcpl-with-coroutines/libhdr](bcpl-with-coroutines/libhdr).
- Added a coroutine build script [bcpl-with-coroutines/compile.sh](bcpl-with-coroutines/compile.sh) that compiles within the coroutine folder.

## Current problems
- The coroutine test program currently stalls or crashes at runtime (previously an `UNKNOWN EXEC` and then a panic due to out-of-bounds writes).
- The coroutine control block layout is still being stabilized. The runtime and interpreter have been updated to save/restore both `sp` and `pc`, but the initialization sequence is still fragile.
- The compiler is picky about syntax and declaration ordering; we had to adjust to `GLOBAL` for coroutine state and remove `REPEAT` syntax.

## Control block layout (current attempt)
The current runtime uses a coroutine control block with the following word layout:
- `C!0` = saved `sp`
- `C!1` = saved `pc`
- `C!2` = parent link
- `C!3` = coroutine list link
- `C!4` = main procedure pointer (`F`)
- `C!5` = requested size
- `C!6` = self pointer

This layout is implemented in [bcpl-with-coroutines/coroutines](bcpl-with-coroutines/coroutines) and [bcpl-with-coroutines/coroutines.b](bcpl-with-coroutines/coroutines.b), and the interpreter expects this in `CHANGECO` in [bcpl-with-coroutines/src/main.rs](bcpl-with-coroutines/src/main.rs).

### Coroutine entry frame (APTOVEC-style)
`CREATECO` now seeds a minimal frame so the first `CHANGECO` lands in `COROENTRY`:
- `C!0` = `SP0 + 1` (frame base)
- `SP0!0` = previous `sp` (0 for new coroutine)
- `SP0!1` = return `pc` (0 for new coroutine)
- `SP0!2` = saved `d_addr` (frame pointer)
- `SP0!3` = argument count (0 for `COROENTRY`)

## Interpreter changes (details)
- Added K‑codes `GETVEC`/`FREEVEC` and a small allocator in [bcpl-with-coroutines/src/main.rs](bcpl-with-coroutines/src/main.rs).
- `CHANGECO` now saves both `sp` and `pc` into the current control block and restores both from the target control block.

## Build/test status
- `./compile.sh` works on simple programs (for example, [bcpl-rust-console/test.b](bcpl-rust-console/test.b)).
- The coroutine test [bcpl-with-coroutines/test_coroutines_inline.b](bcpl-with-coroutines/test_coroutines_inline.b) compiles but still fails at runtime (previously `UNKNOWN EXEC` and earlier an out-of-bounds panic in `F1_S` dispatch).

## Minimal coroutine smoke test (primary)
- The primary smoke test is [bcpl-with-coroutines/test_coroutines_min.b](bcpl-with-coroutines/test_coroutines_min.b) (caller + callee, two yields, return-to-caller check).
- Run it from this folder with fixed input/output and stderr capture:
	- `./compile.sh test_coroutines_min.b -iinput.txt -ooutput.txt > error.txt`
- Always check both output.txt and error.txt after a run; overwrite them once you no longer need the contents.

### Expected output order (output.txt)
1. Coroutines work
2. Coroutines work
3. Coroutines work
4. Coroutines work
5. Coroutines work
6. Lines: 5

If the return value check fails, the test prints "RETURN MISMATCH" before stopping.

## Additional stabilization tests
- [bcpl-with-coroutines/test_coroutines_resume.b](bcpl-with-coroutines/test_coroutines_resume.b): validates the `RESUMECO(CURRCO, A)` self-resume return path.
- [bcpl-with-coroutines/test_coroutines_resume_cross.b](bcpl-with-coroutines/test_coroutines_resume_cross.b): exercises cross-coroutine `RESUMECO` with a separate callee (currently unstable; may hang).
- [bcpl-with-coroutines/test_coroutines_delete.b](bcpl-with-coroutines/test_coroutines_delete.b): validates `DELETECO` only when parentless.

### Debug logging
Set `BCPL_CO_DEBUG=1` to emit coroutine state traces from the interpreter to stderr (captured in error.txt).

## Last known failure notes
- Runtime `UNKNOWN EXEC` suggests the coroutine’s `pc` is not restored to a valid instruction boundary or the stack frame is malformed.
- The out-of-bounds panic occurred in `F1_S` (`self.m[d] = a`), indicating corrupted `d` or `sp` leading to invalid memory accesses.

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

## Fresh-start checklist
1. Re-validate the calling convention used by `APTOVEC` and the runtime stack frame layout in `icint`.
2. Rebuild coroutine entry using `APTOVEC` so the first resume goes through a known-good frame.
3. Add a minimal coroutine test that yields exactly once, then exits cleanly.
4. Add debug prints in the interpreter to log `sp`, `pc`, and `CURRCO` around `CHANGECO`.
5. Confirm `CURRCO` and `COLIST` are in `GLOBAL` and that all procedures are `AND`-linked for forward references.
