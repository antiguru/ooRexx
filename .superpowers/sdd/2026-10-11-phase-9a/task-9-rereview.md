# Phase 9a Task 9 re-review (fix round 1, 6e0395fdf..4dacafc63)

Read-only; no suite reruns. The implementer's mutant run and Ir figures were not reproduced.

* I1 addressed. `collect_under_stress` (`rust/crates/rexx-exec/src/dispatch/library.rs:700-705`) is
  called after `pop_native_frame` and `pin_leave!` in `end_library_method` (:129) and
  `end_library_routine` (:221), after `root_native_answer` (:127, :217) and before the answer's first
  use (`settle_native_call`, :537). It gates on the existing `self.stress_collect` field (lib.rs:1608),
  as `between_halves` (:1745) and :343 do; no env read. Both resume paths (:1020, :1041) reach it
  through the same two functions. Objects live across the new collect: popped.additional/result are
  pushed as temps (`take_native_frame` :742), the trapped object is a temp (:636), `Raised` holds
  bytes only; receiver/args are the caller's.
* M1 addressed: `lives` (:1344) precedes `condition_of`'s doc.
* M2 addressed: test doc rests rc 120 on "outside the API contract"; consistent with the exclusions
  rows (phase-4-exclusions.txt:6206, :6223).
* M3 addressed and true: `register_local` pops the temp (:1689-1698), so the object is rooted by
  locals until released or the frame clears.
* M4 addressed and true: `pop_temp_above(floor, v)` pops only when `len > floor` and top == v
  (rexx-core roots.rs:370).

New defects: none Critical/Important. Minor: the report's "+2 Ir per native call" sits on `extcall`
only (stated); not reproduced here.
