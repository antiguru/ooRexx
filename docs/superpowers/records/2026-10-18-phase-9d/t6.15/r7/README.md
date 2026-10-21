# Task 6.15 controller fix (round 7): adopt only an offset a command moved

Finding: task-6.15-rereview-r6.md Important 1. `FileConsole::adopt_offset` now returns early when
the descriptor's offset is where the console's own reads, walks and syncs last left it
(`left_at`), so a command that read nothing moves nothing, even while another activity reads.

Measured at the fix, release build, stdin five.txt or big.txt (from the re-review's scratch),
`{ rexx P.rex; echo rc=$?; echo succ:; head -c 80; } < FILE`:

| probe | file | head (this fix) | r5 and oracle (re-review r6) |
| --- | --- | --- | --- |
| conc | five.txt | A one, B two, C three, D four, succ five | A one, B two, C three, D four |
| conc | big.txt | ...0001, 0002, 0003, 0004, succ 0005 on | (r5) the same |
| conc2 | five.txt | B one, C two, D three, succ four five | B one, C two, D three |
| conc2 | big.txt | ...0001, 0002, 0003, succ 0004 on | ...0001, 0002, 0003 |

The single-activity probes (two, h1, mix1-mix3, winput, wc1, wcredir, back, chr, cnt, err, ex3,
lineno, raise) cannot reach the new branch differently: no console read runs between a command's
sync and its adoption. `two` gives one/two/16 and `h1` leaves two..five, as in round 6.

New unit test `a_command_that_read_nothing_moves_nothing`: red with the early return disabled
(`&& false`), green restored. Checks: fmt 0, clippy 0, `--lib` 1167 passed, input_oracle 26,
stream_model 26, refusal tables 3/5/3, all release.

Not fixed, by ruling: re-review r6 Minor 2 (the mark and buffer resets are unobservable on an
unchanged file; they stay for a command that writes the file). Minor 1 is queued
(2026-10-21-quick-native-calls-wall-bound-flake).
