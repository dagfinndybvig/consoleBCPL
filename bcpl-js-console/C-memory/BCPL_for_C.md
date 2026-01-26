BCPL interpreter in C (icint.c)
=================================

This file summarizes the C interpreter `icint.c` in this workspace, how its memory and vector allocator work (`GETVEC`/`FREEVEC`), and a short note about coroutines.

Overview
--------
- `icint.c` is a compact interpreter for BCPL "INTCODE". It implements an instruction fetch/decode loop (`interpret()`), builtin (K-code) calls, I/O primitives, and a small runtime memory model in a single flat array `m[]` of words.
- Key constants: `PROGSTART` reserves the low words for program code, `WORDCOUNT` is the total word capacity, and `LABVCOUNT` reserves the label table area.
- The interpreter uses `lomem` and `himem` to manage two regions: growing allocations from the low end (e.g. program assembly) and vector allocations from the high end (heap-like) so they don't collide.

Memory and vectors
------------------
- Memory is a single short array `m[WORDCOUNT]`. The interpreter stores program code starting at `PROGSTART` and uses `lomem` as the next free low-word index.
- Vector allocator (used by BCPL `GETVEC` / `FREEVEC`, exposed as K87 / K88):
  - A vector allocation is implemented with a 2-word header followed by `n` payload words.
  - Header format at header index `h`:
    - `m[h]` = size (number of payload words)
    - `m[h+1]` = next free header index (free-list link; 0 if none)
  - `GETVEC(n)` returns the payload pointer (index `h+2`) or `0` on failure.
  - `FREEVEC(p)` expects the payload pointer `p` and pushes the header `h = p-2` onto the free list.
  - The allocator tries first-fit on the `vecfree` free list; if none found it carves from `himem` (growing downward) and updates `himem`.

Why this layout?
- Keeping program code and vectors in one `m[]` array simplifies serialization and the runtime model for BCPL words.
- Reserving `PROGSTART` and a small label area prevents accidental overlap of program words and vector allocations.

Important invariants
--------------------
- Vector payload pointers are always >= `PROGSTART` and <= `WORDCOUNT`; `FREEVEC` and `GETVEC` must be used consistently.
- `himem` always points just below the last allocated header when allocating from the top; an allocation failing indicates out-of-memory.
- `vecfree` holds the free-list of headers; freed vectors are recycled.

Builtins and K-codes
--------------------
- The interpreter maps BCPL library calls to internal handlers using K-code numbers (e.g. `K87_GETVEC` and `K88_FREEVEC`).
- When an interpreted instruction calls a K-code, the interpreter inspects arguments in the stack frame and dispatches to the matching C function (e.g. `allocvec` / `freevec`).

Coroutines (brief)
------------------
- `icint.c` contains support for two coroutine-related K-codes used by BCPL coroutine runtimes:
  - `K40_APTOVEC` — a low-level primitive that creates a small activation-like frame on the heap and switches `sp`/`pc` to start execution at a new entry point. It is used by some BCPL runtimes to implement controlled entry into heap-based continuations.
  - `K90_CHANGECO` — a higher-level coroutine switch primitive implemented in the interpreter to save and restore `sp` and `pc` into a coroutine control block (vector). This is what a stackful coroutine runtime relies on to switch native interpreter stacks.
- Note: implementing stackful coroutines requires the interpreter to safely save and restore `sp` and `pc` and to agree with the control-block layout that BCPL code constructs. If you don't want interpreter changes, a pure-BCPL approach (trampoline/CPS) stores state in heap vectors and repeatedly calls generator functions; that approach is stackless and portable.

Practical notes for C contributors
---------------------------------
- The interpreter keeps several implementation-specific details (word-size, vector layout, reserved `PROGSTART` area). When editing `icint.c` or writing BCPL glue code, respect the reserved ranges and the vector header conventions.
- Error checking in `icint.c` uses `halt()` for fatal interpreter errors; keep these checks when changing allocation/stack logic to avoid silent corruption.

Where to look in the repository
------------------------------
- `bcpl-js-console/C-memory/icint.c` — main interpreter implementation.
- `bcpl-js-console/C-memory/icint.h` — platform portability and includes used when compiling the interpreter.
- `bcpl-js-console/C-memory/CORO_LIB`, `coro_cps.b`, `corns_cps_example.b` — BCPL-level coroutine (CPS/trampoline) library and examples added to this workspace as alternatives to interpreter-level `CHANGECO`.



*** End of document
