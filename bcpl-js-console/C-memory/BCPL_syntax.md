# BCPL Syntax (local interpreter summary)

This is a short, focused synopsis of the BCPL assembly/syntax used by the local INTCODE interpreter (`icint`). It describes encoding, assembler tokens, vectors, and the two coroutine primitives the runtime supports.

## Words and storage
- Word size: 16-bit signed/unsigned (host `short`) — `BYTESPERWORD = 2`.
- Memory model: single word-array `m[WORDCOUNT]`. Program code starts at `PROGSTART` (401). `lomem` grows upward for program assembly; `himem` grows downward for vector (heap) allocations.
- Strings: stored as a length-prefixed sequence of bytes at a word address; assembler `C` directive stores characters into the current word's byte slots.

## Basic tokens
- Numeric literals: decimal, optional leading `-` for negative.
- Labels and label references: labels are numeric indices resolved by the assembler; `D` with `L` form defines/reserves words used for label table references.
- Whitespace: space, dollar, newline separate tokens. Comments begin with `/` followed to end-of-line.

## Assembler instruction letters (maps to F0..F7)
- `L` : load (F0) — moves value into `a`.
- `S` : store (F1) — `m[d] = a`.
- `A` : arithmetic add (F2) — `a += d`.
- `J` : jump (F3) — `pc = d`.
- `T` : conditional true jump (F4) — if `a` then `pc = d`.
- `F` : conditional false jump (F5) — if not `a` then `pc = d`.
- `K` : K-code / builtin call (F6) — calls into interpreter; arguments are placed at `d+2` (frame area).
- `X` : execute/primitive (F7) — small VM primitives handled by `switch(d)` in `interpret()`.

Flags used in instruction encoding:
- `FD_BIT` : operand-in-follow-word (the word after opcode contains `d`).
- `FP_BIT` : `d` is SP-relative (add `sp`).
- `FI_BIT` : indirect fetch (`d = m[d]`).
- `FN_BITS` : low bits of opcode hold small function numbers.

## K-code calling convention
- When a `K` call executes, the interpreter computes a frame base `d += sp` and sets `v = &m[d+2]`. Library arguments and return values live from `v` onward.
- Common K-codes implemented: `K11_SELECTINPUT`, `K12_SELECTOUTPUT`, `K13_RDCH`, `K14_WRCH`, `K16_INPUT`, `K17_OUTPUT`, `K40_APTOVEC`, `K41_FINDOUTPUT`, `K42_FINDINPUT`, `K46_ENDREAD`, `K47_ENDWRITE`, `K60_WRITES`, `K62_WRITEN`, `K63_NEWLINE`, `K66_PACKSTRING`, `K67_UNPACKSTRING`, `K70_READN`, `K75_WRITEHEX`, `K77_WRITEOCT`, `K85_GETBYTE`, `K86_PUTBYTE`, `K87_GETVEC`, `K88_FREEVEC`.

## Vector allocator (GETVEC / FREEVEC)
- Header layout at header index `h`:
  - `m[h]` = size (payload words)
  - `m[h+1]` = next free header index (free-list)
  - payload starts at `h+2` and is `size` words long
- `GETVEC(n)` returns pointer `h+2` to payload or `0` on failure. `FREEVEC(p)` expects payload pointer `p` and pushes `h = p-2` onto the free-list.

## Coroutine primitives
- `K40_APTOVEC`: creates a small activation on the heap and switches `sp`/`pc` to start at a new entry point. Expected use: implement heap-based continuations/entry frames.
- `K90_CHANGECO`: stackful coroutine switch primitive. Semantics implemented by the interpreter:
  - Call provides `(arg, cptr, currco_addr)` where `cptr` points to the target coroutine control-block vector and `currco_addr` is where the interpreter stores the pointer to the current coroutine control-block.
  - The interpreter saves the caller's `sp,pc` into the current control block (if non-zero), updates `m[currco_addr]` to `cptr`, then loads `sp,pc` from the target control block and resumes execution. The call returns with `a = arg` in the restored context.

## Assembler helpers and directives
- `C <n>`: place character byte into current word (implemented by `stc`).
- `D` / `L` / `G` directives used by the assembler to place numeric data and label references.
- `Z` : verify and clear label table (assembler check for unset labels).

## Practical notes
- Respect reserved region: program area from `0..PROGSTART-1` is used by assembler and metadata; vectors allocate from the top down (`himem`) and must not collide with `lomem`.
- When writing BCPL coroutines that rely on `K90_CHANGECO`, the BCPL runtime control-block layout must exactly match what the interpreter expects (saved `sp` at offset 0, saved `pc` at offset 1, etc.). Mismatches cause interpreter traps.

For implementation details, see `icint.c` / `icint.js` in this folder which contain the exact constants and handlers referenced above.
