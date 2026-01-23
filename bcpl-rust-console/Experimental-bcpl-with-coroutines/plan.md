# Plan: Coroutine-capable INTCODE interpreter (Rust)

## Goal
Enable BCPL coroutine syntax (Callco, Cowait, Resumeco, Createco, Deleteco) to compile to INTCODE and execute correctly under the Rust `icint` interpreter.

## Key coroutine semantics (from PDF)
- Each coroutine has its own stack; all share the global vector.
- Coroutine identity is a pointer to its stack base.
- Operations:
  - `Callco(Cptr, Arg)`: set caller as parent of `Cptr`, then transfer control; result is value returned on resume.
  - `Cowait(Res)`: suspend current coroutine, resume its parent with `Res`; on resume, returns value to caller of `Cowait`.
  - `Resumeco(Cptr, Arg)`: like `Callco` but parent of callee becomes parent of caller; caller becomes parentless. Special case: if `Cptr` is current, yields `Arg` immediately.
  - `Createco(F, Size)`: allocate and initialize coroutine stack. The coroutine runs as if executing `C := F(Cowait(C)) REPEAT`.
  - `Deleteco(Cptr)`: only valid if no parent.
  - `Changeco(A, Cptr)`: low-level primitive that stores resumption point in `!Currco`, switches current coroutine to `Cptr`, restores resumption point, and returns `A` at the new resumption point.
- Stack frame layout used in the paper (per coroutine stack base `C`):
  - `C!0` resumption point
  - `C!1` parent link
  - `C!2` coroutines list link (`Colist`)
  - `C!3` `F` (coroutine main procedure)
  - `C!4` `Size`
  - `C!5` `C` (coroutine pointer)

## Implementation steps
1. **Inspect interpreter stack/return mechanics**
	- Locate where `p`, `g`, `pc` (or equivalent) are saved/restored during calls and `K32_LONGJUMP` in [bcpl-rust-console/src/main.rs](bcpl-rust-console/src/main.rs).
	- Determine the interpreter’s “resumption point” representation (stack pointer + return address) and how to encode it in `C!0`.

2. **Design coroutine runtime state in INTCODE**
	- Decide how `Currco` and `Colist` are stored (likely reserved globals in the BCPL global vector).
	- Define how coroutine stacks are represented in INTCODE memory and how stack pointers map to `C!0` and `C!1`.

3. **Add a primitive for `Changeco` in the interpreter**
	- Introduce a new K-code (or reuse an unused one) to implement `Changeco(A, Cptr)`.
	- On call: store current resumption point into `m[Currco + 0]`, set `Currco = Cptr`, restore `p`/`pc` from `m[Currco + 0]`, return `A`.
	- Ensure the “current coroutine resumed” case for `Resumeco` yields `Arg` immediately.

4. **Implement BCPL library procedures in source**
	- Add BCPL definitions for `Createco`, `Deleteco`, `Callco`, `Cowait`, `Resumeco`, `Changeco` in a runtime module (e.g., new `coroutines.b` or existing runtime file under [bcpl-rust-console](bcpl-rust-console)).
	- Encode stack initialization per the PDF’s `Createco` logic (initialize `C!0..C!5`, add to `Colist`, then `Changeco(0, C)` and loop `C := F(Cowait(C)) REPEAT`).
	- Enforce parent-link rules and error handling as in the paper (invalid parent states).

5. **Integrate into compile pipeline**
	- Ensure the coroutine runtime module is included by [bcpl-rust-console/compile.sh](bcpl-rust-console/compile.sh) so BCPL sources can call coroutine primitives.
	- Verify `libhdr` exports the coroutine procedure signatures.

6. **Validate with BCPL examples**
	- Create small BCPL programs that demonstrate:
	  - `Callco`/`Cowait` ping-pong
	  - `Resumeco` graph-style resumption
	  - `Createco`/`Deleteco` lifecycle
	- Run through the INTCODE path and verify expected output under Rust `icint`.

## Further considerations
- Decide whether coroutine stacks live entirely in BCPL heap (`Getvec`) or need interpreter-side metadata.
- Clarify what “resumption point” encodes in this INTCODE implementation to match the paper’s assumptions.
- Confirm error codes and abort behavior to keep diagnostics consistent with existing runtime.

