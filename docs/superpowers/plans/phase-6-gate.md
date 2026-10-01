# Phase 6 gate record

## S0 and S1

Head `1f9be8ea5` before this record's commits. Base `1754a3b5a`. `S` is
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t11`.

### Performance

Tables, commands and binary hashes: `phase-6-perf.md` `## Task 11`.

- Instructions, running total against base: `fibfunc` +1.88% is over the +0.3% budget; every other
  program is inside it. `fibfunc` over budget accepted by Moritz, 2026-10-01 (P22).
- Wall clock, recorded only, no layout control (P19): `decrender` +5.68%, `sendloop` +10.14% and
  `textnum` +8.79% are over their ±4% bars; `dispatch` +8.94% is inside its 14.91% band.

### Pinning counter

`phase-6-pinning.md` `## S1 close`. Every park Task 3 counted is still reached; the frame chains no
longer contain `TreeSend`. Input to the S2 plan.

### Recursion depth and Error 11

Programs, each run with `$S/bin/NAME/rexx-run`, output `depth 9999 rc 11` on base and on head for
recursion by `CALL`, by function and by send. The three counter programs:

```
n = 0                                   /* call.rex */
signal on syntax name h
call sub
exit
sub:
n = n + 1
call sub
return
h: say 'depth' n 'rc' rc
```

`func.rex` and `send.rex` are the same shape with `.local~n` as the counter, through a `::ROUTINE`
`f` (`return f()`) and a `::METHOD` `m` (`return self~m`).

Cap lifted: a scratch copy of `1f9be8ea5` with `MAX_ACTIVATION_DEPTH` and `MAX_EVAL_DEPTH` at
100,000,000 (`rust/crates/rexx-exec/src/run/call.rs`, `src/eval.rs`), run under
`ulimit -v 4194304`, the program taking its depth as an argument and returning `'bottom'` through
`return f(n - 1)` (function), `return self~m(n - 1)` (send) and `call sub k - 1` (call):

| form | depth | wall s | max RSS KB | result |
|---|---:|---:|---:|---|
| call | 1,000,000 | 1.00 | 1,211,076 | bottom |
| function | 1,000,000 | 0.72 | 1,001,456 | bottom |
| send | 1,000,000 | 0.79 | 1,032,424 | bottom |

The counter programs with an `exit` at depth 4,000,000: `call` reaches it (rc 0, 2.38 s, 3,398,964 KB);
function and send abort with signal 6 (rc 134) after 17-18 s at about 3.3 GB, under the
address-space cap. An `exit` at depth 1,000,000 from the function and send counter programs did not
finish in 300 s; the send counter program with `exit` at depth 10,000, 20,000 and 40,000 takes
0.10 s, 0.22 s and 0.69 s. The `call` counter program takes 0.61 s at 1,000,000. The growth of the
`exit` cases was not investigated.

### Remaining loud refusals that name Phase 6

Derived by the commands below, run from the repository root at `1f9be8ea5`:

```
/bin/grep -a -rn 'Phase 6' rust/crates --include=*.rs | /bin/grep -av '/tests\.rs\|/tests/' | /bin/grep -av ':[0-9]*:\s*//'
/bin/grep -a -rPzoc 'Phase\s+6' rust/crates --include=*.rs        # wrapped occurrences: same files as the first command
/bin/grep -a -c 'Phase 6' rust/corpus/refusal-sites.tsv             # 0: that file records no owner
```

The first command's non-test hits, grouped by where the refusal or its owner is:

| where | refusals |
|---|---|
| `rexx-exec/src/lib.rs:679` | a `GUARD` that has to wait for another activity to make its `WHEN` expression true |
| `rexx-exec/src/lib.rs:688` | `Message~result` on a message whose send has not been made |
| `rexx-exec/src/lib.rs:696` | a `REPLY` inside a `DO`, `SELECT` or `IF` |
| `rexx-exec/src/internal_routines.rs` | `SysCloseEventSem`, `SysCloseMutexSem`, `SysCreateEventSem`, `SysCreateMutexSem`, `SysOpenEventSem`, `SysOpenMutexSem`, `SysPostEventSem`, `SysReleaseMutexSem`, `SysRequestMutexSem`, `SysResetEventSem`, `SysWaitEventSem` |
| `rexx-exec/src/dispatch/native.rs:116` | `alarm_startTimer`, `alarm_stopTimer`, `ticker_createTimer`, `ticker_waitTimer`, `ticker_stopTimer` |
| `rexx-api/src/layout.rs:384` | `MethodContextInterface.SetGuardOnWhenUpdated`, `MethodContextInterface.SetGuardOffWhenUpdated` |

The remaining hits are tests (`rexx-exec/tests/`, `*/tests.rs`, `run/tests/`), comments
(`run.rs:2273`, `handle.rs:50`) and the gate's own phase lists (`closed_phases.rs`,
`internal_routines.rs`, `native/tests.rs`). The Phase 6 corpus witnesses stay listed in
`rust/corpus/phase-8.txt` (P17).

### Inputs to S2

- The driver's Park path is refused in S1 and lands with S2's first real parker (P21).
- `REPLY` ordering: the oracle runs a `REPLY` continuation concurrently with its caller, so its
  output can come before the caller's later output; this crate runs it after the caller.
  Pre-existing.

### Gates

Run at `6a621dbbd` by `.superpowers/sdd/2026-09-29-phase-6-s0-s1/p6-gates/gates.sh`; its result lines, verbatim:

```
6a621dbbd76297395444c504aed11f3e8bab31b8
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
6a621dbbd76297395444c504aed11f3e8bab31b8
finished 2026-10-01T11:22:06+02:00
```

G4 release: 2803 passed, 0 failed. G6 debug: 2805 passed, 0 failed (sums of `test result` lines).
