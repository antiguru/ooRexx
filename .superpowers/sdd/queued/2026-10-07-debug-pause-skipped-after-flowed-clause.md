# 2026-10-07-debug-pause-skipped-after-flowed-clause

Found during the trace-analysis assert diagnosis (P87,
`.superpowers/sdd/2026-10-01-phase-6-s2-s5/trace-assert-diagnosis.md`). Not fixed. It is present at base
1754a3b5a, so it is not a Phase 6 regression.

Under interactive debug we do not pause after a clause whose `Op::Exec` answers
`ExecOutcome::Done(flow)`. That includes `NUMERIC` and `TRACE`, which is the ignored-under-debug
case where C++ pauses explicitly (`TraceInstruction.cpp:162`, `:186`). `ir/drive.rs` has one pause
site, `debug_pause_after_clause` on the hot exit of `clause_region!` (drive.rs:1905). The
`RegionEnd::Flowed` path (drive.rs:1383, settled at drive.rs:1930) never asks it. C++ pauses at
the end of every instruction's `execute`.

Probe `pa.rex`, stdin `say 'p1'`, empty, `say 'p2'`, empty, `say 'p3'`, empty:

    trace ?a
    say 'c1'
    numeric digits 9
    say 'c3'
    say 'c4'

Both interpreters exit rc 0, and their stderr is identical: the banner, then `2 *-* say 'c1'`, the
prompt, `3 *-* numeric digits 9`, `4 *-* say 'c3'`, `5 *-* say 'c4'`. stdout differs:

    oracle: c1 p1 p2 c3 p3 c4
    ours:   c1 p1 c3 p2 c4 p3

The pause after `numeric digits 9` is missing, so every later typed line runs one clause late. The
same happens with `trace off` in place of the `NUMERIC`. With all four clauses `SAY`, both print
`c1 p1 c2 p2 c3 p3 c4`. One run each, at HEAD 2286a7e5d and at base 1754a3b5a.
