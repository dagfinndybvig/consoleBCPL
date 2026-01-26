# Remaining Work / Suggestions

This file lists remaining suggestions and next steps for the `C-memory` interpreter and runtimes.

- [High] Investigate and fix `K90_CHANGECO` segfaults
  - Reproduce failure with minimized test case; add assertions to interpreter to trap inconsistency early.
  - Verify control-block layout expected by interpreter matches BCPL runtime (saved `sp` at offset 0, `pc` at offset 1, etc.).
  - Add unit tests and fuzz inputs covering many `sp`/`pc` permutations.

- [High] Add focused unit tests for `K90_CHANGECO` edge cases
  - Bad `cptr` values (<=0, out-of-range), null/zero current coroutine, very small/large stacks.
  - Automate via `tests/` harness and assert interpreter error messages where appropriate.

- [High] CI: build & test on push
  - GitHub Actions workflow to build `C_icint` (release + debug), run `node icint.js` tests, and execute `./tests/run_tests.sh`.
  - Run on Linux (Ubuntu) with matrix for Node and GCC versions; optional ASAN job.

- [Medium] Sync runtime identifiers across implementations
  - `coutfd` is used in C/JS; update Python and Rust runtimes to use same identifier or document mapping.
  - Ensure `K12_SELECTOUTPUT`/`K17_OUTPUT` semantics identical across runtimes.

- [Medium] Expand test coverage & examples
  - Add more BCPL programs exercising coroutines: resume-cross, nested coroutines, coroutine deletion.
  - Add regression tests for previously failing cases.

- [Medium] Improve and centralize documentation
  - Add explicit example of coroutine control-block layout in `BCPL_syntax.md` / `BCPL_for_C.md`.
  - Add quick-start and troubleshooting sections to `BCPL_build_instructions.md`.

- [Low] Code cleanup and tooling
  - Run formatters (clang-format for C, prettier for JS) and add lint rules.
  - Remove any remaining temporary debug instrumentation across branches.
  - Consider extracting shared constants into a single machine-readable file (JSON) and generate `icint.h` / `icint.js` to avoid drift.

- [Optional] Advanced diagnostics
  - Add optional ASAN/Valgrind GitHub job to catch memory issues in the C interpreter.
  - Add a `--trace` mode that emits compact, opt-in instruction traces for hard-to-reproduce bugs.

- [Optional] Release / packaging
  - Add a `Makefile` or `package.json` scripts to simplify build/test tasks for contributors.
  - Create release notes documenting the `coutfd` rename and coroutine work.

If you'd like, I can start by implementing the CI workflow or by writing the `K90_CHANGECO` unit tests next — tell me which to prioritize.
