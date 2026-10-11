### Task 2: `tests/suite.rs`, the harness

**Files:** `rust/crates/rexx-exec/tests/suite.rs` (new), `support/group_runner.rs`
(`crate_environment` `:469`, `interpreter_bin` `:652`, `run_crate_process` `:545`), fixtures under
`rust/crates/rexx-exec/tests/suite/fixtures/` (tiny `.testGroup` files with the framework copied by the
test, not committed twice).

**Interfaces:** Consumes Task 1's functions. Produces, in `suite.rs` or a `support/suite.rs` module:

```rust
pub struct GroupRun {
    pub group: String,                    // "base/bif/C2X"
    pub expected: BTreeSet<String>,       // filled by Task 3
    pub started: BTreeSet<String>,        // union over iterations, uppercased
    pub passed: BTreeSet<String>,
    pub failed: BTreeSet<String>,
    pub blocked: BTreeMap<String, String>, // test -> captured refusal or end reason
    pub blocked_at_load: Option<String>,
    pub iterations: usize,
    pub ended_normally: bool,
    pub cap: CapKind,                      // Memcap or UlimitFallback
}
pub fn run_group_on_crate(dir: &str, group: &str, renamed_out: &BTreeSet<String>) -> GroupRun;
pub fn run_group_on_oracle(dir: &str, group: &str, renamed_out: &BTreeSet<String>) -> GroupRun;
pub fn discover_groups() -> Vec<(String, String)>; // (dir, group), as testOORexx.rex finds them
```

- [ ] **Step 1:** Fixture tests first, each failing until Step 2: a fixture group whose second test
      calls a refused routine (assert two iterations, the refusal text recorded against that test, the
      third test passed); one whose prolog refuses (Review Focus 3); one whose test loops (Review Focus
      4, with a 5 s bound set by `REXX_SUITE_CRATE_FLOOR` for the test); the `memcap` fallback (Review
      Focus 2); and a `PATH` test: the crate side's environment resolves `rexx` inside the copy's
      `bin`, and `rexxc` nowhere.
- [ ] **Step 2:** Implement per spec section 2: discovery as `ooTestFinder::findFiles` does
      (recursive `*.testGroup` under `ootest/ooRexx`); per side a `fresh_copy`, the same-named-file
      deletion, `without_rxfuncquery`, `reaching_rxapi` renames, `mark_starts`; crate side a child
      `rexx` process (a symlink to `CARGO_BIN_EXE_rexx-run` in the copy's `bin`) with a `PATH` holding
      that `bin` and the system directories only (no oracle directory), asserted per run; bounds (oracle
      300 s, crate max(60 s, 4x oracle), `memcap 2G` or the `ulimit -v 4194304` fallback); iteration
      (rename the last started test, re-run, cap 60; a run that ends with no test started is blocked at
      load); eight groups at once; `REXX_SUITE_GROUPS` restricts the run to named groups.
- [ ] **Step 3:** The svn revision: read `svn info --show-item revision ootest` once, write it in the
      table, and assert it equals `rust/corpus/suite-revision.txt` (new, `13178` if that is what it
      answers; read it, do not assume).
- [ ] **Step 4:** Output: a TSV per run (group, side, expected, started, passed, failed, blocked with
      text, blocked at load, iterations, ended normally, cap) to `REXX_SUITE_TABLE` or
      `target/suite/table.tsv`, and the oracle-difference report beside it.
- [ ] **Step 5:** Run three groups through it (`base/bif/C2X`, `base/class/Package_Options`,
      `samples/samples`) and compare with `scout-a-groups.tsv`'s rows; differences explained in the
      report. Per-task check. Commit.

