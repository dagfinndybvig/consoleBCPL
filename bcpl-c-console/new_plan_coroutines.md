# Plan: Coroutines for consoleBCPL (C version) — From Scratch

## Scope

Everything produced by this plan goes into `bcpl-c-console/new_coroutines/`.
Nothing in the existing codebase is modified except where explicitly stated.
Previous coroutine attempts in `b-files/` are **ignored** — this is a clean start.

---

## 1. How the interpreter actually works

Before designing coroutines we must nail down the exact execution model.

### 1.1 Interpreter registers

The `interpret()` function in `icint.c` uses four registers stored in C locals:

| Register | Type   | Meaning                      |
|----------|--------|------------------------------|
| `pc`     | `word` | Program counter              |
| `sp`     | `word` | Stack (frame) pointer        |
| `a`      | `short`| Accumulator / return value   |
| `b`      | `short`| Secondary register           |

All state lives in the memory array `m[WORDCOUNT]` (16-bit words, `WORDCOUNT` = 19900).

### 1.2 Call convention (K-code, opcode F6)

When the BCPL compiler emits a function call, it generates a `K` instruction.
The interpreter does this:

```
d += sp;                          // d is the frame offset from sp
if (a < PROGSTART) {
    // system call — a is a K-code number
    v = &m[d + 2];               // v points to the arguments
    switch (a) { ... }
} else {
    // user function call
    m[d]     = sp;               // save old sp in new frame slot 0
    m[d + 1] = pc;               // save return pc in new frame slot 1
    sp = d;                      // set sp to new frame base
    pc = a;                      // jump to function entry
}
```

### 1.3 Return convention (X4)

```
pc = m[sp + 1];                  // restore return address
sp = m[sp];                      // restore caller's frame pointer
```

Return value is in register `a`.

### 1.4 Key insight: what sp and pc mean

- `sp` always points to the **base of the current activation frame**.
- `m[sp]` is the **saved sp of the caller** (the link back up the call chain).
- `m[sp+1]` is the **saved pc of the caller** (where to resume after return).
- Local variables start at `m[sp+2]`, `m[sp+3]`, etc.

The call chain forms a linked list through `m[sp] -> m[m[sp]] -> ...`

---

## 2. What CHANGECO (K90) must do

`CHANGECO` is already implemented in the interpreter. It is called from BCPL as:

```
CHANGECO(arg, cptr, @CURRCO)
```

The interpreter handler receives three arguments via `v`:
- `v[0]` = arg (value to pass to the target coroutine)
- `v[1]` = cptr (pointer to target coroutine's control block)
- `v[2]` = currco_addr (address of the global variable holding the current coroutine pointer)

**What it does:**

1. Read `currco = m[currco_addr]` (the current coroutine's control block address).
2. If `currco != 0`, save the current interpreter state:
   - `m[currco] = sp`
   - `m[currco + 1] = pc`
3. Update the global: `m[currco_addr] = cptr`.
4. Restore the target's state:
   - `sp = m[cptr]`
   - `pc = m[cptr + 1]`
5. Set `a = arg` (the passed value becomes the return value in the new context).
6. Continue fetching instructions at the new `pc` with the new `sp`.

**This is already correct and working.** The problem has always been in the BCPL-level code that sets up coroutine control blocks and initial stack frames.

---

## 3. Coroutine control block layout

Each coroutine is represented by a vector (allocated with `GETVEC`). We define a fixed layout:

| Offset | Field      | Purpose                                                    |
|--------|------------|------------------------------------------------------------|
| 0      | `c.sp`     | Saved stack pointer (for CHANGECO)                         |
| 1      | `c.pc`     | Saved program counter (for CHANGECO)                       |
| 2      | `c.parent` | Pointer to parent coroutine (set by CALLCO, cleared by COWAIT) |
| 3      | `c.next`   | Linked list pointer for COLIST                             |
| 4      | `c.fn`     | The BCPL function this coroutine runs                      |
| 5      | `c.size`   | Stack size (number of words allocated)                     |
| 6      | `c.self`   | Self-pointer (points back to this control block)           |
| 7+     | stack area | The coroutine's private stack space                        |

---

## 4. The hard problem: CREATECO's initial stack frame

When `CREATECO(fn, size)` allocates a new coroutine, the coroutine has never run.
We must set up `c.sp` and `c.pc` so that when `CHANGECO` loads them, the interpreter
arrives at a valid execution point with a valid stack frame.

### 4.1 The bootstrap sequence

The new coroutine cannot just jump to `fn` directly — `fn` is a BCPL function that
expects to be *called* via the normal K-code mechanism (which sets up a frame with
saved sp/pc at the frame base). Instead, we use a trampoline function called
`COROENTRY`.

The sequence is:

1. `CREATECO` allocates the control block + stack area.
2. It sets `c.sp` to point into the coroutine's own stack area.
3. It sets `c.pc` to the address of `COROENTRY`.
4. It initializes the stack area at `c.sp` with a minimal valid frame:
   - `m[c.sp] = 0` (no caller to return to — sentinel)
   - `m[c.sp + 1] = 0` (no return address — sentinel)
5. It does `CHANGECO(0, c, @CURRCO)` to switch into the new coroutine.
6. The interpreter now runs `COROENTRY` with `sp` pointing at the synthetic frame.
7. `COROENTRY` immediately calls `COWAIT(c)` to suspend back to the creator.
8. When later resumed via `CALLCO`, the argument is passed to `fn(arg)`.

### 4.2 Why previous attempts failed

The critical detail is that when `CHANGECO` loads `sp` and `pc` from the control block,
**the interpreter must find a valid frame at `m[sp]`** and **`pc` must point to valid
INTCODE**. The previous attempts had these problems:

- **Wrong sp value**: used the control block base or `LEVEL()` of the wrong context
  instead of pointing into the coroutine's private stack area.
- **Wrong pc value**: `COROENTRY` is a BCPL function; its *address* in memory is
  its INTCODE entry point. But from BCPL, `COROENTRY` is a global variable holding
  that address. You must store the *value* of the function variable, not the address
  of the variable.
- **Frame linkage mismatch**: the synthetic frame at `sp` must have `m[sp]` and
  `m[sp+1]` set so that if the coroutine ever returns past `COROENTRY`, it doesn't
  crash. Using 0 as a sentinel and checking for it is the safest approach.
- **Inconsistent `@CURRCO` passing**: some attempts passed `@CURRCO` to `CHANGECO`,
  others didn't, leading to the interpreter not updating the global correctly.

### 4.3 The correct CREATECO

```bcpl
AND CREATECO(F, SIZE) = VALOF
$( LET C = GETVEC(SIZE + 7)       // control block (7 words) + stack area
   LET SP0 = C + 7                 // stack base = first word after control block
   IF C = 0 RESULTIS 0

   // Initialize control block
   C!0 := SP0                      // saved sp -> coroutine's own stack
   C!1 := COROENTRY                // saved pc -> trampoline entry point
   C!2 := CURRCO                   // parent = creator (so COROENTRY's COWAIT works)
   C!3 := COLIST                   // link into coroutine list
   C!4 := F                        // function to run
   C!5 := SIZE                     // stack size
   C!6 := C                        // self-pointer

   // Initialize synthetic stack frame at SP0
   SP0!0 := SP0                    // frame link (points to self as sentinel)
   SP0!1 := 0                      // return pc (0 = never return past here)

   // Link into global list
   COLIST := C

   // Switch to the new coroutine; it will immediately COWAIT back to us
   CHANGECO(0, C, @CURRCO)

   // We arrive back here after COROENTRY does COWAIT(C)
   RESULTIS C
$)
```

---

## 5. Complete BCPL runtime library design

### 5.1 Globals

```bcpl
GLOBAL $(
   CURRCO:500;       // pointer to current coroutine's control block
   COLIST:501        // linked list of all coroutines
$)
```

`CHANGECO` is already at global 90 (defined in LIBHDR and handled by the interpreter).

### 5.2 INITCO — initialize the main coroutine

The main program itself is a coroutine. Before any coroutine operations, call `INITCO()`:

```bcpl
LET INITCO() BE
$( LET C = GETVEC(7)
   IF C = 0 DO ABORT(200)
   C!0 := LEVEL()     // save current sp (LEVEL gives the current frame pointer)
   C!1 := 0           // pc doesn't matter for the running coroutine
   C!2 := 0           // no parent
   C!3 := 0           // end of list
   C!4 := 0           // no function
   C!5 := 0           // no separate stack
   C!6 := C           // self-pointer
   COLIST := C
   CURRCO := C
$)
```

### 5.3 COROENTRY — trampoline for new coroutines

```bcpl
AND COROENTRY() BE
$( LET C = CURRCO
   LET F = C!4
   LET ARG = COWAIT(C)        // suspend immediately; return value = first CALLCO arg
   LET RESULT = F(ARG)        // run the user function
   COWAIT(RESULT)              // yield the final result
   ABORT(199)                  // should never reach here
$)
```

### 5.4 CREATECO

As shown in section 4.3 above.

### 5.5 DELETECO

```bcpl
AND DELETECO(CPTR) = VALOF
$( LET A = @COLIST
   // Walk the linked list to find and unlink CPTR
   WHILE !A NE 0 & !A NE CPTR DO A := (!A) + 3
   IF !A = 0 RESULTIS FALSE          // not found
   UNLESS CPTR!2 = 0 DO ABORT(112)   // can't delete an active (has parent) coroutine
   !A := CPTR!3                       // unlink from list
   FREEVEC(CPTR)
   RESULTIS TRUE
$)
```

### 5.6 CALLCO

```bcpl
AND CALLCO(CPTR, A) = VALOF
$( UNLESS CPTR!2 = 0 DO ABORT(110)   // target must not already have a parent
   CPTR!2 := CURRCO                   // set us as the parent
   RESULTIS CHANGECO(A, CPTR, @CURRCO)
$)
```

### 5.7 COWAIT

```bcpl
AND COWAIT(A) = VALOF
$( LET PARENT = CURRCO!2
   CURRCO!2 := 0                      // clear parent link
   IF PARENT = 0 DO ABORT(111)        // must have a parent to return to
   RESULTIS CHANGECO(A, PARENT, @CURRCO)
$)
```

### 5.8 RESUMECO

```bcpl
AND RESUMECO(CPTR, A) = VALOF
$( LET PARENT = CURRCO!2
   IF CPTR = CURRCO RESULTIS A        // resuming self is a no-op
   CURRCO!2 := 0                      // clear own parent
   UNLESS CPTR!2 = 0 DO ABORT(111)   // target must not be active
   CPTR!2 := PARENT                   // transfer parent chain to target
   RESULTIS CHANGECO(A, CPTR, @CURRCO)
$)
```

---

## 6. Debugging strategy

### 6.1 Add CHANGECO tracing to the C interpreter

Create a modified `icint.c` (in `new_coroutines/`) that adds `fprintf(stderr, ...)`
traces inside the `K90_CHANGECO` handler. This prints:

- Entry: arg, cptr, currco_addr, currco value
- The control block fields at cptr: m[cptr] through m[cptr+6]
- Exit: new sp, new pc

This is the single most useful debugging tool — it shows exactly what the interpreter
sees when switching coroutines.

### 6.2 Add a BCPL-level DUMPCO function

Write a small BCPL function that prints the contents of a coroutine control block:

```bcpl
AND DUMPCO(C) BE
$( WRITES("CO: "); WRITEN(C); NEWLINE()
   WRITES("  sp="); WRITEN(C!0)
   WRITES("  pc="); WRITEN(C!1)
   WRITES("  parent="); WRITEN(C!2)
   WRITES("  fn="); WRITEN(C!4)
   WRITES("  size="); WRITEN(C!5)
   NEWLINE()
$)
```

---

## 7. Test programs (incremental)

### Test 1: CHANGECO only (no CREATECO)

Verify the raw `CHANGECO` primitive works by switching between main and a
manually-set-up second context. This isolates the interpreter from the BCPL runtime.

### Test 2: INITCO + CREATECO + immediate COWAIT

Verify that creating a coroutine and having it suspend back works. No actual work
is done by the coroutine function yet.

### Test 3: CALLCO / COWAIT single round-trip

Main calls a worker coroutine with a value, worker does COWAIT with a modified value,
main receives it.

### Test 4: Multiple CALLCO / COWAIT exchanges

Worker yields multiple times before completing.

### Test 5: RESUMECO

Test parent-chain transfer via RESUMECO.

### Test 6: DELETECO

Test cleanup and reuse.

---

## 8. File structure

Everything goes in `bcpl-c-console/new_coroutines/`:

```
new_coroutines/
├── icint.c              # Copy of interpreter with CHANGECO debug tracing
├── icint.h              # Copy of header (unchanged or with debug flags)
├── compile.sh           # Build script: compile icint.c, then compile+run BCPL
├── LIBHDR               # Copy of LIBHDR (unchanged — CHANGECO:90 already there)
├── CORO_LIB.b           # The coroutine runtime library (INITCO, CREATECO, etc.)
├── test1_changeco.b     # Test 1: raw CHANGECO
├── test2_createco.b     # Test 2: CREATECO + suspend
├── test3_callco.b       # Test 3: single CALLCO/COWAIT round-trip
├── test4_multi.b        # Test 4: multiple yields
├── test5_resumeco.b     # Test 5: RESUMECO
├── test6_deleteco.b     # Test 6: DELETECO
└── README.md            # How to build and run
```

The compile script will:
1. Compile `icint.c` → `icint` (the interpreter binary)
2. Copy `syni`, `trni`, `cgi` from parent directory
3. Compile BCPL source → OCODE → INTCODE → run

---

## 9. Implementation order

| Step | What                                              | Validates                          |
|------|---------------------------------------------------|------------------------------------|
| 1    | Set up `new_coroutines/` with icint.c + build     | Build system works                 |
| 2    | Add CHANGECO tracing to icint.c                   | Can see context switches           |
| 3    | Write + run test1 (manual CHANGECO)               | Interpreter primitive works        |
| 4    | Write CORO_LIB.b with INITCO only                 | Main coroutine initialization      |
| 5    | Add COROENTRY + CREATECO, run test2               | Coroutine creation + initial COWAIT|
| 6    | Add CALLCO + COWAIT, run test3                    | Basic coroutine communication      |
| 7    | Run test4 (multiple yields)                       | Sustained coroutine operation      |
| 8    | Add RESUMECO, run test5                           | Parent-chain transfer              |
| 9    | Add DELETECO, run test6                           | Cleanup                            |
| 10   | Remove debug tracing (or make it flag-controlled) | Production-ready                   |

---

## 10. Key risks and mitigations

| Risk | Mitigation |
|------|------------|
| Synthetic stack frame layout wrong | Test 1 uses manual setup with tracing — we see exactly what the interpreter loads |
| COROENTRY address incorrect | Use `DUMPCO` to print the pc value stored; cross-reference with INTCODE listing |
| Stack overflow in coroutine | Use generous SIZE (200+); add bounds check in debug build |
| GETVEC returns overlapping memory | Verify allocator with MAPSTORE or manual inspection |
| BCPL compiler reorders/optimizes | This BCPL compiler is simple and predictable; no optimizations to worry about |

---

## References

- `icint.c` lines 489-515: existing K90_CHANGECO handler
- `icint.c` lines 443-468: K-code call convention (F6_K handler)
- `icint.c` lines 520-521: X4 return convention
- `BCPL_machine.md`: interpreter architecture summary
- Martin Richards, *BCPL: The Language and Its Compiler*
- [Martin Richards' Cintcode BCPL](https://www.cl.cam.ac.uk/~mr10/BCPL.html)

It should be done for the C version in the subfolder bcpl-c-console. The result should be put in the folder bcpl-c-console/new_coroutines. Put everything needed there, ans only there