# BCPL CPS/Trampoline Coroutines (C_coroutines)

This document describes the BCPL-only coroutine library and examples added to this workspace. It explains what I changed, how to use the library, examples to run, and important limitations.

**What I added**
- `CORO_LIB` — a standalone BCPL CPS/trampoline coroutine library (in `bcpl-js-console/C-memory/`). Use `GET "CORO_LIB"` to include it.
- `coro_cps.b` — a small reusable library (earlier iterations) and quick tests.
- `test_cps_coroutine.b` — a minimal generator example (yields 1..5).
- `corns_cps_example.b` — an example showing two generators scheduled together.
- Modified `corns.b` to `GET "CORO_LIB"` and added a tiny demo `STARTCPS()` and `GENACPS` to show usage without changing existing stackful code.

Files you can run directly with the workspace toolchain:

- `./C_compile.sh coro_cps.b` — runs the minimal yield example.
- `./C_compile.sh corns_cps_example.b` — runs the two-generator scheduler example.
- `./C_compile.sh corns.b` — original file now includes `CORO_LIB` and exposes `STARTCPS()` demo.

Example commands (run from `bcpl-js-console/C-memory`):

```sh
./C_compile.sh coro_cps.b
./C_compile.sh corns_cps_example.b
./C_compile.sh corns.b   # then call STARTCPS in program entry
```

API (functions exposed by `CORO_LIB`)

- `CORONEW(F, INIT)` -> vector
  - Allocate a small vector to hold coroutine state. `F` is a BCPL function which will be called with the coroutine vector as single argument. `INIT` is an initial numeric state stored in slot 0.
- `CORONEXT(C)` -> value
  - Run the coroutine `C` for one step by calling the function stored in the vector. The function should use `COROGET`/`COROSET` to access and update its state and return a value to yield; returning `0` means the generator is finished.
- `COROSET(C, IDX, VAL)` and `COROGET(C, IDX)`
  - Read/write small state fields inside the coroutine vector. Use these to maintain local state between yields.

Minimal usage pattern

1. Create a coroutine/generator:

```bcpl
LET G = CORONEW(MYGEN, 0)
```

2. Implement generator function `MYGEN(C)` that uses `COROGET`/`COROSET` to persist state and `RESULTIS <value>` to yield values. When finished, return `0`.

3. Drive the coroutine with repeated `CORONEXT(G)` calls and schedule returned values as you like.

Example generator (conceptual)

```bcpl
AND MYGEN(C) = VALOF
$( LET ST = COROGET(C,0)
   LET LIMIT = 5
   IF ST >= LIMIT RESULTIS 0
   ST := ST + 1
   COROSET(C,0,ST)
   RESULTIS ST
$)
```

Limitations and trade-offs

- This is a cooperative, stackless coroutine implementation. It does not change the interpreter or swap program-stack/PC. It works entirely at the BCPL level by storing state in heap vectors and repeatedly calling the generator function.
- Pros: portable BCPL-only solution; no interpreter changes required; simple to reason about and test.
- Cons: you must rewrite coroutine bodies as stateful generators (trampolined style) or transform them automatically; you cannot capture or resume the native call-stack; code using deep call stacks or relying on local activation frames must be converted.

When to use which approach

- If you need true, stackful coroutines (preserving call stacks and PC), continue debugging/implementing `CHANGECO` in the interpreter (this requires interpreter/VM support).
- If you prefer portability and BCPL-only code, port coroutine code to the CPS/trampoline style shown here.

Next steps I can take (pick one):

- Convert one or more existing coroutine examples (for example `test_small2_coroutine.b`) to use `CORO_LIB` so you can compare behavior.
- Extract `CORO_LIB` into a documented library file and update README entries (already added as `CORO_LIB`).
- Continue debugging the interpreter `CHANGECO` implementation for full stackful coroutines.

If you'd like me to convert a specific example to use `CORO_LIB`, tell me which file and I'll implement it and run the test.

-- End of document
