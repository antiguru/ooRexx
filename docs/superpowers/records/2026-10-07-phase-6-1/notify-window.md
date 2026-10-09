# A `~notify` inside the notification loop: the oracle hangs

Evidence for the `corpus/lang/message_notify.rex` row of `rust/corpus/sim-exempt.tsv` (Phase 6.1
Task 10, red R5).

`notify-window.rex` starts a message whose one party's `messageComplete` sleeps 1 s, then, 0.3 s
later, has main `~notify` a second party `w` and wait on it. The notification loop fixes its count
before the sends and sets all-notified after them (`interpreter/classes/MessageClass.cpp:666-685`),
and `~notify` sends at once only where all-notified is set (`:217-229`), so `w` is never sent
`messageComplete` and `w~wait` never ends.

Oracle command, from a fresh empty directory holding the probe, five runs:

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 8 /home/moritz/dev/repos/ooRexx/build/bin/rexx notify-window.rex )
```

Measured 2026-10-09, oracle 5.3.0 at `build/`: rc 137 on 5 of 5 runs, stdout `notified 1` and never
`waited`, stderr empty. The crate in its normal mode (`rexx-run notify-window.rex`) prints
`notified 1` and refuses with `rexx-exec: a wait that nothing left to run can end is not
implemented`, rc 120.
