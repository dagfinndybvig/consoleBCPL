# Coroutine Tutorial (Passing Tests)

This tutorial explains the three passing coroutine tests and the coroutine syntax/semantics used in this project. The tests are:
- [bcpl-rust-console/bcpl-with-coroutines/test_coroutines_min.b](bcpl-rust-console/bcpl-with-coroutines/test_coroutines_min.b)
- [bcpl-rust-console/bcpl-with-coroutines/test_coroutines_resume.b](bcpl-rust-console/bcpl-with-coroutines/test_coroutines_resume.b)
- [bcpl-rust-console/bcpl-with-coroutines/test_coroutines_delete.b](bcpl-rust-console/bcpl-with-coroutines/test_coroutines_delete.b)

## Coroutine syntax and semantics

The coroutine API is provided by the runtime library [bcpl-rust-console/bcpl-with-coroutines/coroutines.b](bcpl-rust-console/bcpl-with-coroutines/coroutines.b) and exposed via [bcpl-rust-console/bcpl-with-coroutines/libhdr](bcpl-rust-console/bcpl-with-coroutines/libhdr). At a high level:

- `INITCO()` initializes the coroutine system and creates the main coroutine control block.
- `CREATECO(f, size)` creates a coroutine for function `f` with stack size `size`. It returns a coroutine control block pointer, or 0 on failure.
- `CALLCO(c, a)` switches to coroutine `c` and passes argument `a`. It returns when the coroutine yields or exits.
- `COWAIT(a)` yields back to the parent coroutine, returning `a` to the caller.
- `RESUMECO(c, a)` resumes coroutine `c` and passes `a`. It returns when `c` yields back.
- `DELETECO(c)` deletes a coroutine control block when it is safe to do so (typically when it has no parent).

Conceptually, each coroutine has a control block that stores its saved stack pointer (`sp`), program counter (`pc`), parent link, and entry function. A coroutine runs until it calls `COWAIT` (or exits), which switches control back to its parent. The interpreter’s `CHANGECO` K‑code does the actual context switch by saving/restoring `sp` and `pc` from the control blocks.

## Test 1: Minimal coroutine smoke test

File: [bcpl-rust-console/bcpl-with-coroutines/test_coroutines_min.b](bcpl-rust-console/bcpl-with-coroutines/test_coroutines_min.b)

What it does:
- Creates a worker coroutine and the main coroutine acts as the caller.
- The worker yields multiple times using `COWAIT`, allowing the caller to regain control after each yield.
- The caller verifies the return values and prints the expected sequence of lines.

How it works:
- `INITCO()` sets up coroutine state.
- `CREATECO` allocates a coroutine for the worker function.
- The caller repeatedly uses `CALLCO` to enter the worker. Each time the worker calls `COWAIT`, control returns to the caller.
- The test checks for correct return values and prints “Coroutines work” for each yield, then prints the final line count.

Semantics exercised:
- Coroutine creation and first entry.
- Cooperative yielding via `COWAIT`.
- Returning control and values through `CALLCO`.

## Test 2: Self-resume semantics

File: [bcpl-rust-console/bcpl-with-coroutines/test_coroutines_resume.b](bcpl-rust-console/bcpl-with-coroutines/test_coroutines_resume.b)

What it does:
- Validates that `RESUMECO(CURRCO, a)` is a no‑op resume that returns `a` immediately.

How it works:
- The test runs inside the current coroutine and calls `RESUMECO` on itself.
- The runtime detects this case and returns the input argument without switching context.
- The test prints “Resume ok” if the returned value matches.

Semantics exercised:
- Self‑resume shortcut behavior.
- Correct handling of `CURRCO` without context switching.

## Test 3: Delete semantics

File: [bcpl-rust-console/bcpl-with-coroutines/test_coroutines_delete.b](bcpl-rust-console/bcpl-with-coroutines/test_coroutines_delete.b)

What it does:
- Creates a coroutine and then deletes it once it is no longer active, ensuring `DELETECO` works safely.

How it works:
- `INITCO()` initializes the system.
- `CREATECO` allocates a coroutine control block.
- The test ensures the coroutine is not running and has no parent before calling `DELETECO`.
- If deletion succeeds, it prints “Delete ok”.

Semantics exercised:
- Allocation and deallocation of coroutine control blocks.
- Safety check that deletion is only performed when the coroutine is not active.

## Where to look next

- Runtime implementation: [bcpl-rust-console/bcpl-with-coroutines/coroutines.b](bcpl-rust-console/bcpl-with-coroutines/coroutines.b)
- Interpreter coroutine switching: [bcpl-rust-console/bcpl-with-coroutines/src/main.rs](bcpl-rust-console/bcpl-with-coroutines/src/main.rs)
- Build/test workflow: [bcpl-rust-console/bcpl-with-coroutines/compile.sh](bcpl-rust-console/bcpl-with-coroutines/compile.sh)
