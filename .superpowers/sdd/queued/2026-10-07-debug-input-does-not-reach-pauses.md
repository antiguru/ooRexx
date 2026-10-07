# 2026-10-07-debug-input-does-not-reach-pauses

Found during the trace-analysis assert diagnosis (P87,
`.superpowers/sdd/2026-10-01-phase-6-s2-s5/trace-assert-diagnosis.md`). Not fixed. Checked at HEAD
2286a7e5d only.

An interactive-debug pause does not read from `.DebugInput`'s destination. `debug_pause_after_clause`
(`run/interpret.rs:202`) reads with `input_line` (`input.rs:330`), which is standard input. This is
why ooTest `TRACE` TEST_TRACE_NUMERIC_DEBUG fails on ours (`whole-groups/alone.txt` line 37:
oracle 1 assertion and 0 failures, ours 0 and 1). Its debug lines come from
`.DebugInput~destination(.ArrayStream~of(...))`, so ours traces `nop /* 3 */` and `nop /* 5 */`,
which the `trace -1` and `trace (0 - 1)` lines would have suppressed.

Probe `di.rex`, stdin `/dev/null`:

    .DebugInput~destination(.lines~new)
    trace ?a
    nop
    say 'after'
    exit
    ::class lines
    ::method init
      expose n
      n = 0
    ::method linein
      expose n
      n = n + 1
      if n = 1 then return "say 'typed'"
      return ''

Both interpreters exit rc 0, and their stderr is identical: the banner, `3 *-* nop`, the prompt,
`4 *-* say 'after'`, `5 *-* exit`. stdout differs: the oracle prints `typed` then `after`, and ours
prints only `after`. One run each.
