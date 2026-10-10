# SYNTAX propagating through deep frames grows superlinearly

RESOLVED by e8a19b2e6 (Phase 6.1 Task 11a Step 7): `List~append` and `Array~append` read their length and last item without copying slots, so the unwind is linear within the cap (callgrind 173 / 224 / 326 M Ir at depth 2000 / 4000 / 8000); test `dispatch::tests::appends_and_reads_copy_slots_linearly`.

Found by the controller from Phase 6 Task 11's cap-lifted runs (2026-10-01), not fixed. A bare `EXIT`
in a function (`return f(n+1)` chain) raises SYNTAX 44 in its caller (oracle agrees: `h 44`, rc 0),
and the condition unwinds to a `SIGNAL ON SYNTAX` in the main program. On the cap-lifted build
(Task 11 scratch `bin/cap`) time goes 0.17 s at depth 20,000 to 0.58 s at 40,000; `exit 0` instead,
or no trap, stays linear (0.10 / 0.17 s). Depth 1,000,000 did not finish in 300 s. Within the
default cap (9999) base and head are both under 0.1 s. Program:

    signal on syntax name h
    say f(0)
    exit
    h: say 'h'
    ::routine f
      use arg n
      if n = 40000 then exit
      return f(n+1)
