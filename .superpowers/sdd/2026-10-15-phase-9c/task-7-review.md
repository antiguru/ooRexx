# Phase 9c Task 7 review: standard stream buffering

Reviewed `6116a18f9..cc306292f`. I read the code from `git diff 6116a18f9 cc306292f -- rust/crates`
and skimmed the records and `refusal-sites.tsv`. `git diff --stat e95e41416 cc306292f -- rust/crates`
is empty, so the binary callgrind measured at `e95e41416` is the code under review. Probes used a
release `rexx` built at `cc306292f` in my own target dir. Each one ran from a fresh empty `mktemp -d`
directory, with the oracle under the standard wrapper and the crate under `memcap 2G timeout`. Each
shape got 1 run a side.

### Spec Compliance

- ✅ Spec compliant. Every brief step has its hunk:
  - the per-descriptor terminal test (`bin/rexx.rs:59`, `is_terminal`, not the FIFO test);
  - the 64 KiB bound (`lib.rs:351`, `2804`, `2815`, `2824`);
  - the hand-over before an ADDRESS command (`command.rs:765`);
  - no stdout hand-over from a stderr write (`lib.rs:2822-2826`);
  - only the bound check on the SAY path (`lib.rs:2810-2816`);
  - the tests (`tests/program_end.rs:117-374`);
  - Deviation 40 (`phase-4-exclusions.txt:2209-2263`);
  - the licence row (`suite-licences.txt:9`).
- ⚠️ The queued item `.superpowers/sdd/queued/2026-10-11-output-lost-on-kill.md` is closed in the
  report only, and the file is still in `queued/`. The controller should check that this is the
  convention for closing.
- Report concern 1 (Ruled 2 naming Deviation 38) is fixed in the working tree's
  `global-constraints.md`, which now reads "Deviation 40".

Checks I ran:

1. **Kill, stdout and stderr on separate targets** (`timeout -s KILL 2`, pipe `| cat >` and file `>`).
   Pipe and file gave identical bytes on both sides.

   | shape | oracle stdout / stderr | crate stdout / stderr |
   |---|---|---|
   | `call charout 'STDOUT','co'; call lineout 'STDOUT','lo'; .output~say('os'); call charout 'STDERR','ce'; call lineout 'STDERR','le'; .error~charout('partial'); do forever; end` | `colo\nos\n` / `cele\npartial` | nothing / `cele\n` |
   | `.error~charout('partial'); do forever; end` | nothing / `partial` | nothing / nothing |
   | START'd method `say 'thread'; .error~say('thread err'); do forever; end`, main `say 'main'` | `thread\nmain\n` / `thread err\n` | nothing / `thread err\n` |
   | `say 'a'; 'sh -c "echo b; sleep 4"'; say 'c'` | `a\nb\n` | `a\nb\n` (the command's streamed output is handed over at the idle) |
   | `say 'x'; call syssleep 5` | `x\n` | `x\n` |

   A partial stderr line is lost under a kill. Python 3.14.8 does the same:
   `sys.stderr.write('partial')`, then a loop under `timeout -s KILL 2`, delivers 0 bytes to a file
   and 0 bytes through `2>&1`. That is Python's behaviour and is covered by "still buffered is lost".
2. **Prompt before a read**: `say 'q'; pull x` with stdin `sleep 3 |`, killed at 2 s. Both sides
   deliver `q\n`. With `(sleep 0.5; echo ans) |` the crate prints `q` then `got ANS`.
3. **Every exit path, all three descriptors.** Each shape below ran in three modes: stdout and
   stderr to separate files, stdout through a pipe, and `2>&1 | cat`. The shapes were:
   - normal end: `say 'a'; .error~say('e1'); say 'b'`
   - `exit 3`
   - SYNTAX `x = 1/0`
   - `raise syntax 40.1 array('F')`
   - `raise error 7`
   - `trace r` mid-program
   - `'echo b'` and `'echo d 1>&2'` interleaved with `say`
   - HALT by SIGINT (`timeout -s INT 1`), both untrapped and trapped by `signal on halt`

   stdout, stderr and rc are byte-identical to the oracle in every case, separately for files and
   pipes. On a shared pipe every shape except the two commands' interleave differs as licensed.
   The stderr lines (the error report, trace and `.error~say`) come before the stdout that was
   still buffered. For example, SYNTAX on the oracle gives `a, e1, report`, and the crate gives
   `e1, report, a`. Python does the same with a traceback. The commands' shape matches the oracle
   (`a b c d e`).

   The START'd thread's two HALT reports come in a different order in stderr. The oracle itself
   gave a third, line-mixed order in its pipe run. That ordering is between threads and is outside
   this task.
4. **Deviation 40's figures match the records.** `wc -c` over `t7/step1` gives:
   - `say300` 2592 bytes;
   - oracle `say20000` 208894 bytes, crate 196618 bytes, whose last line is `line 18884`;
   - `trace` 55 bytes on both sides;
   - the crate's `order_lineout` is `b\na\nc\n`.
5. **Licence row**: the file's tab-separated format (`dir/group TEST owner reason`). I ran
   `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=utilities/rexx/rexx_command ... --test suite l3_core_suite`
   once. It exited 0, `verdicts.tsv` says `green`, and `tests.tsv` has
   `TEST_PASSINGARGS7 Deviation(40) passed failed`.
6. **callgrind**: `callgrind-e95e41416-summary.tsv` and the table match the report.
   - sayloop: head 179916965 against t6 179718921, which is +198044 Ir over 100000 SAYs, about 2 Ir
     a SAY. That is the bound check.
   - startup is flat.
   - At `e4bbcbba4` sayloop was +0.3072%, and `e95e41416` brings it to +0.1386%. That supports the
     `#[inline(never)]` on the cold trace wrapper (`route.rs:155`).
7. **Mutants** in a `git worktree add --detach` copy of `cc306292f`, each reversed by an exact edit
   with an empty `git diff --stat` before the next:
   - `say_bound` for a terminal set to `OUTPUT_BOUND` (`lib.rs:3789`):
     `a_terminal_standard_output_takes_each_line` went red (`left: ""`, `right: "started X\r\n"`).
   - `say_bound` for a redirected stdout set to `usize::MAX` (`lib.rs:3791`):
     `a_redirected_standard_output_keeps_what_it_handed_over` went red (`program_end.rs:272`).
   - `out_bound` set to `usize::MAX` and the stderr bound dropped (`lib.rs:3787`, `2824`): all 6
     `buffering` tests stayed green. See Minor 2.
8. `cargo test -p rexx-exec --test program_end`: 8 passed, in 8.01 s.

### Strengths

- The design is the smallest one that fits the ruling. The SAY path pays one compare against a
  per-run `say_bound`, which folds the terminal and redirected cases into one branch
  (`lib.rs:2810-2816`), and that is about 2 Ir a SAY as measured.
- The `hand_over_output` split (`input.rs:421-444`) also removes the old take-and-restore of
  `sinks`, which is simpler.
- Every stderr writer has a hand-over. All trace lines funnel through `route_trace_line`, so one
  hand-over in `route_trace_line_as` covers all of them, and the direct `trace.extend` sites
  (`trace.rs`, `run/interpret.rs:340`, `436`, `run.rs:3075`, `route.rs:206`) all end in
  `route_trace_line`.
- The terminal test runs through a real pty, with no tty needed, and witnesses Python's
  CR-as-line-end rule.
- The records keep oracle and crate bytes per shape and per target, and the report's revert
  evidence is specific.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

None.

#### Minor (Nice to Have)

1. **The fixed 2 s waits** (`tests/program_end.rs:182`, `326`) are report concern 2.
   - A debug `rexx` runs the 20000-SAY shape in about 0.08 s here, so the margin is about 25x.
   - On a slow or loaded machine, the positive tests fail.
   - `a_redirected_standard_output_loses_what_it_still_holds` (`:231`) passes vacuously, because a
     program killed before its SAY also has empty stdout.

   A cheap readiness marker fixes both:
   - **Negative shapes**: end the writes with `.error~say('ready')`. A stderr write never hands
     stdout over, so the marker does not disturb the subject. Have the stderr reader signal on the
     marker with a generous deadline (30 s), then kill.
   - **Positive shapes** (the stderr ones, the command ones, the bound one, and the terminal ones
     with expected bytes): poll for the expected bytes until the deadline, then assert the child is
     still running and kill it.
   - **The terminal `""` case** needs the stderr marker, so stderr should go to a pipe instead of
     `Stdio::null()` (`:317`).
   - Do not put a marker before `'sleep 4'` in the command shape. The hand-over runs after the
     marker, so a kill at the marker races it. Poll stdout for `started X\n` instead.
2. **Two of the brief's 64 KiB bounds are unwitnessed.** Disabling the non-SAY stdout bound
   (`lib.rs:3787`) and the stderr bound (`lib.rs:2824`) together leaves every test green.

   The behaviour itself is right. Under a kill on a pipe the crate gave:
   - `call lineout 'STDOUT', 'line' i` to 20000: 196618 bytes (oracle 208894);
   - `.output~charout` of 9 bytes 20000 times: 131076 bytes (oracle 180000);
   - `call charout 'STDERR'` of 9 bytes 20000 times, with no newline: 131076 bytes.

   Adding the `charout` STDOUT and STDERR shapes to `a_redirected_standard_output_keeps_what_it_handed_over`
   would witness both bounds.
3. **The leftover `sleep 4`** (report concern 3). `command.rs:619` puts the command child in its own
   process group, so killing `rexx` does not reach it. The crate captures the command's stdout on a
   pipe either way, so a command that writes until its reader dies ends with `rexx`. An example is
   `'while :; do echo; sleep 0.1; done'`, which dies of SIGPIPE on the first echo after the kill.
   With `with output stem lines.` it still exercises the hand-over before `start`.
4. **`phase-4-exclusions.txt:2247-2248`**: "and the same with `address system` and `with output stem
   lines.`" reads as two more shapes. The records hold one,
   `address system 'sleep 4' with output stem lines.`. Write that shape out.
5. **`route.rs:155`**: the `#[inline(never)]` has no reason beside it. The measured reason (inlined
   into `loop_advance`, +0.17% sayloop) is only in the commit message. A one-line reason would keep
   a later cleanup from deleting it, and the prose rules allow a decision to be stated.

### Assessment

**Task quality:** Approved

**Reasoning:** The ruling is implemented as Python behaves. Every exit path I ran matches the oracle
byte for byte per descriptor, and the cost on sayloop is the measured 2 Ir bound check. The open
items are test robustness (the fixed waits and two unwitnessed bounds) and two prose nits, none of
which changes behaviour.
