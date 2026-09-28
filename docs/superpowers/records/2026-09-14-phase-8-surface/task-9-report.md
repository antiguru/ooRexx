# Task 9 report: close Phase 8

Status: DONE_WITH_CONCERNS. BASE `2ae06085c`; gated at `14636fe6a`.

## 1. The Phase 8 enumeration

Three commands, run from the worktree root. The first two read the exclusions file with its line
breaks collapsed, since its rows are hard-wrapped:

    # (A) every OWNER sentence naming Phase 8, NO OWNER sentences dropped
    tr '\n' ' ' < docs/superpowers/plans/phase-4-exclusions.txt | tr -s ' ' \
      | /bin/grep -a -oE '(NO )?OWNER[^.]*Phase 8[^.]*' | /bin/grep -av '^NO OWNER'
    # (B) every sentence naming Phase 8, owner or not, for attribution
    tr '\n' ' ' < docs/superpowers/plans/phase-4-exclusions.txt | tr -s ' ' \
      | /bin/grep -a -oiE '[^.]*phase 8[^.]*'
    # (C) every quoted "Phase 8" outside a comment in any crate's sources
    git grep -n '"Phase 8"' -- rust/crates | /bin/grep -a '/src/' \
      | /bin/grep -avE '^[^:]+:[0-9]+:[[:space:]]*//'

(C) is also what `closed_phases.rs` asserts, now with `"Phase 8"` in `CLOSED` and `rexx-api` among
the scanned crates.

Before (at 2ae06085c, via `git show HEAD:...`):

(A)

    OWNER: Phase 5 for the rest of ::REQUIRES, and Phase 8 for EXTERNAL
    OWNER: Phase 8                      (rcond: POSITION under CALL ON)
    OWNER: Phase 8                      (Directory-subclass AT/PUT)
    OWNER: Phase 8                      (blocking member on a busy outer context)

(the first form of (A), without the NO OWNER filter, also printed the setMethod row's "NO OWNER:
Phase 5 is closed, and the ruling that gave its Directory debt to Phase 8 ...", which is history,
so the filter was added and the before list re-taken with it.)

(C)

    rust/crates/rexx-api/src/layout.rs:395:        .map_or("Phase 8", |(_, owner)| owner)
    rust/crates/rexx-exec/src/dispatch/library.rs:564:  ... Some("Phase 8")),
    rust/crates/rexx-exec/src/dispatch/native/tests.rs:150:    const OPEN: &[&str] = &["Phase 6", "Phase 8", "Phase 10"];

(B)'s sentences are each attributed in section 2's table B.

After, at 915e57a8d: (A) prints nothing; (C) prints nothing, and `closed_phases` is green with
Phase 8 in `CLOSED`. (A) cannot see an owner phrased without "OWNER" (the loader row's "OWNER: the
surface half", row 662's "it is Phase 8's", the L2 paragraph); (B) is what finds those, and section
2's table B attributes each of its sentences.

## 2. Per row: verdict and run

Every run: the oracle under the standard wrapper and the crate's release `rexx-run`, each from a
fresh `mktemp -d` directory, three descriptors compared separately (`compare.sh` from the Task 5
forge, its `S` pointed at `scratchpad/surface-9`), at 2ae06085c plus, for the relabel rows, the
working tree of 5a48f75fa..281b356df. Probes: `docs/superpowers/records/2026-09-14-phase-8-surface/
task-9-probes/` and `task-5-forge/probes/`.

### Table A: rows and refusals Phase 8 owned

| row / site | run | verdict |
|---|---|---|
| exclusions ~431, EXTERNAL refusal forms (`nosuchlib` routine/method/attribute, `LIBRARY REXX file_separator` routine) | x431a-d: identical, rc 158/158/158/166 | DELIVERED note |
| exclusions ~521, "Phase 8 for EXTERNAL" | constlib (::ROUTINE LIBRARY REXX after a failing ::CONSTANT) identical rc 214; constlib2 (::METHOD rxmath) identical rc 166 | DELIVERED; the `LIBRARY rexxutil` spelling differs (below) |
| exclusions ~662, ::REQUIRES LIBRARY | reqlib: identical, `4`, rc 0 | DELIVERED note |
| L2 paragraph (~4537) | METHOD group, unpatched framework, ours: rc 120 at the ticker's GUARD WHEN (Phase 6); with -U, tests run, rc 120 at RXFUNCQUERY (Phase 10) | updated; section 6 |
| "interface member not written yet" (~4566) | sinunit (`RxCalcSin(30, 2, 'Q')`): identical, 88.916 rc 168 | CLOSED by Tasks 5 and 6 |
| Task 5: native frame and PROPAGATED (cond3, sendthrow, nframe; no exclusions row) | all three identical, rc 0 | matched; nothing to delete |
| Task 5: POSITION on a CALL ON-trapped RaiseCondition (rcond) | stdout differs by `  POSITION = 6`; stderr, rc 0 agree | re-homed to Phase 9 (pending lead ruling) |
| Task 5: Directory-subclass AT/PUT (ov, rewritten; the original was not committed) | oracle `v` / `w The NIL object 2`; ours `overridden K` / `The NIL object w 2`; rc 0 both | re-homed to Phase 10 (pending lead ruling) |
| Task 5: blocking member on a busy outer context (outer9, new forge; outer.rex is the non-blocking member and is identical) | oracle `main ok y-in-run run-y` / `y set-by-inner`; ours `run ok y-in-run set-by-inner` / `y Y`; rc 0 both | re-homed to Phase 6 (pending lead ruling) |
| loader/unloader hooks row (~4906) | Task 4's unit tests named in the row | CLOSED by Task 4 |
| Method~new third argument (~4931) | mnew3: ours rc 120 "method "NEW" of class "Method" is not implemented (Phase 9)"; oracle `a Method` rc 0 | owner Phase 9 via the m4 relabel |
| `layout.rs` `refusal_owner` default "Phase 8" | tests/layout.rs holds populated tables to REFUSING_MEMBERS | owner now `None` (5a48f75fa) |
| `library.rs` StaleHandle/Raised "Phase 8" | stale (new forge routine): oracle `got released`/`after` rc 0; ours rc 120 | owner None, recorded (pending lead ruling; fix offered) |
| `native/tests.rs` OPEN list | -- | Phase 8 removed (a3022e3e4) |

New, found by the enumeration: `::routine r external 'LIBRARY rexxutil SysSleep'` (extrout) runs on
the oracle and is 98.903 rc 158 here; the ::METHOD form (extmeth) is 90.998 rc 166 against 98.903
rc 158; `REXXUTIL` spelled upper case agrees at 98.903 (extmeth2). Recorded, Phase 10.

### Table B: (B)'s sentences after the edits

Each sentence naming Phase 8 in the collapsed exclusions file is one of: the Phase 7 close
paragraph's history ("re-homed to Phase 8"); the new Phase 8 close paragraph; row 431's history
(with its DELIVERED note); row 521's DELIVERED pointer; the L2 paragraph's history with its
AT PHASE 8'S CLOSE note; the three trace/blame divergences' history ("predate Phase 8", "partly
Phase 8's", "fixed at 40093e99b"); the rxregexp L1 history; the groups ("stayed Phase 8's, and
pass at its close"); the three RE-HOMED AT PHASE 8'S CLOSE notes; row 8's group history; the loader
row's "Every Phase 8 group's binary" (a description of the binaries; the row now has a CLOSED
note); the Method~new row's history plus its re-home note; the setMethod NO OWNER line; the CLOSED
Unfilled-refusal transcript ("was: ... (Phase 8)"); the Task 8 residuals header ("Phase 8
surface"); the PHASE 8 CLOSES header; and the stale-handle row's "named Phase 8 until this
close". None assigns open work to Phase 8.

## 2b. Negative controls (predictions written before each run)

* NC1: put `.map_or("Phase 8", |(_, owner)| owner)` back into `rexx-api/src/layout.rs`'s
  `refusal_owner` (with the `Option` wrapper removed as needed to compile). PREDICTION:
  `closed_phases`'s `no_refusal_names_a_closed_phase` goes red naming exactly
  `src/layout.rs:<line>: Phase 8`, which also proves `rexx-api` is now scanned.
* NC2: a `"Phase 8"` owner back into `dispatch/library.rs`'s StaleHandle arm. PREDICTION: the same
  test red naming exactly `src/dispatch/library.rs:<line>: Phase 8`.
* NC3: `"Phase 8"` as the owner of one `ExternalBody::Deferred` entry in the LIBRARY REXX registry.
  PREDICTION: `every_deferred_entry_point_names_an_open_phase` red naming that entry, and
  `no_refusal_names_a_closed_phase` red too if the owner is a literal on a non-comment line.

## 3. Refusals relabelled

* `Loud::native_method` (lib.rs): Phase 5 -> Phase 9, per ruling m4(a) (281b356df); the
  expectations of its text in `dispatch/tests.rs` follow. `refusal-sites.tsv` re-derived by
  `tests/refusal_sites.rs`: equal to the committed table (the change moves no line), so unchanged.
  Queued file `.superpowers/sdd/queued/2026-09-28-native-method-refusal-names-closed-phase.md`
  is closed by this and is to be deleted (untracked scratch; not deleted by me).
* `rexx_api::layout::refusal_owner`'s default: Phase 8 -> none (5a48f75fa).
* The stale-handle refusal: Phase 8 -> none (5a48f75fa).
* Not relabelled, and named for the lead (message sent): every other refusal naming Phase 5, a
  closed phase that `closed_phases.rs` documents as deliberately absent from `CLOSED`. Found by
  `git grep -n '"Phase 5"' -- rust/crates | /bin/grep -a /src/` less comment lines: `lib.rs`'s
  `receiver_class`, `operator_operand`, `object_position`, `method_from_source`, `object_method`,
  `method_body`, `library_source`, `required_source`, `setup_method`, `expose_receiver`,
  `use_local_in_a_method`, the `Options` arm of `instruction_owner`; `environment.rs:363`;
  `redirect.rs:644` and `run.rs:3589` (`.STREAM`).

## 4. Miri

`RUSTUP_HOME=<scratchpad>/surface-4/rustup-home CARGO_TARGET_DIR=<scratchpad>/surface-9/target-miri
cargo +nightly miri test -p rexx-api --lib --offline`, Stacked Borrows (no `MIRIFLAGS`), miri
0.1.0 (f7575a9da8 2026-09-24), on the tree committed as 5a48f75fa (rexx-api's sources unchanged
after): exit 0, 54 passed, 0 failed, 8 ignored (`scratchpad/surface-9/miri-1.txt`). The gate
document records it as a recorded run, not a gate: the gate hosts cannot install Miri through
`rustup`.

## 5. Gate status lines

Run 3, at `14636fe6a` (the closing code commit), `scratchpad/surface-9/gates/status.txt`:

    14636fe6a72af47ad8b18f6d3579436db5f29aed
    started 2026-09-28T22:37:59+02:00
    load at start 3.91 7.51 8.58 6/2443 4007506
    G1 fmt exit 0
    G2 clippy(empty target) exit 0
    G3 release build (test --no-run) exit 0
    load G4 6.61 7.56 8.52 5/2584 4009648 2026-09-28T22:39:03+02:00
    G4 release test exit 0
    G4 Compiling lines: 0
    load after G4 4.02 7.20 8.29 2/2570 4138129
    G5 debug build (test --no-run) exit 0
    load G6 4.02 7.20 8.29 3/2582 4138619 2026-09-28T22:44:58+02:00
    G6 debug test exit 0
    G6 Compiling lines: 0
    load after G6 3.39 6.67 7.93 5/2586 73143
    14636fe6a72af47ad8b18f6d3579436db5f29aed
    finished 2026-09-28T22:51:09+02:00

G4 2752 passed / 0 failed / 4 ignored; G6 2753 / 0 / 4. Both failing sets empty. Corpus 652 of
652 in both; api_group_tests passes (RECORDED = FUNCTION.TEST_REXXQUEUE, DETAIL_DIFFERS empty);
gate tables "gated by this run: 0 row(s)".

Run 1, at `915e57a8d`: G4 exit 101, one member, `introspection_arity::the_table_matches_the_three_sides`
-- two RexxInfo evidence cells carry native_method's text, which I relabelled. My miss: I had seen
those rows in the grep before relabelling. Stopped, refreshed through
`REXX_INTROSPECTION_ARITY_REFRESH=1` (diff: those two cells), `783a57e40`. Run 2 at `783a57e40`: all
green. Then I noticed the gate tables' `CLOSED_PHASES` lacked `8` (Phase 7's close added `7`); the
gate-table tests with `REXX_PHASE_GATE=8 REXX_CORPUS_GATE=1` were green at `783a57e40`, no row names
owner 8; added at `14636fe6a` and run 3 taken.

## 6. L2

Still blocked, and not by Phase 8. From a scratch copy of `ootest/framework`, `API/oo`,
`testOORexx.rex`, `worker.rex`, `ooTest.frm` and `rxregexp.cls`, with the oracle's `build/lib` on
`LD_LIBRARY_PATH`, the crate's release `rexx-run testOORexx.rex -f METHOD.testGroup -V 1`:

* without `-U`: rc 120, `rexx-exec: a GUARD that has to wait for another activity to make its WHEN
  expression true is not implemented (Phase 6)` -- the framework's ticker (Phase 6's work);
* with `-U`: the group's tests run, the summary header prints, and the run ends rc 120 with
  `rexx-exec: routine "RXFUNCQUERY" is not implemented (Phase 10)` -- `printSummary`'s probes,
  which the oracle answers through rxapi (Phase 10's work).

The roadmap's row 8 rung is `L2 -> 6, 10`, saying so.

## 7. Commits

* `5a48f75fa` Stop naming Phase 8 in the native boundary's refusals
* `281b356df` Name Phase 9 in the generic native-method refusal
* `a3022e3e4` Close Phase 8 in the closed-phase and open-owner checks
* `915e57a8d` Re-home or close every exclusions row Phase 8 owned
* `783a57e40` Refresh introspection-arity.tsv for the Phase 9 native-method refusal
* `14636fe6a` Gate Phase 8's gate-table rows for the rest of the project
* then a docs-only commit: phase-8-gate.md section 10, roadmap rows 8 and 10, this report in records

## 8. Concerns

* **Rulings requested and not answered while I worked** (two messages to team-lead). I proceeded on
  my recommendations, all reversible: rcond POSITION -> Phase 9; Directory-subclass AT/PUT ->
  Phase 10 (a Phase 8 API defect in `table_at`/`table_put`; fixing it here was the alternative);
  blocking member on a busy outer context -> Phase 6; stale handle -> no owner, recorded (I
  recommended fixing it instead: keep a released local resolvable and rooted until the call
  ends); `LIBRARY rexxutil` -> Phase 10; Phase 5 refusals other than native_method not relabelled.
* The ov.rex probe of Task 5 was never committed; mine is a reconstruction from the review's
  description, and my first version was self-defeating (its PUT override changed the setup key)
  and read "identical" until rewritten.
* `outer.rex` (Task 5's) exercises only a non-blocking member and is identical; the blocking
  divergence was "read, not measured" until `outer9.rex`.
* Commit `783a57e40`'s message splits a test name across a line break.
* The queued file `.superpowers/sdd/queued/2026-09-28-native-method-refusal-names-closed-phase.md`
  is closed by `281b356df` and should be deleted (untracked; I did not delete it).
