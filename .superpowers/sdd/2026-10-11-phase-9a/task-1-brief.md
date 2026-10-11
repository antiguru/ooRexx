### Task 1: Byte-exact group copies, the corpus list, the perf pin

**Files:** `rust/crates/rexx-exec/tests/support/group_runner.rs` (`read_lossy` `:134`,
`without_rxfuncquery` `:294`, `skip` `:667`), `concurrency_tests.rs` (`mark_starts` `:2946` and
`started` `:2960` move to `group_runner.rs`), `rust/corpus/phase-9.txt` (new, empty) and the five
harnesses, the 9a gate record `docs/superpowers/plans/phase-9a-gate.md` (new).

**Interfaces:** Produces `group_runner::{read_bytes(&Path) -> Vec<u8>, skip(...)` (byte-preserving,
same signature), `mark_starts(&Path)`, `started(&Run) -> Vec<String>`}`.

- [ ] **Step 1:** Test first: `skip` over a scratch copy of `ootest/ooRexx/base/bif/C2X.testGroup`
      renaming one test changes exactly that method's line (byte comparison of the rest), and
      `without_rxfuncquery` over a copy of `ooTest.frm` with an `0xFF` byte inserted in a comment does
      not panic and keeps that byte. Both fail today (lossy UTF-8, `read_to_string`).
- [ ] **Step 2:** Make `read_lossy`'s callers that write back use bytes; `skip` and
      `without_rxfuncquery` match and rewrite on `&[u8]`. `whole_groups` uses the same functions.
      Move `mark_starts` and `started` into `group_runner.rs` unchanged.
- [ ] **Step 3:** Create `rust/corpus/phase-9.txt` and register it in the five harnesses.
- [ ] **Step 4:** Build the perf pin: `git archive a98238be5 rust | tar -x -C /tmp/claude-1000/p9/pin/`
      (literal paths), `touch` the tree, build `rexx-run` release under `memcap 8G` with its own
      `CARGO_TARGET_DIR`, require a `Compiling rexx-exec` line, copy the binary to
      `/tmp/claude-1000/p9/pin/rexx-run`, and record its sha256 and the command in the gate record's
      `## Perf pin`.
- [ ] **Step 5:** Per-task check (`--test concurrency_tests` compiles; its gated parts are not run).
      Commit.

