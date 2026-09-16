# Self-contained, single-purpose modules — implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to
> implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the Rust files that have grown into several responsibilities into self-contained,
single-purpose modules, with roughly 1,000 lines as the point where a file is worth looking at.

**Requested by Moritz, 2026-09-15:** *"queue a task to split files into smaller ones. No more than 1k
LoC."* Then: *"1k LoC is definitely approximate, and up to negotiation. It's more a 'keep modules
self-contained and single-purpose', without encoding too much opinion."* So the number is a trigger
for looking, not a limit, and nothing in the tree enforces it. This supersedes the 2026-08-15 ruling recorded when `run.rs`'s tests moved out (rank 2
declined, rank 3 "carving `impl Interp` by responsibility: don't").

**Queued, not started.** It runs after Phase 8 surface Task 3's review loop closes and before
surface Task 4, because every later surface task adds to files this plan splits, and splitting first
keeps those additions in modules that already have one purpose. No task of this plan runs while another agent has the worktree.

**One task takes precedence, ruled 2026-09-16.** The repair of gate table C's row set runs first.
The whole-workspace corpus gate has been red since 2026-09-12 (`98a0db498`), and this plan's every
claim of a pure move rests on the test suite noticing if a move is not pure. Splitting files under a
gate that cannot go red would remove the only instrument this plan has.

**Architecture:** Pure moves. Code moves into child modules (`foo.rs` → `foo/*.rs` with `foo.rs`
keeping the module declarations, or a sibling file), `#[cfg(test)] mod tests` blocks into
`foo/tests.rs` or split by topic, with the narrowest visibility widening that compiles
(`pub(super)`, then `pub(crate)`) and `pub(crate) use` re-exports where an existing path must keep
working. No behaviour, name, signature, comment or doc text changes except a comment made false by
the move (for example "this file's own tests"), which is corrected in the same commit.

**Tech Stack:** Rust workspace `rust/`; no new dependency.

## Global Constraints

* **The number is a trigger, not a rule.** A file over roughly 1,000 physical lines (`wc -l`) is a
  candidate; the question for each is whether it holds more than one responsibility. A file that is
  one cohesive thing (a single dispatch table, one type's `impl`, one set of test cases for one
  function) may stay larger, with a sentence in the report saying why. A split never cuts a
  cohesive unit to reach a number, and never creates a module whose only reason is its size.
* **Candidates measured at `5d84dd8cb`** (`find crates -name '*.rs' | xargs wc -l | awk '$1 > 1000'`):
  48 files, 40 under `src/` and 8 under `tests/`; the largest are `rexx-exec/src/dispatch.rs`,
  `run.rs`, `run/tests.rs` and `lib.rs`. Re-measure at the plan's BASE.
* **Little opinion encoded.** No test enforces a length, no allowlist, no naming scheme beyond what the
  surrounding code already does. Module boundaries follow what the code does, found by reading it.
* **Pure move, and proved one:** a green suite does not prove a move (the `run.rs` split record,
  memory `oorexx-run-rs-split`). Every task uses the four instruments that reviewed that move:
  1. the lines that did not move are byte-identical to the corresponding lines before (`cmp` of
     extracted ranges);
  2. whitespace-stripped token streams of every moved item are identical before and after (compare
     per item, since items may reorder across files);
  3. every string and char literal in moved code decodes to the same value before and after (a
     whitespace-blind comparison cannot see a backslash-newline continuation that changed);
  4. test results per test binary and per result block are identical before and after: the same
     test names, the same pass/fail per name, the same count per binary (not a total).
  Plus `cargo doc --no-deps` with its warnings read, since doc-tests do not resolve intra-doc links.
* **Never drop a comment**, and never re-wrap one. A doc comment stays attached to its item; check
  that no insertion orphaned a doc block onto a different item (memory `insertions-orphan-doc-blocks`).
* **File-granular invariants must survive.** Three tests enforce design decisions by file:
  * `rust/crates/rexx-core/tests/unsafe_sites.rs` — D-U1 grants `unsafe` to exactly
    `rexx-api/src/ffi.rs` and `src/load.rs`. A split may not move an `unsafe` block out of those two
    files; if keeping them self-contained would need that, leave them as they are and say so.
  * `rust/crates/rexx-exec/tests/environment_seam.rs` — D45's security chokepoint: the directory
    reads live in `src/environment.rs`. The seam stays in that file.
  * `rust/crates/rexx-exec/tests/dispatch_seam.rs` — an allowlist of dispatch files. Adding a new
    child file to it is allowed only where the moved code is dispatch code the list already
    covered; say so per addition.
  Before each task, `/bin/grep -rn 'src/<file>' rust/crates/*/tests rust/crates/*/src` for every
  file the task touches, and keep every test that names a path meaning what it meant.
* **Derived tables that record source locations re-derive**: `rust/corpus/refusal-sites.tsv`
  (`REXX_REFUSAL_SITES_REFRESH=1`, column 4 only may change), and any other table whose test fails
  on a moved location. A change outside the location column is a finding, not a refresh.
* **Performance:** moving code between modules can change codegen-unit partitioning and inlining.
  Measure `rexxcps` and the `rexx-bench` suite's axes with callgrind before and after each task,
  interleaved, each revision in its own `CARGO_TARGET_DIR`; record the figures. Under ~4% on one
  axis is layout noise on this machine (memory `layout-attribution-needs-a-control`); a larger move
  is a finding for the controller.
* **`git blame -w -C -C -C`** recovers moved lines' history; the commit message says so.
* Commits: one per file split (or per tightly coupled pair), explicit paths, `git commit -F`.
* Every gate green at the plan's closing commit, with the failing sets identical member for member
  to BASE's.
* `/tmp` is a shared tmpfs: `df -h /tmp` before builds; delete target directories by path.

---

### Task 1: The survey, before anything moves

**Files:** the task report only.

- [ ] **Step 1:** Re-measure the candidates at BASE. For each, read it and write down its
      responsibilities: what the parts do, which parts call which, and which parts could stand as a
      module with a one-sentence purpose.
- [ ] **Step 2:** For each candidate, a proposal: split (the target modules, each with its purpose and
      rough length) or leave (why it is one thing). Include the file-granular invariants and the
      tests and tables that name each file by path.
- [ ] **Step 3:** The controller reviews the proposal and rules on it before Task 2 starts. The
      proposal, not a line count, is what later tasks execute; a task may revise it for a reason it
      finds while moving, and says so.

### Task 2: `rexx-exec/src/dispatch.rs`

### Task 3: `rexx-exec/src/run.rs` and `run/tests.rs`

### Task 4: `rexx-exec/src/lib.rs`

### Task 5: the other `rexx-exec/src/` top-level files over the limit
`eval.rs`, `environment.rs` (the D45 seam stays in `environment.rs`), `error.rs`, `value.rs`,
`plan.rs`, `trace.rs`, `activation.rs`, `parse_template.rs`, `builtin.rs`.

### Task 6: `rexx-exec/src/dispatch/*` over the limit
`hash.rs`, `collection.rs`, `string.rs`, `stream.rs`, `library.rs`, `native.rs`, `package.rs`
(`dispatch_seam.rs`'s allowlist per the constraint).

### Task 7: `rexx-exec/src/builtin/*` and `ir/*` over the limit
`convert.rs`, `string.rs`, `datetime.rs`, `numeric.rs`; `ir/drive.rs`, `ir/compile.rs`,
`ir/golden_tests.rs`.

### Task 8: `rexx-api`
`src/values.rs`, `src/invoke.rs`, `tests/values.rs`, and `src/ffi.rs` only by moving safe code out
(the D-U1 constraint; stop and ask if that is not enough).

### Task 9: `rexx-parse`
`src/instruction.rs` and `src/instruction/tests.rs`, `src/directive.rs` and
`src/directive/tests.rs`, `src/ast.rs`, `src/scanner.rs`, `src/expr.rs`, `src/block.rs`,
`tests/scanner.rs`.

### Task 10: the remaining test crates and tools
`rexx-exec/tests/{gate_table_c,coverage,ir_recorded,method_bodies}.rs`,
`rexx-classes/tests/native_classes_wiring.rs`, `rexx-num/tests/format.rs`,
`rexx-bench/src/bin/rexx-bench-suite.rs`, `rexx-extract/src/docs/classes.rs`, and any file the
re-measure at BASE adds.

Tasks 2-10 execute the ruled proposal for their files, and share one shape:

- [ ] **Step 1:** Re-read the ruled proposal against the files at the task's BASE; grep every test
      and table that names them by path.
- [ ] **Step 2:** Move. Narrowest visibility; re-exports for existing paths; comments made false by
      the move corrected.
- [ ] **Step 3:** The four instruments, `cargo doc` warnings, derived tables re-derived with only
      location columns changed.
- [ ] **Step 4:** Performance before and after, recorded.
- [ ] **Step 5:** Commit per file.

### Task 11: Close

- [ ] **Step 1:** Re-measure: list every file still over roughly 1,000 lines with the report's reason
      it is one thing.
- [ ] **Step 2:** The five gates, failing sets identical member for member to BASE's, and the
      performance figures from every task summed per axis, so cumulative drift is visible.
- [ ] **Step 3:** Commit.
