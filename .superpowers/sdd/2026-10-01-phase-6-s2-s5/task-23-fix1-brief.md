# Task 23 fix round 1: condensed from two parallel reviews

Sources (detail there): `task-23-review-code.md` (C1-C5), `task-23-review-record.md` (F1-F7).
Ledger: P81 is withdrawn (its premise was false).

## Code

1. **C1.** `island.rs` `Send` grant SAFETY: name the callback path (`Island::host` -> `HostRef`
   deref, `ffi.rs` ~:200/:212) and the abandoned-call path (`Box<OffBaton>` dropped after the lend is
   given back; sound because `OffBaton` holds no `Rc`/`Cell`; check that is true, and if a
   `ThreadContext` Rc rides along, say why its drop is sound or fix it).
2. **C2.** `Heap::clear_uninit_all` (and any other collector or registry walk) must not count as a
   touch: use the untagged resolve there. Add a witness test (the reviewer's probe shape: UNINIT
   objects a second activity never names are not shared), shown red before the fix.
3. **C3.** Make the count profile-independent: debug-only reads (debug asserts etc.) use the
   untagged path. Show corpus figures equal in debug and release, or, if a residue remains, name it.
4. **C4.** Split `SharingReport` at the bootstrap boundary: shared objects made during bootstrap vs
   made by the program, with each population's total, so the fraction is readable.
5. **C5.** `wallclock.sh` usage: distinct names for the oracle list and the run-list override.

## Record (`phase-6-gate.md` `## S5`, records dir, spec)

6. **F1 (P81 withdrawn).** Criterion 6 over the whole ooTest that the in-process runners reach
   (`group_runner::run_crate` and the keyword/bif/expression assertion harnesses), plus the corpus;
   state each population exactly. Re-derive all criterion 6 figures after C2-C4 with commands and logs.
7. **F2.** P74 caveat: `NativeState::Pointer` also holds Alarm/Ticker `TimerId`s
   (`time_support.rs` ~:195-206); state what it holds truly.
8. **F3, F4.** Fix the `ffi.rs:1320` citation (not a HostRef assert; cite what pins the claim or drop
   it); add rows or row entries for `island.rs:163`, `signal.rs:237`, `:269`; correct "cfg(test)
   counters" to what those statics are.
9. **F5.** Spec amendment cites P72 for "sealed". **F6.** "Two spec sentences": no count.
10. **F7.** `thirty-runs.sh` runs `rexx-run` from a fresh empty dir; re-run if its results are cited.

Checks: P51 bar (fmt, clippy -D warnings with no features, `sharing`, `sharing`+`pinning`;
`cargo test --workspace --release`; rexx-core doctests). Rules as in `task-23-impl-brief.md`
(no `bash -c`, literal rm, foreground runs, scratch `/tmp/claude-1000/p6-t23/`, commit with -F and
the two trailer lines). Append a "Fix round 1" section to `task-23-report.md`.
