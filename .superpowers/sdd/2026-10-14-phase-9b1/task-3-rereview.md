# Phase 9b1 Task 3 fix round 1 re-review (3536722be..1cd3928eb)

Crate binary: `rexx-run-9b1-t3f` (sha256 prefix `019c2841d223779d`, the report's head binary; the
commits after 386a4797f change records and probes only). Probes are in
`/tmp/claude-1000/rr3/probes/`. Each ran from a fresh `mktemp -d`, the crate under `memcap 4G`,
stdout, stderr and rc compared. Not edited, not committed.

## Verdict: PASS. No open findings.

## I2: per-call copy removed, lent buffer sound

Read `run_internal_as`, `lend_arguments`, `lend_shared_arguments` and `add_internal_level`.
- Every exit path swaps back: the body's result is held in `outcome` with no `?` between the
  lend and the swap-back, so an `Err` and a park (`Started::Entered`) both return the buffer
  before the blame/`?`. `internal_depth` is reset on the same line. A panic is not a path here.
- Re-entry uses the next slot (`internal_depth` indexes), and the slot's `Rc` is cloned for the
  body and dropped before the swap-back, so `Rc::get_mut` succeeds on exit. The cold
  `lend_shared_arguments` path is not reachable today (frame building uses `to_vec`, nothing else
  clones the `Rc`), and it is correct if reached. The lent-out caller buffer holds the slot's
  empty old contents meanwhile; the caller does not read it during the call.
- Probes, oracle vs crate, all IDENTICAL: `rec.rex` (recursive internal procedures calling
  `filespec` at every level), `nest.rex` (a STRING callback inside SysStemSort that itself runs
  a nested SysStemSort whose callback calls `filespec` and reads `.context~stackFrames` at
  depth 2, 7 frames, arguments of the outer and inner sort), `err.rex` (error raised in a STRING
  callback, trapped, then internal calls on and below the unwound slot), `err2.rex` (failing
  `filespec` calls trapped, then more calls and recursion), `act.rex`/`act2.rex`/`act3.rex` (two
  started activities calling internal routines in loops, with `SysSleep` parking, and two
  activities each sorting with a parking STRING callback; act2 and act3 3 runs each).
- GC: the lent arguments are in the slot, which `internal_levels: _` does not root. Ran
  `gc1.rex` (heap argument `'longstem' || 'name.'`, frames held in `.local`, read after more
  allocations) and the nested-sort probe under `run_program_collect_every_alloc` in a scratch
  tree at 1cd3928eb: 460 collections, output equal to the plain run and to the oracle's; the full
  `collect_stress` binary was green (40 passed with my extra test).
- intcall: `callgrind.sh -B 0.5 -p intcall`, pin vs head: 839773236 vs 749689909, -10.7271%,
  -libc -9.8616%, spreads 0.001%. Reproduces the report. Oracle and crate output identical
  (`1050000`).

## intcall follows the conventions

`intcall.rex` has the header-comment-then-loop-then-`say` shape of `pkglocal.rex`, one README
row (with the why, size and oracle time), and appears in `callgrind.sh` `PROGRAMS` (the on-disk
check in that script would fail otherwise), `rexx-bench` `PROGRAMS` with a comment, and the suite
`AXES` as `Role::Loop`. The README row is alphabetically out of order (it sits after `pkglocal`,
which the table does too: the table is chronological), so nothing to change.

## I1: roots and Package~routines

- `routines` is out of the root loop; `package_public_routines` and `merged_public_routines`
  stay, each with a collect_stress test (read; the report's red mutations are credible and the
  weak-reference form is what makes the public test a witness).
- `Package~routines` reads the `.ROUTINES` table once `routines_written` has the package.
  Oracle vs crate IDENTICAL on `ra.rex`: no writes (1 item, `findRoutine('FOO')` nil), after
  `addRoutine` of a row and of a copy, after a `.ROUTINES` write through the returned table, an
  `addRoutine` after the write, a fresh `.Package~new` package, a package `~copy`, and a global
  `.routines[...]` write seen from two packages. `rb.rex` (public routine added, then
  `.routines['P1'] = u`) IDENTICAL, including the final error text.
- The corpus witness gains the `written 1 0 1` line; the `.rex` and the sourceline file agree
  (`count 30`).

## Entry 37

The added paragraph states what q2 showed (93.900 from an element's STRING inside the sort,
untrapped, SIGSEGV rc 139, 1 of 1), the wording "filed here ... as the three above do" is
accurate (40.922, .923, .924), the probe path `records/.../task-3/probes/q2.rex` exists, and the
crate answer matches the new case in `stem_sort_untrapped_cases/crate` (rc 163, method line, the
`Compiled routine "SYSSTEMSORT".` line, the call line, 93.900). It says the C++ path was not
read, which is true.

Disclosure: I did not intend to run the crashing shape on the oracle, but two probes of mine did,
by mistake. `rec.rex` (first form) called `SysStemSort 'x.'` with `x.0` unset (the 40.922 shape,
rc 139 on the oracle), and `nest.rex` (first form) raised an untrapped 97 inside a STRING callback
during a nested sort (rc 139 on the oracle, the crate gave rc 159 with the ordinary report). Both
agree with entry 37 and add no contradiction: untrapped error inside the sort, SIGSEGV. I
corrected both probes and did not use them as comparisons. The second one is a second observation
of the q2 family (an error raised from a method the sort sends), 1 of 1.

## New defects in the fix diff

None found. Checked: all `run_internal_as` callers pass the buffer and `from` consistently
(`begin_call`, `begin_call_over_values`, `invoke_call` paths, the `mark` offset reads
`&arguments[from..]` everywhere it used to read the slice; `enter_library_program` and
`enter_external_program` copy `arguments[from..]`); `live_levels` limits to
`..internal_depth`; `build_internal_frame_at` derives the name from the row or `called` for
all three `LevelName`s (probes l11-style frames above print the right names).

## Out-of-scope observations

None.
