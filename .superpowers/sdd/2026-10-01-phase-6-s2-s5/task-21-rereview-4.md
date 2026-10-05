# Task 21 re-review, fix round 4

Reviewer: s4-t21-rr4 (opus). Range `4a20439fa..8faaf27b8`, HEAD `8faaf27b8`.

How the probes were run:
- Driver: rr3's `drv.py`, copied to `$CLAUDE_JOB_DIR/tmp/rr4/drv.py` (job `91ae65d5`) with one
  option added, `--close T` (close the program's standard input at T). Each run gets a fresh
  empty directory and `env --default-signal=INT,TERM,HUP`; the oracle runs under
  `ulimit -v 1048576` with the 20 s timeout in the driver (rr3's reason: `timeout(1)` forwards a
  group signal). Signals go only to the PID or group the driver started.
- `cmp.sh NAME N`: N runs per side, oracle and ours interleaved; results counted by distinct
  (rc, stdout, stderr). Programs are under `$CLAUDE_JOB_DIR/tmp/rr4/p/` and `p3/`.
- "HEAD" is `rexx-run` built at `8faaf27b8`, copied aside. "Base" is `rexx-run` at `4a20439fa`
  (the round's base, after P66). "Pre-P66" is `rexx-run` at `bf76afa1d` (rr3's base, before
  P66). Both bases built from `git archive` of `rust` and `interpreter`, touched, each in its own
  target dir, one `Compiling rexx-exec` line in each log.
- Unless stated, input is `one\ntwo\nthree\n` in one write at 1.0 s and the second activity
  starts its access at 0.2 s. Load average 0.5-3 during the probes.

## Verdict

**CHANGES REQUIRED.**

Counts:
- Point 1: the five F-1 shapes rr3 listed are resolved, 30/30 identical to the oracle each.
- New findings: High 1 (N-1), Medium 3 (N-2, N-3 pre-existing, N-4), Info 1 (N-5).

What blocks approval:
- **N-1.** The residual P68 licensed is not 3 cells. Any read that waits pinned, against any
  second access that waits pinned for `.STDIN`'s guard, still ends rc 120 where pre-P66 and the
  oracle run. That includes the plain builtins with arguments: two activities each doing
  `charin(,,1)` or `linein(,,1)`. A 63-cell matrix: 30 cells refuse, 27 of them outside P68's
  three. A `.INPUT` routed through a user object that forwards to `.stdin` refuses too.
- **N-2.** A halt that is pending while an access waits in `stdin_turn` runs its `CALL ON HALT`
  handler one clause late. `Message~halt` to a reader is a deterministic witness: HEAD is late
  30/30; base and the oracle are on time 30/30 each.
- **N-4.** P68 says "recorded as a divergence row". `phase-4-exclusions.txt` has no such row.

## Point 1: F-1 shapes

**Verdict: resolved for every shape the re-review listed.**

rr3's programs are byte-identical to mine; `diff` against `$CLAUDE_JOB_DIR/tmp/progs/` shows
no difference. 30 runs per side, interleaved:

| Shape | Oracle | HEAD |
| --- | --- | --- |
| `mainreads` | 30/30 `main [one]` / `w lines 1` | the same 30/30 |
| `pullstdin` | 30/30 `r [one]` / `main lines 1` / `main end` | the same 30/30 |
| `tworeaders` | 30/30 `main [two]` / `end one` | the same 30/30 |
| `tworeaders2` | 30/30 `r [one]` / `main [two]` | the same 30/30 |
| `tworeaders3` | 30/30 `r [one]` / `main [two]` / `main end` | the same 30/30 |

The shapes beyond rr3's list are in N-1.

## Point 2: the .STDIN guard as a lock

**Verdict: no deadlock, no lost wakeup and no hang found. Every probe ends, and on every
divergence from the oracle the output order is one HEAD always takes (30/30). The
divergences are N-2, which is new, and the halted read that answers nothing, which DEVIATIONS 10
already licenses.**

**A program that holds `.STDIN`'s guard itself and then PULLs cannot be written in Rexx here.**
The guard's scope is the Stream class's. Getting Rexx code to run in that scope on `.stdin` was
refused the same way on both sides:
- `::extension stream`: 99.916 on the oracle, and the same message here (rc 157 there, rc 120 here,
  the loud parse refusal);
- `.stream~define(...)`: 98.985 "User additions are not allowed to the REXX language classes" on
  both, rc 158;
- `.stdin~run(method)`: 97.2 (private method) on both, rc 159.
Stream's own methods are native, so no Rexx code runs inside them.
- A Stream subclass whose `LINEIN` does a `parse pull`, installed as `.input`'s destination,
  recurses on both sides (`sub1`; the oracle dies 2/3 by SIGSEGV, both end rc 245 otherwise).
- The subclass that forwards to `.stdin~linein` (`sub2`) refuses. That is N-1.

The remaining way in is a pinned `.stdin~` or `.input~` send that holds the guard. The matrix in
N-1 covers it.

Probes, 30 runs per side, interleaved. SIGINT goes to the PID at 0.5 s unless the row says otherwise:

| Probe | Shape | Oracle | HEAD |
| --- | --- | --- | --- |
| `hq1` | readers `a` and `b` started, `main` third; all PULL; no traps | 252, three 4.1 tracebacks, each with the monitor's `UNKNOWN` line, interleaved in five orders (DEVIATIONS 7) | 30/30 252: `a`'s traceback has the `UNKNOWN` line; `b`'s and `main`'s do not (N-2) |
| `hq1g` | the same, SIGINT to the group | 252, the same shape, five orders | 30/30 the same as `hq1` |
| `hq2` | the same three readers, each with CALL ON HALT; `main` reads twice | 30/30 ends with "timed out" (the oracle's second read waits for input that never comes), five orders | 30/30 rc 0 `a halted` / `a []` / `b [one]` / `b halted` / `main [two]` / `main halted` / `main2 [three]` |
| `hq3` | holder `a` (CALL ON HALT) reads twice; `main` queued, SIGNAL ON HALT, and its handler PULLs | 30/30 hangs until the driver's 20 s timeout. Its SIGTERM then ends 25 runs with 4.1 inside the handler's PULL (rc 252); the other 5 need SIGKILL | 30/30 rc 0 `a halted` / `a []` / `main halt` / `a 2 [two]` / `main after [three]` |
| `hq4` | holder `a` in `linein()`; `main` queued in `charin()`, then `lines()`, then `linein()`; CALL ON HALT on both | 30/30 `a halted` / `a [one]` / `main halted` / `main [74] 1` / `main2 [wo]` | 30/30 `a halted` / `a []` / `main [6F] 1` / `main halted` / `main2 [ne]` |
| `late_parsepullv`, `late_vlinein` | `a` holds; `main` CALL ON HALT, queued in PULL / `linein()` | 30/30 `main halted` before `main read [two]` (two orders, 19/11, of `a`'s line against `main halted`) | 30/30 `main read [one]` / `main halted` (N-2) |
| `eof2` | readers `a` and `b` started, `main` third; standard input closed at 0.5 s, no data | 30/30 `a []` / `b []` / `main []` / `a b` | identical 30/30 |
| `eof3` | `a` in `linein()`, `b` in `charin()`, `c` in PULL, `main` in `linein()`; `x` written at 0.6 s, closed at 0.8 s | 30/30 `a [x] 0` / `b [] 0` / `c [] 0` / `main [] 0 0` / `a b c` | identical 30/30 |
| `leak_sign`, `leak_nop`, `leak_call` | a started reader halted (SIGNAL ON HALT / no trap / CALL ON HALT) while main busy-loops, then main PULLs at 1.5 s; 5 runs each | `main [two]`, the halted read's `one` lost | `main [one]` 5/5 each: the halted read's chunk is filed for the next reader (DEVIATIONS 10). `leak_nop`'s two tracebacks are byte-identical to the oracle's. |

What these show about the lock:
- **A halted holder releases the guard.** `hq3`: the holder is halted. `main` is queued and then
  halted, and its handler's PULL still gets `three`, after `a`'s second read. `hq4`: `main`'s
  queued `charin()` goes on after the holder's halted `linein()`.
- **Halt and Ctrl-C with readers queued** neither hang nor refuse (`hq1`, `hq1g`, `hq2`). Every
  reader is served in queue order.
- **CALL ON HALT in a queued reader** runs, but one clause late (`hq2`, `hq4`, `late_*`): N-2.
- **EOF with several queued readers** is identical to the oracle (`eof2`, `eof3`).
- The `hq2` and `hq3` differences from the oracle are the halted read answering nothing, where
  the oracle's read in another thread is not interrupted and gets `one` (DEVIATIONS 10). They also
  include the oracle hanging, while HEAD finishes.

The deadlock refusal in `stdin_turn` (`input.rs:477-479`, 98.905) has no witness. ME in the
mutation table shows that deleting it changes no test. I found no way for the guard's owner to wait
on another activity while it holds the turn: the owner holds it only across a read.

## Point 3: PULL / PARSE LINEIN as Op::Exec, single activity

**Verdict: unchanged against base `4a20439fa`.**

Thirteen programs under `p3/` were run with standard input a file. They cover:
- TRACE R, I, A, L and C over PULL, PARSE PULL, PARSE LINEIN, PARSE UPPER LINEIN with templates,
  and PULL with a positional template (`t1`, `t2`, `t2a`, `t2l`, `t2c`, `t5`);
- RC and RESULT across PULL, PARSE LINEIN and `linein()` after a command and a CALL (`t3`);
- SIGNAL ON NOTREADY, CALL ON ERROR and SIGNAL ON SYNTAX around PULL and PARSE LINEIN (`t4`,
  `t5`), and `stream()` state and description after PULLs (`t4b`);
- interactive TRACE `?R` (`t6`);
- queue order (`t7`): PUSH and QUEUE then PULL and PARSE PULL, then PULL reaching standard
  input, then a QUEUE that PARSE LINEIN skips and the next PULL takes;
- a `.input` destination that is a user object, and its reset (`t8`);
- every stream builtin with and without arguments, traced (`t9`).

Results:
- With the path masked, HEAD's stdout, stderr and rc are byte-identical to base's for all 13.
- The same 10 of them, run with standard input a live pipe (writes at 0.3, 0.6 and 0.9 s, closed
  at 1.2 s, so the reads park), give identical hashes on HEAD and base.
- HEAD differs from the oracle in `t4`, `t4b`, `t9` (`lines()` and `chars()` counts after a
  read, already queued as pre-existing) and in `t6` (the oracle prints no `RC(127)` line under
  interactive trace). Base differs from the oracle the same way.
- `th1`-`th3` (TRACE R over a PULL, a `linein()` and a PARSE PULL halted by SIGINT at 0.3 s,
  input at 0.6 s), 3 runs each: HEAD and base give the same hash in each.

## Point 4: the moved HALT

**Verdict: the move takes no halt twice and none on the wrong activity. A halt that is
pending without waking the read is not moved at all, and it is taken one clause late (N-2).**

- **Not twice.** `mhalt2`: a `Message~halt` is pending on the reader, and then SIGINT at 0.6 s
  wakes its read. One `r halted from main` in 5/5 runs, as on the oracle 5/5.
- **Not the wrong activity.** `input.halted` is one flag for the whole interpreter. It is set
  by the woken activity's re-test (`input.rs:510-516`) and taken by the next `fill_stdin`. That
  activity runs straight from its re-test to its read. `leak_*` checks whether anything is left
  behind: a reader halted under each of the three trap kinds, and then another activity reads.
  In all three, the second reader gets the next line (`one`), not an empty interrupted read. The
  traceback of the untrapped kind is byte-identical to the oracle's.
- **A handler that reads.** In `hpull` and `hpull2` (single activity), a CALL ON HALT handler
  does its own PULL or `linein()` while the halted read is unwinding. HEAD and base agree 3/3:
  `handler [one]` / `main []`. The oracle's handler reads nothing (DEVIATIONS 10).
- **The gap.** `fill_stdin` moves a pending request only when `input.halted` is set
  (`input.rs:385-397`), that is, only when a signal woke this read. A request with no wake keeps
  its old activation. That covers a `Message~halt` to a reader waiting for its chunk, and a
  signal to an access queued on the guard. Its CALL ON HALT handler then runs after the next
  clause rather than at the end of the reading clause. See N-2.

## Point 5: the P68 residual cells

**Verdict: the three named cells refuse loudly and do not hang, but they are not the only
ones. No DEVIATIONS row exists.**

- Each of the three named cells (`mx_3_5` `.stdin~linein`/`.input~lines`, `mx_4_5`
  `.input~linein`/`.input~lines`, `mx_5_5` `interpret 'parse pull v'`/`.input~lines`), 30 runs:
  30/30 rc 120, stderr only `rexx-exec: a pinned wait for a guard that only an activity pinned
  below it can end is not implemented`, at most 1.02 s (when the input arrives).
- **They are not exactly 3.** The fixer's 35-cell matrix had no second access that waits pinned
  except `.input~`. With such accesses added, 30 of 63 cells refuse (N-1).
- **No DEVIATIONS row.** `docs/superpowers/plans/phase-4-exclusions.txt` ends its DEVIATIONS
  list at entry 12. No line in the file names P68, a pinned-both-sides refusal or stdin
  contention: `grep -n -i "P68\|inverted\|contention\|pinned below"` matches two lines, both
  unrelated (`:2816` CPU contention, `:4152` an inverted probe). The fix commits do not touch `docs/` (`git diff --stat
  4a20439fa 8faaf27b8 -- docs` is empty). N-4.

## New findings

### N-1 (High). F-1 still refuses for every read that waits pinned, against every access that waits pinned for the guard: 27 matrix cells beyond P68's three

**Where.**
- `builtin/stream.rs:49-53`: only `LINEIN` and `CHARIN` with no argument after the name, and
  `CHARS`, take the turn. `linein(,,1)`, `charin(,,n)` and `linein(,,0)` still send pinned.
- `stream()` (`builtin/stream.rs:236`) takes no turn.
- `input.rs:468`: a route that does not end at the bootstrap `.stdin` takes no turn, and so sends
  pinned.
Any such pinned read holds `.STDIN`'s guard in `pinned_wait(Input)` and runs the other activities
above itself. A second access that also waits pinned for the guard buries it, which is the
F-1 mechanism.

**Matrix.** `mx_R_S`: a started activity does access R, and main does access S at 0.2 s. One
run per cell per side. Readers R: 0 `parse pull`, 1 `linein(,,1)`, 2 `charin(,,1)`, 3
`.stdin~linein`, 4 `.input~linein`, 5 `interpret 'parse pull v'`, 6 `linein()`. Second accesses S: 0
`stream('STDIN')`, 1 `stream('STDIN','C','QUERY EXISTS')`, 2 `linein(,,1)`, 3 `charin(,,1)`, 4
`.stdin~lines`, 5 `.input~lines`, 6 `lines()`, 7 `parse pull`, 8 `linein(,,0)`.

| R \ S | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 `parse pull` | = | N-5 | = | = | = | = | = | = | = |
| 1 `linein(,,1)` | **120** | **120** | **120** | **120** | = | **120** | = | = | **120** |
| 2 `charin(,,1)` | **120** | **120** | **120** | **120** | = | **120** | = | = | **120** |
| 3 `.stdin~linein` | **120** | **120** | **120** | **120** | = | 120 (P68) | = | = | **120** |
| 4 `.input~linein` | **120** | **120** | **120** | **120** | = | 120 (P68) | = | = | **120** |
| 5 `interpret` PULL | **120** | **120** | **120** | **120** | = | 120 (P68) | = | = | **120** |
| 6 `linein()` | = | N-5 | = | = | = | = | = | = | = |

Key: `=` is identical to the oracle; 120 is the inverted-wait refusal, with no stdout. Every
refused cell, run once on pre-P66 `bf76afa1d`, ends rc 0 with the oracle's stdout. The one
exception is column 1, where both HEAD and pre-P66 answer `''` for `QUERY EXISTS` (N-5). So
these are F-1's regression, not shapes that never ran.

Also `sub2`: `.input~destination` is a Stream subclass whose `LINEIN` returns
`'sub:' || .stdin~linein`. A started activity and main each PULL. Oracle 3/3 and pre-P66 1/1:
`main [sub:two]` / `r [sub:one]`. HEAD 3/3 and base 1/1: rc 120.

**Failure scenario.** Two activities that each read a byte or a counted line from the console,
`x = charin(,,1)` (`mx_2_3`): rc 120 with no output, 30/30 runs at HEAD. The oracle runs it.
`mx_1_2` (`linein(,,1)` both sides) is the same, 30/30.

**What P68 licensed.** "direct .stdin~linein/.input~linein or a read inside INTERPRET against a
direct .input~ send; 3 of 35 matrix cells". Rows 1 and 2 are builtins, not direct sends.
Columns 0-3 and 8 are not `.input~` sends. The count came from a matrix whose second accesses
did not include one that waits pinned other than `.input~`.

**Fix direction.** Take the turn for every stream builtin whose stream resolves to `.stdin`
(all arguments, and `STREAM`), not only the no-argument forms. Either follow user routes that
forward to `.stdin`, or rule them into the licence. Then re-measure the matrix with the pinned
second accesses as columns.

### N-2 (Medium, from this round). A halt pending while an access waits in `stdin_turn` without a wake runs its CALL ON HALT handler one clause late

**Where.** `input.rs:385-397` moves pending requests to the reading activation only when
`input.halted` is set, that is, only when a signal woke this access's chunk wait
(`retest_stdin`, `input.rs:510-516`). Two cases are never moved:
- a `Message~halt` to a reader waiting for its chunk (halts from messages do not wake a park);
- a signal to an access queued on `.STDIN`'s guard (`wake_for_halt` wakes only `input_readers`,
  `scheduler.rs:2225-2233`).
In both, the handler runs after the clause that follows the read, not at the end of the read's
clause. The mechanism is inferred from the code; the observable is measured.

**Failure scenario** (`mhalt1`, deterministic, no signal):

```
o = .r~new; m = o~start('go')
call SysSleep 0.2
say 'halt sent' m~halt('from main')
call SysSleep 1.5
say 'r' m~result
exit
::class r
::method go
  call on halt
  parse pull v
  say 'r read [' || v || ']'
  say 'r next'
  return 'done'
halt: say 'r halted' condition('D'); return
```

`one\ntwo\n` written at 1.0 s, 30 runs each:

| | Output |
| --- | --- |
| Oracle | 30/30 `halt sent 1` / `r halted from main` / `r read [one]` / `r next` / `r done` |
| Base `4a20439fa` | 30/30 the same as the oracle |
| HEAD | 30/30 `halt sent 1` / `r read [one]` / **`r halted from main`** / `r next` / `r done` |

`mhalt3` (`v = linein()` in place of the PULL) is the same, 3/3 each side.

The signal case is `late_vlinein` and `late_parsepullv`: main queued behind a started holder.
HEAD 30/30 `main read [one]` / `main halted`. The oracle 30/30 has `main halted` before
`main read [two]`, and so does pre-P66 3/3 (`main halted` / `main read [one]`). `hq2` and `hq4`
show the same in queued readers. In the untrapped form (`hq1`), the queued activities'
tracebacks lack the monitor's `UNKNOWN` line and name `PROGRAM line N` where the oracle names
`REXX line 1457`.

**Fix direction.** Move a pending request to the reading activation whenever an access that
parked in `stdin_turn` resumes and reads, not only after a wake by halt. Witness: `mhalt1` as a
`stdin_contention` case, which needs no signal.

### N-3 (Medium, pre-existing, outside Task 21). CALL ON NOTREADY at the end of the default input stream is Error 43.1

`call on notready` then `v = linein()` (or `parse linein`, or `charin()`) at end of input. The
oracle prints `main notready` / `main []`, rc 0. HEAD, base and pre-P66 all end rc 213, with
`Error 43.1: Could not find routine "NOTREADY"` raised from the monitor's `UNKNOWN`
(`nt1`, `nt4`, `nt6`; standard input a pipe closed at 0.3 s, or `/dev/null`). In a method
(`nt2`) and in a started activity (`nt3`) it is the same. SIGNAL ON NOTREADY works (`nt5`,
identical on all four). The CALL trap is looked up in the activation that raised the condition,
the monitor's `UNKNOWN`, rather than in the caller's. It is not in `queued/` (the two NOTREADY
mentions there are about other streams and other properties) or in DEVIATIONS. It makes
`eof1` (EOF with queued readers under CALL ON NOTREADY) unusable as a probe, so point 2's EOF
probes have no NOTREADY trap.

### N-4 (Medium). P68's divergence row does not exist

P68: "stays a loud refusal (rc 120) where the oracle runs; recorded as a divergence row". There
is no such row in `phase-4-exclusions.txt` (point 5). When it is written, it should state the
extent N-1 measures, not "3 of 35".

### N-5 (Info, pre-existing). `stream('STDIN','C','QUERY EXISTS')` answers `''`; the oracle answers `STDIN`

Single activity, standard input a pipe: HEAD `[]`, pre-P66 `[]`, oracle `[STDIN]`. It is
the matrix's column 1 difference in rows 0 and 6.

## Mutations

How each mutant was run (`$CLAUDE_JOB_DIR/tmp/rr4/mut.py`):
1. Copy the file aside and assert the site occurs exactly once.
2. Write the mutant.
3. Run `cargo test --release -p rexx-exec --lib --test signals --test stdin_contention
   --no-fail-fast`. Each log has one `Compiling rexx-exec` line.
4. Copy the file back and `cmp` it against the saved copy.

After the set, `git status --short` shows only the lead's `progress.md`, and `git diff -- rust`
is empty.

Unmutated, at HEAD: lib 1000, signals 37 and stdin_contention 1, all passing (the gate run).

| Mutant | Claim | Result |
| --- | --- | --- |
| MA: `end_stdin_turn` releases nothing | the turn's guard is released | **red**: stdin_contention; lib and signals green |
| MB: the moved-halt loop in `fill_stdin` disabled | concern 4 | **red**: 6 signals tests (`call_on_halt_sees_an_interrupted_read`, `sigint_ends_a_charin`, `_parse_pull`, `_linein`, `a_halted_read_loses_no_input`, `a_halt_during_a_read_reaches_an_activity_that_ran_meanwhile`), stdin_contention |
| MC: `retest_stdin` reads `woken_by_halt` without clearing it | a halt is not taken twice | **survives** all three. Not witnessed by any test. `mhalt2` (point 4) is the probe a test would need. |
| MD: `end_stdin_turn` leaves `input.halted` set | a halted flag does not leak to the next access | **survives**. As point 4 found, the flag is taken by the woken activity's own read in the same run, so this clear is reached only where that read does not happen (a PULL served from the queue). |
| ME: no 98.905 deadlock check in `stdin_turn` | the documented error | **survives**; no shape reaches it (point 2) |
| MF: `parse_stdin_turn` reads the queue even while holding the turn | `queuemid` | **survives**. `pull_line`'s own check (the fixer's M3) covers the same case, so this guard is redundant on the paths tested. |
| MG: the `CHARS` builtin takes no turn | `chars()` waits its turn | **survives**. No case has `chars()` as the second access. |

The fixer's M8 (= MA) and M10 (= MB) reproduce. MC, MG and ME are unwitnessed claims. They are
recorded, not findings on their own, since point 2's probes show the behaviour holds at HEAD.

## Checks (P51)

At `8faaf27b8` (`git rev-parse HEAD` captured by the gate script), before any mutant. Target
dir `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/p6-scratch/t21rr4/target`.
Each status was written unpiped to a status file.

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0, with one `Checking rexx-exec`
  line in the log.
- `cargo test --workspace --release --no-run`: exit 0. Then `memcap 8G cargo test --workspace
  --release --no-fail-fast`: exit 0. 144 result lines: 3030 passed, 0 failed, 4 ignored.
- Not run: loom, pinning clippy, loom clippy (not in this review's bar).
- At the end, `git status --short` shows only the lead's `progress.md` (this report is new, and
  `.gitignore:30` ignores `.superpowers/`, so it needs `git add -f`). The scratch dir `p6-scratch/t21rr4` (HEAD target dir, both base
  trees and target dirs) is deleted.
