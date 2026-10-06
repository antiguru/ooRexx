# Task 23 review, record half (base a11c91613, head 3d0ab296b)

Spec verdict: CHANGES NEEDED (F1, F2).
Quality verdict: CHANGES NEEDED (F3-F6, all Low).

Line numbers are in `docs/superpowers/plans/phase-6-gate.md` unless another file is named.

## Findings

**F1, Medium (spec), gate:992, task-23-report.md:58-60.** Criterion 6 says "over the corpus and
ooTest". The gate row calls its population "ooTest, criterion 1's derived list". That population
is a subset chosen for concurrency, so its 0.21% says nothing about ooTest as a whole, and the gate
never says the rest of ooTest went unmeasured. The report gives a reason, "the in-process ooTest
runner that exists", and the reason is false. `tests/support/group_runner.rs:404-416`
(`run_crate`, through `Invocation`, used by `tests/api_group_tests.rs`) runs whole ooTest groups in
process. `tests/keyword_assertions.rs`, `bif_assertions.rs` and `assertions.rs` also run
ooTest-lifted bodies in process. Fix: one clause in the gate saying the ooTest figure covers
criterion 1's derived list only, not every ooTest test. Then either drop the report's reason or
measure a wider population.

**F2, Medium (spec), gate:964 (P74 clause).** The clause says "`NativeState::Pointer(*mut c_void)`
... holds C-supplied addresses only", and that is false. `dispatch/time_support.rs:195-206`
(`handle_object`) makes a `.Pointer` whose `NativeState::Pointer` holds an Alarm or Ticker
`TimerId` (`without_provenance_mut(id.address())`), and `timer_of` (`:216`) reads it back. The
clause comes from the audit's R5. The conclusion still holds, because neither kind of value is a
frame. Fix: "holds a C-supplied address or a timer id (`dispatch/time_support.rs:196`), never a
frame".

**F3, Low, gate:937.** "`HostRef` debug asserts (`ffi.rs:192`, `:208`, `:1320`) tested by
`ffi.rs:6330`": `ffi.rs:1320` is not a `HostRef` assert. It is in the callback path and says "a
callback read the interpreter's state without the baton". No test names that message
(`/bin/grep -rn "read the interpreter's state without the baton" rust/crates` finds only
`ffi.rs:1321`). `ffi.rs:6330` expects only the `HostRef` message of `:194` and `:210`. This came
from the audit's table C. Fix: drop `:1320`, or cite it separately with no test.

**F4, Low, gate:921, :934, :942.** The gate says "Every hit falls in a row below". No row names
three test-only hits: `island.rs:163` (`thread::scope` in island's tests) and `signal.rs:237`,
`:269` (the test raisers' `thread::spawn`). The audit's signal row named `:237`/`:269`, and Task 23
dropped them. In the test row, "`#[cfg(test)]` counters" is wrong for some of the statics it
covers: `lib.rs:228` (`POOL_SHAPE`), `install.rs:2481` (offered libraries) and `addsub.rs:198` (a
fast-path switch) are not counters. Fix: add `rexx-exec/src/{island,signal}.rs` tests to the test
row, and say "test statics".

**F5, Low, spec 2026-09-29-phase-6-concurrency-design.md:346-348.** The section 5 amendment says
"whose payloads are sealed ... (ruling P52)". The sealing is P72's ruling; P52 rules the lend. Fix:
"(rulings P52, P72)".

**F6, Low, gate:955.** "Two spec sentences are amended" puts a set size in prose, and no log is
cited beside it. Fix: "Spec sentences are amended to what holds (P73)".

**F7, Info, `pingpong/thirty-runs.sh:365`.** The oracle side runs from a fresh empty dir. The
rexx-run side runs from the invoking cwd. The outputs matched, so this changes no figure.

## Checked and holding

- R1 at head: `/bin/grep -a -rn '...' rust/crates` (P71's pattern) gives the same set of lines as
  `criterion-4-inventory.txt` (sorted `diff` empty). The only Rust change after `0ac73b804` is the
  island.rs comment at e57dc8315, and it keeps the line count. Every hit's file maps to a row,
  apart from F4's test hits. Cited lines checked at head: `island.rs:31,52,130-138`,
  `library.rs:1077`, `scheduler.rs:171,633,664,723`, `input.rs:377`, `timer.rs:205`,
  `lib.rs:3151`, `command.rs:618`, `invoke.rs:250-254`, `tests/invoke.rs:754-760`,
  `load.rs:191,1681`, `body.rs:57,138,177`, `frame.rs:90,99,108,117,168`, `handle.rs:52,57`,
  `context.rs:159`. Every named test exists.
- Non-test `thread_local!`s: `/bin/grep -a -rn -B3 'thread_local!' --include=*.rs rust/crates`
  shows `ffi.rs:700`, `layout.rs:421` and `load.rs:1518` without `#[cfg(test)]`. Every other hit
  is gated or sits in a test file. The 2.5 amendment and the gate row hold.
- `cargo test -p rexx-core --doc` re-run at head (own target dir): "3 passed" and "5 passed"
  (compile fail), as gate:953 says.
- Criterion 6: summing the record tables gives corpus shared 1088, and the derived list's rows give
  objects 374510, made before the program 54944, shared 790. Both match the header lines and the
  gate. 1088/520530 and 790/374510 both round to 0.21%. The tag mechanism (`heap.rs` `made`,
  `resolved`; `share_as` in `switch_to` and `swap_running`) matches the gate's description. The
  text-hash record is consistent: base = base2, aonly = head, and the one renamed symbol.
- Criterion 7: medians recomputed from `wall.tsv` match the gate and `table.txt`. The arm order
  rotates per round (`wallclock.sh` `(k + r - 1) % n`, visible in `wall.tsv`). Load before and
  after is in `binaries.txt`. `nproc` is 32. The command is quoted with its `OUT`/`TARGET`
  placeholders, and `binaries.txt` gives the actual binary path and sha256.
- Criterion 5: the test names exist (`scheduler/tests.rs:981`, `:1013`), the corpus entries exist
  (`phase-8.txt:465-466`), and the P74 caveat is one clause (F2 is about its content).
- Prose: the added lines contain no em-dashes and nothing forward-looking.
