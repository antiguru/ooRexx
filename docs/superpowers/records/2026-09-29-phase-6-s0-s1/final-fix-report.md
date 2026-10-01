# Phase 6 S0/S1 final review fix round

Base `70030c6ed`. Scope: review A finding A1 and pre-existing B1; review B
findings B1-B10, as ruled by the controller. Scratch:
`scratchpad/p6-final-fix/`.

Status: complete. Commits `1242ba315`, `ead73e1f1`, `1a81353e3`; gates green at `1a81353e3`.

## Item 2: review A pre-existing B1, program recursion bound (`1242ba315`)

Change: `run_loaded` (`lib.rs`) raises `Raised::insufficient_stack` when an
activation is already running and `activation_depth()` has reached
`MAX_ACTIVATION_DEPTH`, the check `begin_call` takes. Covers external,
library and `::REQUIRES` program runs. Test:
`spike.rs` `unbounded_external_recursion_raises_11_1_rather_than_overflowing`
(`extf.rex` recursing on itself under `SIGNAL ON SYNTAX`, asserting stdout
`11.1 9998`, rc 0).

Evidence (release `rexx-run`, `memcap 8G`, fresh run dirs under
`p6-final-fix/run/`):

* `deep` (extf self-recursion to 200,000, untrapped): 70030c6ed rc 134
  "has overflowed its stack"; head rc 245, `Error 11.1: Insufficient control
  stack space`.
* `trap` (the test's program): 70030c6ed rc 134 overflow; head stdout
  `11.1 9998`, rc 0. Oracle: `11.1 11278`, rc 0 (its own depth, licensed).
* The test itself: green on head (test profile, under `memcap 8G`); on
  70030c6ed the test binary aborts with SIGABRT, "has overflowed its stack"
  (`red-item2.txt`).
* `refusal_sites` 5 passed: the table lists constructor definitions, not call
  sites, so no row changed.

## Item 1: review A A1, `PinKind::Program` (`ead73e1f1`)

Change: `PinKind::Program`; `run_loaded` runs the body inside
`pinned!(self, nested.then_some(PinKind::Program), ...)`, `nested` being
"an activation is already running" (the same flag item 2 uses).
`concurrency_tests.rs`: `report_of` runs each probe in
`$CARGO_TARGET_TMPDIR/pinning-probes` holding `extf.rex` (recurses to 3 then
`call SysSleep 0`), and `FRAME_PROBES` gains
`("Program", "say f(0)\n::routine f ...return extf(n + 1)")`.

Evidence (`--features pinning`, `memcap 8G`):

* 70030c6ed with the new test file: `a_park_under_each_frame_kind_records_it`
  FAILED, `"Program: {(SysSleep, []): 1}"` (`red-item1.txt`).
* Head, with the probe's kind temporarily renamed to force the map out:
  `{(SysSleep, [Program]): 1}` (`head-item1-map.txt`); file restored.
* Head: `measured::a_ measured::an_` 7 passed (`green-item1.txt`).
* fmt 0; clippy `-p rexx-exec --all-targets` 0 with and without `pinning`.

## Item 3: review B, B1-B10 (`1a81353e3`)

Applied as final-review-b.md says, nothing else in the prose:

* B1 `phase-6-perf.md`: the Task 2 padding-band sentence deleted.
* B2 `phase-6-gate.md`: "recorded only, no layout control (P19)".
* B3 `2026-07-27-rust-rewrite.md` D-U2: `:86`, `:95`, `:157`, `:168`
  (re-read at head: `Rc::clone(&binding.library)`, `thread.clone()`,
  `.map(Rc::clone)`, `thread.clone()`).
* B4 `phase-6-pinning.md` `## Counter`: Native row exempts only the park
  points `RESULT`, `WAIT`, `ACQUIRE`; rows added for `Delegate`
  (`Interp::send_to_delegate`) and `Program` (`Interp::run_loaded` under a
  running activation); TreeEval cell "`Op::EvalExpr`; a call's non-leaf
  argument in `run/call.rs`" (sites re-read at `run/call.rs:334`, `:399`,
  `:1241`).
* B5 `ir/drive.rs` `Interp::drive` doc: "a call op of a driven region";
  `concurrency_tests.rs`: "an unlabelled plain `DO` block".
* B6 `run.rs`: the "boundaries the construct does owe" sentence deleted.
* B7 `clause.rs` `CLAUSES_PER_CHECK`: the request/completion clause deleted.
* B8 `install.rs`: "for the rows of `executable.rs`'s `RESUMABLE_METHODS`"
  (the table is `dispatch/executable.rs:154`); `eval.rs`: "`function_value`'s
  arms".
* B9 `phase-6-gate.md` REPLY ordering: the reviewer's replacement text.
* B10 `phase-6-gate.md`: "its result lines, verbatim".

`refusal-sites.tsv`: `refusal_sites` 5 passed after the edits, so no row
moved and nothing was re-derived. `concurrency_tests` (default features) 3
passed; pinning `measured::a_ measured::an_` 7 passed. No em-dash on an added
line.

## Perf

`bench-programs/callgrind.sh -r 1 -j 6 -p "rexxcps fibcall extcall"`, base
= 70030c6ed, head = 1a81353e3 (`p6-final-fix/cg/`), instructions with libc
and ld-linux subtracted:

| program | base | head | d% |
|---|---|---|---|
| rexxcps | 17663721519 | 17663734272 | +0.0001 |
| fibcall | 8324736209 | 8324736240 | +0.0000 |
| extcall | 8230425246 | 8230425277 | +0.0000 |

## Gates

`S=scratchpad/p6-final-fix bash .../p6-gates/gates.sh` at `1a81353e3`;
status file `p6-final-fix/gates/status.txt`.

Status file, result lines verbatim:

```
1a81353e3d12ac01e42eaacd8a241861b26fdd1e
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
G4 release test exit 0
G4 Compiling lines: 0
G5 debug build (test --no-run) exit 0
G6 debug test exit 0
G6 Compiling lines: 0
G7 clippy --features pinning exit 0
G8 pinning self-tests exit 0
1a81353e3d12ac01e42eaacd8a241861b26fdd1e
finished 2026-10-01T13:08:48+02:00
```

No `git status` lines between the closing sha and `finished`. Totals summed
from the `test result` lines: G4 2804 passed / 0 failed, G6 2806 / 0, G8 7 / 0
(the S1 close run had 2803 and 2805; the difference is the new `spike.rs`
test).
