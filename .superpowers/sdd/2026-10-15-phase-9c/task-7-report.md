# Phase 9c Task 7 report: standard stream buffering

Status: DONE. Commits `e4bbcbba4` (implementation, Deviation 40, licence row, Step 1 records),
`e95e41416` (the trace hand-over kept out of line, a perf fix), and the commit carrying this report
(re-derived `rust/corpus/refusal-sites.tsv`, records).

## What changed

* `Sinks` gains `stdout_terminal`, set by `rexx` from `std::io::stdout().is_terminal()`.
* `Interp` hands each buffer over on its own (`hand_over_stdout`, `hand_over_stderr`;
  `hand_over_output` does both).
  * `SAY` (`write_out_line`): one compare, `out.len() > say_bound`. `say_bound` is 0 for a
    terminal, 64 KiB for a redirected standard output, `usize::MAX` without sinks.
  * Other standard output writes (`write_out`): past 64 KiB, or on a terminal when the write holds
    `\n` or `\r` (Python's own test).
  * Standard error (`write_err`, and each trace line in `route_trace_line_as`): when the write holds
    `\n` or `\r`, or past 64 KiB. A standard error write never hands standard output over.
  * Both before an `ADDRESS` command starts (`run_command`, before `start`). The existing
    hand-overs before a read, in `idle_until` and at exit are unchanged.
* Without sinks (every in-process run: corpus, lib tests) nothing is handed over, as before.
* Deviation 40 in `docs/superpowers/plans/phase-4-exclusions.txt` after 39.
* `rust/corpus/suite-licences.txt`: `utilities/rexx/rexx_command TEST_PASSINGARGS7 Deviation 40`.

## Step 1: oracle, and Step 3: this crate

Programs and raw outputs: `docs/superpowers/records/2026-10-15-phase-9c/t7/step1/` (`oracle/`,
`crate/`, one file per shape and descriptor target). Fresh empty directories, 1 run a side, killed
by `timeout -s KILL 2`, the descriptor on a pipe (`| cat >`) and on a file (`>`); pipe and file gave
the same bytes on each side. The crate runs were under `memcap 2G`, whose own "Killed" line on stderr
was filtered out.

| shape | oracle | crate before (6116a18f9) | crate after |
|---|---|---|---|
| `.error~say('started X'); do forever; end`, stderr | `started X\n` | nothing | `started X\n` |
| `trace r; x = 1; trace o; do forever; end`, stderr | 3 trace lines, 55 bytes | not measured | same bytes |
| `say 'started X'; do forever; end`, stdout | `started X\n` | nothing | nothing |
| `do i = 1 to 300; say 'line' i; end; do forever; end` | 2592 bytes | nothing | nothing |
| same to 20000, then `.error~say('done')` | 208894 bytes | not measured | 196618 bytes, a prefix ending `line 18884` |
| `say 'started X'; 'sleep 4'` | `started X\n` | not measured | `started X\n` |
| `say 'started X'; address system 'sleep 4' with output stem lines.` | `started X\n` | not measured | `started X\n` |
| `say 'a'; 'echo b'; say 'c'`, `2>&1 \| cat`, runs to its end | `a\nb\nc\n` | `a\nb\nc\n` | `a\nb\nc\n` |
| `say 'a'; call lineout 'STDERR', 'b'; say 'c'`, `2>&1 \| cat` | `a\nb\nc\n` | `a\nc\nb\n` | `b\na\nc\n` |

The shapes that differ under Deviation 40: the three redirected standard output shapes (output
still buffered at the kill is lost) and the `lineout 'STDERR'` shared-pipe shape. Python 3.14.8
measured the same day: `print` to a pipe and to a file lost under `timeout -s KILL 2`,
`sys.stderr.write` kept, and `print('a'); sys.stderr.write('b\n'); print('c')` through `2>&1 | cat`
gives `b\na\nc\n`. On a pseudo-terminal (a Python `pty.openpty` driver): `print` arrives at once,
`write('a')` alone does not, `write('a')` then `write('b\nc')` arrives as `ab\r\nc` (the whole buffer),
and `write('a\rb')` arrives (CR flushes too). The crate's terminal test asserts those four.

## Tests

`rust/crates/rexx-exec/tests/program_end.rs`, module `buffering`, spawning `CARGO_BIN_EXE_rexx`:
`standard_error_keeps_each_line` (both stderr shapes, pipe and file),
`a_redirected_standard_output_loses_what_it_still_holds`,
`a_redirected_standard_output_keeps_what_it_handed_over` (20000 lines: more than 64 KiB kept, fewer
than all, a whole-line prefix; the `.error~say('done')` after the loop arrives),
`a_command_starts_after_what_came_before`, `a_shared_pipe_interleaves_by_hand_over` (both ordering
shapes byte for byte: oracle's for `echo`, Deviation 40's for `lineout 'STDERR'`), and
`a_terminal_standard_output_takes_each_line` (through a pty from the existing `rustix` `pty`
dev-dependency, so no tty is needed).

Red with the change reverted, each by an exact edit, reversed and confirmed with `cmp`/`git diff`:

* `write_err`'s hand-over disabled: `standard_error_keeps_each_line` red (`left: ""`, right
  `"started X\n"`), `a_redirected_standard_output_keeps_what_it_handed_over` red (`done` lost),
  `a_shared_pipe_interleaves_by_hand_over` red (`a\nc\nb\n`).
* The hand-over in `route_trace_line_as` removed: `standard_error_keeps_each_line` red on the trace
  shape (`left: ""`).
* `stdout_terminal: false`: `a_terminal_standard_output_takes_each_line` red (`left: ""`).
* The hand-over before a command removed: `a_command_starts_after_what_came_before` red on the
  `with output stem` shape (`left: ""`). The plain `'sleep 4'` shape stays green under this mutant,
  because a command without redirection waits off the baton and `idle_until` hands over first.

## Step 5: callgrind

From `rust/`, binaries in `/home/moritz/dev/repos/ooRexx-p9-pin/` (`t6` is `rexx` built at
`6116a18f9`, this task's base):

```
memcap 8G bench-programs/callgrind.sh -B 0.5 -p 'sayloop startup' -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9c/t7b \
  pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run t6=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-9c-6116a18f9 \
  head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-9c-t7-e95e41416
```

exit 0. At `e95e41416` (`docs/superpowers/records/2026-10-15-phase-9c/t7/callgrind-e95e41416.txt`):

| program | pin | t6 | head | t6 d% | head d% | head -libc d% |
|---|---|---|---|---|---|---|
| sayloop | 179667983 | 179718921 | 179916965 | +0.0284 | +0.1386 | +0.0965 |
| startup | 119095416 | 119114890 | 119112297 | +0.0164 | +0.0142 | -0.1685 |

At `e4bbcbba4` the same command (output dir `cg-9c/t7`, table in `callgrind-e4bbcbba4.txt`) gave
sayloop +0.3072% against the pin. `cgdiff.py` against t6: `say_evaluated` +200015 (2 instructions a
`SAY`, the bound check) and `loop_advance` +299888, because the new two-line `route_trace_line_as`
wrapper was inlined into it with the stderr hand-over. `#[inline(never)]` on the wrapper
(`e95e41416`) restored `loop_advance` to its old size (0x190f bytes) and leaves sayloop at +0.1386%,
about 2 instructions a `SAY` over t6. No bench program writes standard error per line
(`grep -il -E "\.error|trace|stderr"` over `bench-programs/*.rex`, `pingpong/*.rex` and
`samples/rexxcps.rex` finds only rexxcps, whose `trace value tracevar` is `Off`), so no `-p` program
was added.

## Commands and exit statuses

`docs/superpowers/records/2026-10-15-phase-9c/t7/checks-statuses.txt`, unpiped, at `e95e41416` with
the re-derived `refusal-sites.tsv`:

* `cargo fmt --all --check`: 0
* `cargo clippy -p rexx-exec --all-targets -- -D warnings`: 0 (log shows `Checking rexx-exec`)
* `cargo test -p rexx-exec --test program_end`: 0, 8 passed
* `cargo test -p rexx-exec --test rexx_command`: 0, 1 passed
* refusal tables (`owners`, `closed_phases`, `phase_ten_reasons`, `representation_refusals`,
  `refusal_sites`, `refusal_dispositions`): 101 first (`refusal_sites`: `lib.rs` line numbers moved
  by the new constants), then `REXX_REFUSAL_SITES_REFRESH=1 ... --test refusal_sites` 0, the diff
  48 rows, every one a `lib.rs` line number only, then all six 0
* `cargo test -p rexx-exec --lib`: 0, 1142 passed
* `cargo test --release -p rexx-exec --test suite`: 0, 52 passed
* `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=utilities/rexx/rexx_command
  REXX_SUITE_TABLE=docs/superpowers/records/2026-10-15-phase-9c/t7/suite/table.tsv ... --test suite
  l3_core_suite`: 0, verdict green, `TEST_PASSINGARGS7` row `Deviation(40)`, oracle passed, crate
  failed. With the licence row removed: 101, verdict red, `not passed: TEST_PASSINGARGS7`.
* `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus`: 0, 29 passed,
  1 ignored
* every other test binary that spawns `rexx` (`buffer_growth_error_exit`, `input_oracle`,
  `package_requires`, `signals`, `datetime_zone`, `sim_processes`, `instance_copy_memory`,
  `library_routine_memory`, `prompt_before_read`, `release_local_reference`, `rexxutil_process`,
  `stdin_contention`), `--release --no-fail-fast`: 0

## Queued item

`.superpowers/sdd/queued/2026-10-11-output-lost-on-kill.md` is closed: standard error written before
a kill reaches its descriptor, as on the oracle, and what a redirected standard output loses is
Deviation 40's.

## Concerns

1. `global-constraints.md` Ruled 2 still says "Deviation 38 licenses what a kill loses"; 38 is the
   DO OVER keyword deviation and this one is 40. Controller's file, not edited.
2. The kill tests wait a fixed 2 s and assert the process was still running; a debug `rexx`
   reaches its loop in about 0.2 s here, so a machine more than ten times slower would fail them
   rather than hang.
3. `a_command_starts_after_what_came_before` leaves its `sleep 4` running for up to 2 s after the
   kill (memcap reports the leftover cgroup).

## Fix round 1

Review: `task-7-review.md` (approved, minors). Each minor fixed:

1. **No fixed waits.** `killed` in `tests/program_end.rs` now polls one descriptor for a marker and
   kills once it is there, panicking with "never wrote" after 30 s. The negative shapes end with
   `.error~say('ready')`, which hands nothing on standard output over. The positive shapes poll for
   their own expected bytes. The terminal `""` case sends its marker to a stderr pipe. The module ran
   in 1.02 s, where it took 8.01 s before. To show the "loses" test is no longer vacuous, a scratch
   edit made its first program `do forever; end; say 'started X'; .error~say('ready')`. The test
   then failed after 30.01 s with `held never wrote "ready"` instead of passing. The file was
   restored from a copy (`cmp` identical).
2. **Both remaining bounds witnessed.** `a_redirected_standard_output_keeps_what_it_handed_over`
   gains `call charout 'STDOUT'` and `call charout 'STDERR'`, each writing 9 bytes 20000 times with
   no newline. The STDERR shape's marker is a `say 'ready'` on a terminal standard output. Mutants,
   each reversed from a copy and checked with `git diff --stat`:
   * `out_bound = usize::MAX` (`lib.rs:3787`): red, `0 bytes kept: do 20000; call charout
     'STDOUT', ...`.
   * The stderr bound dropped (`lib.rs:2824`, `if ends_a_line(bytes)`): red, `0 bytes kept: do
     20000; call charout 'STDERR', ...`.
3. **No `sleep 4` left behind.** The command shapes run `while :; do echo; sleep 0.1; done`, which
   dies of SIGPIPE once `rexx` is gone. The command test polls standard output for `started X\n`.
   With the hand-over before `start` removed it fails, `slow-command never wrote "started X\n"`
   after 30.03 s, on the `with output stem` shape. The full module run printed no memcap leftover
   line. The mutant run printed one, from the mutant's own 30 s run. `pgrep -af "while :; do echo"`
   afterwards found only its own shell.
4. **Deviation 40** gives the `address system 'sleep 4' with output stem lines.` shape on its own
   row, as the records hold it.
5. **`route.rs`**: one comment line beside `#[inline(never)]` with the measured reason (sayloop
   +0.31% inlined, +0.14% out of line).
6. **Queued item closed** in the file, with a `**CLOSED 2026-10-17 by Phase 9c Task 7 (e4bbcbba4).**`
   head as `2026-10-13-class-side-string-method-not-sent.md` has.

Checks, unpiped (`docs/superpowers/records/2026-10-15-phase-9c/t7/checks-statuses-fix1.txt`):
* `cargo fmt --all --check`: 0
* `cargo clippy -p rexx-exec --all-targets -- -D warnings`: 0
* `program_end`: 0, 8 passed
* `rexx_command`: 0
* refusal tables: 0
* `--lib`: 0, 1142 passed
* `--release --test suite`: 0, 52 passed
* the rexx_command group through the harness: 0, verdict green
* `REXX_CORPUS_GATE=1 --release --test corpus`: 0, 29 passed, 1 ignored

No code outside tests changed except a comment, so the callgrind figures stand.
