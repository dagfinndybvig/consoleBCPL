# Fresh start — Where we left off

## Short summary ✅
- Intermittent failure remains: BAD VEC / UNKNOWN CALL observed while running `test_coroutines_inline.b`.
- Repro trace shows a syscall K at `pc=503` computing `d=4`, `d_addr=19705`, `v_ptr=d_addr+2=19707` and then failing because the vector at `19707` is all zeros.
- A `WATCH-HIT` shows the allocation began at `19693` and writes populate `19703` and `19704` but not `19705`/`19707`, suggesting an offset/offset-mismatch or frame-layout issue rather than a direct single-word overwrite.

---

## Evidence & artifacts 📁
- Failing run captured: `/tmp/failure_decoded.log` (decoded instruction history + recent-writes + memory windows)
- Recent writes show writes to 19693..19704 but not to 19707
- Notable outputs:
  - `WATCH-HIT alloc@19693 idx=19693 pc=548 sp=929 a=19701 old=0 new=19701`
  - `UNKNOWN CALL: a=3 v_ptr=19707 d_addr=19705 d=4 sp=19701 pc=504`
  - `VEC @19707..19715 = [0,0,0,0,0,0,0,0]`

Modified code (working-state):
- `src/main.rs` — diagnostics & instrumentation (watch regions, per-instruction canaries, `instr_history`, `recent_writes`, decoded-instruction dump, GETVEC zeroing, K/CHANGECO logging)
- `coroutines.b` — `CREATECO` initializes `C!7 := 0`; `COROENTRY` consumes `C!7` if nonzero
- New tests: `test_getvec.b`, `test_coroutine_arg*.b`, `test_coroutines_inline.b` (failing intermittently)

---

## How to reproduce (quick) ▶️
1. cd `bcpl-rust-console/bcpl-with-coroutines`
2. Run in a loop (already used):
   BCPL_CO_DEBUG=1 timeout 30s ./compile.sh test_coroutines_inline.b
3. Inspect `/tmp/failure_*.log` or `/tmp/failure_decoded.log` for the first failing capture.

---

## Immediate hypotheses (short) ⚠️
- The `K` syscall is looking two words ahead (`v_ptr = d_addr + 2`) while the code that stores the descriptor places values at a different offset — an off-by-two or frame-layout mismatch.
- Less likely: allocator returning partially-initialized area later zeroed without being recorded (we have watch/canary coverage and no observed direct zeroing of `19707`).

---

## Prioritized next steps (actionable) 🔧
1. (Diag) Add additional diagnostic logging in the `K` syscall handler to always dump `sp..sp+8`, `d_addr`, and `d_addr-2..d_addr+4` when the computed `v_ptr` is invalid. Gate by `BCPL_CO_DEBUG`. (low-risk)
2. (Mitigate) Implement a small, gated fallback for `K` syscall: if `v_ptr` contains a zeroed vector or is implausible, scan `d_addr-2` and `d_addr-4` for a plausible descriptor and log a warning while continuing. Keep this behind debug flag.
3. (Root fix) If fallback confirms offset mismatch, trace the producer of the INTCODE around `pc≈492..504` (BCPL source emissions) and fix the emitter or the BCPL frame layout assumptions.
4. Add a deterministic regression test that reproduces the earlier failing sequence (capture input case that hits the problematic `K`) and ensure CI prevents regressions.
5. If the problem looks like allocator bookkeeping/corruption, add stricter allocator invariants and tests for `GETVEC/FREEVEC` overlaps & reuse.

---

## Short-term acceptance criteria ✅
- Diagnostic run confirms the offset mismatch hypothesis by showing correct descriptor words lie at `d_addr-2` (or nearby) and not at `d_addr+2`.
- A guarded fallback prevents BAD VEC crashes and allows the suite to complete while root cause is fixed and tests added.

---

## Notes & where logs live 🗂️
- Captured logs: `/tmp/failure_decoded.log`, `/tmp/failure_recent_writes.log`, `/tmp/failure_canary.log`
- Branch: `main` in repository `dagfinndybvig/consoleBCPL`

---

## Who to contact / next owner
- Continue on this branch and coordinate with the person who added the coroutine instrumentation (me / the repo owner). Keep the failing run logs as artifacts for the root-cause fix.

---

Made concise to allow resuming quickly. Add or edit any prioritized step you want me to take next.
