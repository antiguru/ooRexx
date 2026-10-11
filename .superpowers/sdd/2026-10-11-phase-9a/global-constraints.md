## Global Constraints

* **Behaviour.** The corpus differential stays identical except where a task names the programs it
  changes; each is witnessed against the oracle. New corpus witnesses go in a new list
  `rust/corpus/phase-9.txt`, registered in Task 1 beside `phase-6-1.txt` in every harness that names
  it (`grep -rln '"phase-6-1.txt"' rust/crates`: `corpus.rs`, `coverage.rs`, `collect_stress.rs`,
  `trace_oracle.rs`, `ir_recorded.rs`).
* **`unsafe`**: no new site (`crates/rexx-core/tests/unsafe_sites.rs` lists the granted ones). No new
  dependency. No `Op::Generic`; `Op::Clause` at discriminant 0; `size_of::<Op>() == 16`.
* **Oracle runs**: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
  timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )` from a fresh empty directory
  you `mkdir`; stdout, stderr and status compared separately; `rust/corpus/oracle-crashes.txt` read
  first. Every crate interpreter run under `memcap 2G` and `timeout`, every test binary and build under
  `memcap 8G`, `-j 4`, `CARGO_INCREMENTAL=0`.
* **Read-only**: the C++ tree, `samples/`, `build/`, `ootest/`, `oodocs/`, `testbinaries/`, `api/`.
* **Prose**: comments and docs state decisions, never a mechanism that does not exist yet, never a
  set's size, no em-dashes; a false sentence is deleted. Extent claims carry the command that derived
  them. Every witness's stdout is read to confirm each path it claims prints.
* **Shell**: no `bash -c`/`sh -c` wrappers; `rm` only with literal absolute paths, no globs; never
  `git checkout`/`stash`/`reset`; mutants are reversed by an exact edit and confirmed with
  `git diff --stat`.
* **Per-task check, at most 5 minutes in all (spec section 9, Moritz 2026-10-11):** `cargo fmt`,
  `cargo clippy -p <touched crates> --all-targets -- -D warnings`, the test binaries covering what
  changed (`cargo test --release -p rexx-exec --test <name> ...`), and the ooTest groups the task
  touches through Task 2's harness (`REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=<dir/group,...>`). Name the
  commands in the report with their exit status, unpiped.
* **Refusal tables**: a task that adds, deletes or relabels a `Loud` constructor re-derives
  `rust/corpus/refusal-sites.tsv` with `REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test
  refusal_sites` and keeps `refusal-dispositions.tsv`, `closed_phases` and `owners` green in the same
  commit.
* **Mechanical batches.** A missing method or a refusal with one obvious oracle answer is one
  function and one witness; Tasks 5 and 6 batch them. A batch item that turns out not to be mechanical
  (it needs a new store, a new identity rule, or a change of a shared path) is reported and moved to
  its own task by the controller, not widened inside the batch.
* **Gates**: the full gates (G1-G9, TSan, the whole-suite gate, callgrind) are the Phase 9 close's.
  9a's close runs Task 10's list only. Commit before any long run; the tree stays frozen until its
  status file says finished.

## Rulings made in this plan

* Ruling: the denominator enumerates with `group_runner::source_test_names`, widened to follow a test
  class that inherits from or subclasses another `ooTestCase` class in the same file -- spec section 2
  named `rexx-extract`'s `extract`, which counts methods of non-test classes and resource bodies; the
  spec is amended to match -- if wrong, the enumeration function is swapped and `suite-not-run.txt`
  re-derived.
* Ruling: the perf pin is built in Task 1 from `a98238be5` and stored outside the tree with its sha256
  in the 9a gate record; 9a measures nothing against it -- spec section 8 measures once at the Phase 9
  close -- if wrong, the pin is rebuilt from the same commit.
* Ruling: a group that refuses before any test starts (scout A: `RexxInfo~executable` in the
  `samples` groups) is recorded as blocked at load with the refusal text, not iterated -- nothing can be
  renamed out -- if wrong, those groups get a load-time rename rule.

## Review Focus

1. **Latin-1 group files.** `C2X.testGroup` carries a literal `0xAA`; a rename-out must leave every
   other byte unchanged. Task 1 adds a test that `skip` over a copy of `C2X.testGroup` changes only the
   renamed method's line, compared byte for byte.
2. **A session without `memcap`.** The harness falls back to `ulimit -v 4194304` and says so. Task 2
   tests the fallback by pointing the `memcap` lookup at a missing name.
3. **A refusal raised at group load.** No test has started, so iteration cannot rename one. Task 2's
   harness test runs a fixture group whose prolog refuses and asserts the table records "blocked at
   load" with the refusal text, after one run.
4. **A test that hangs on the crate side.** The crate run times out, the last started test is renamed
   out, and the next run proceeds; the cap stops a group that hangs on every test. Task 2 tests it with a
   fixture whose test loops forever.
5. **A relabel that changes refusal text.** Task 4 changes labels only; every refusal's text other
   than its phase name stays byte-identical. Task 4's `refusal-sites.tsv` refresh is diffed, and only
   phase names may change in it.

