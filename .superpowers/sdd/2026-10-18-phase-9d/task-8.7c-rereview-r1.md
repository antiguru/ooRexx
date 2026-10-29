# Phase 9d Task 8.7c re-review of fix rounds 1 and 2

Reviewer t87c-rr1. Code BASE `e620097f0`, code HEAD `8dd9a4ae0` (commits `1462fd46f` `6da312e62` `246ab583d`
`3c8978185` `9af69058d` `8dd9a4ae0`); `rust/` at the checkout's HEAD `f98b83015` equals `8dd9a4ae0`
(`git diff --stat 8dd9a4ae0 HEAD -- rust` empty).

**Verdict: CHANGES REQUESTED** (Critical 0, Important 2, Minor 2). Every brief item is ADDRESSED; the
two Important findings are new, and both come from round 1's I1 change.

## What I ran

* `git archive` trees of `e620097f0` and `8dd9a4ae0` (`rust interpreter extensions`), each built with
  `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 --bin rexx` into
  `/tmp/claude-1000/t87c-rr1-target-base` and `-head`, exit 0 both. The head binary's `.text`
  (`objcopy -O binary -j .text`) is byte-identical to the pinned `cg-9d/t8.7c/rexx-head-8dd9a4ae0`.
* Probes three ways from fresh directories under `t87c-rr1-scratch/` (`bin/r3.sh`: `o` oracle under the
  standard wrapper, `b` the `e620097f0` build, `h` the pinned `rexx-head-8dd9a4ae0`, crate under
  `memcap 2G timeout -k 5 20`; stdout, stderr and rc kept apart under each `out/`). The `b` build is
  deleted now, so rerunning `r3.sh` needs it rebuilt. Shapes of `oracle-crashes.txt` 64 to 67 and the
  I2 probes ran on the crate only.
* From the head archive tree (with read-only symlinks to the checkout's `build`, `ootest`, `api`,
  `docs`, `samples`, `testbinaries`, `oodocs`), `CARGO_TARGET_DIR=/tmp/claude-1000/t87c-rr1-target-head`,
  `--locked`:
  * `REXX_CORPUS_GATE=1 memcap 12G cargo test --release -j 4 --no-fail-fast -p rexx-exec --lib --test
    input_oracle --test stream_model`: 101. Lib 1210 passed, 1 failed:
    `builtin::rexxutil::files::tests::a_move_across_file_systems_copies_and_unlinks` (environmental:
    "needs ... target/tmp ... and /tmp ... on different file systems", the target dir being on /tmp).
    input_oracle and stream_model ok. Every witness the rounds name ran and passed, read from the
    log: `tests::a_stems_string_value_is_its_defaults_and_nostring_names_the_stem`,
    `tests::a_stems_string_sends_string_to_its_default_and_a_trace_line_names_it`,
    `tests::a_hash_lookup_sends_the_identical_indexs_equality`,
    `tests::a_join_rendering_itself_through_defaultname_runs_out_of_stack`,
    `run::tests::conditions::a_condition_raised_while_its_handler_runs_is_held` (the
    `delayed_trap_cases/crate` cases), `the_first_seek_of_an_unopened_transient_stream_is_refused`,
    `the_unfiled_ruled_programs_answer_as_this_crate_rules`, both stream_model tests.
  * `REXX_CORPUS_GATE=1 ... --test corpus`: 101, "1233 of 1234 matching", the one being the known
    `lang/rexxutil_file_date_time.rex`.
  * `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=t87c-rr1-scratch/suite/table.tsv memcap 8G cargo test -j 4
    --release -p rexx-exec --test suite l3_core_suite`: 0. `tests.tsv`, `table.tsv`, `verdicts.tsv`,
    `oracle-differences.tsv` byte-identical to `t8.7c/fix-r2/suite/` (`cmp`);
    `base/keyword/TRACE TEST_TRACE_FORWARD - passed passed`.
* A mutation build (`/tmp/claude-1000/t87c-rr1-src-mut`, the header-pause arm's guard
  `&& !self.activity.pending_traps.is_empty()` in `ir/drive.rs:2583` made `&& false`).
* Callgrind on arith and parse myself (Perf).

## Brief items

| item | verdict |
|---|---|
| I1 (rendering failures propagate) | ADDRESSED for every review shape; see N1 for the reference path |
| I2 (a routine's own CALL ON NOTREADY) | ADDRESSED |
| M1 (non-string DEFAULTNAME answer renders by name) | ADDRESSED |
| M2 (`leveled_store_at` indirection) | ADDRESSED |
| M3 (doc blocks in `dispatch/stream.rs`) | ADDRESSED |
| R1 (stem string value, NOSTRING names the stem) | ADDRESSED |
| R2 (identical index's `==` sent, pinned) | ADDRESSED |
| R3 (APPEND extent) | ADDRESSED; the saturation is M-a |
| R4 as replaced (traps OFF only while the typed line runs) | ADDRESSED |
| R5 (93.958 pinned) | ADDRESSED |
| R6 narrowed (Stem~string, TRACE keeps the name) | ADDRESSED |
| round 1's R4 and R6 code removed | ADDRESSED |
| perf | ADDRESSED |

**I1.** Review probes `c1/e1`, `e3`, `e4`, `e5` and `c1r/r4` rerun: o~h equal on stdout, stderr and rc,
o~b differ (`r4`: oracle and head rc 245 ending `Error 11.1`, base rc 0 `a REC|after`). Further
renderings of a raising `DEFAULTNAME` (`i1e/m.rex`: SAY, `||`, `length`, `substr` argument, a String
method argument, `1 + o`, `do o`, `o~string`, `o~nosuch`, `~toString`, a stem default's `~string`) all
answer `98.900 boom.` on both. Two differ: N1 and N2.

**I2.** `nr/n1.rex`, crate only: head `hn2 NOTREADY no_such_file_zz2.txt` / `rn after its linein` /
`hn 1 ...`, base without the `hn2` line; `n2.rex` (SIGNAL ON in the routine) equal at both. The
`same_notready_trap` test by `Rc::ptr_eq` of the label holds for the shapes I reasoned through
(a routine's own `CALL ON ANY` while the caller's NOTREADY trap is inherited still counts as
inherited, `CALL OFF` stops the chain).

**M1.** `c1/e6.rex`: head prints `an Array` for both `OBJECTNAME` and `DEFAULTNAME` answers in the join,
the trace line and the 97.1 insert; the oracle's remaining differences are its raw-byte join (the
review's oracle defect) and `.on~new~string` answering the array, as the report says.

**R1.** `r1a/a.rex`, `r1d/e.rex`, `r1e/f.rex`: under SIGNAL ON NOSTRING, head raises NOSTRING naming the
stem (`condition('o')~additional` `Stem`, or `USTEM` for a subclass instance), description its `STRING`
answer (`NSTR`), in SAY, builtin arguments (`substr`, `reverse`, `pos`, `d2x`, `x2c`, `abbrev`,
`translate`), both concatenation operands, abuttal and blank, a comparison's right operand, `1 +`,
PARSE VALUE, PARSE VAR, INTERPRET, PUSH, QUEUE, a DO count, NUMERIC DIGITS, SIGNAL VALUE, TRACE VALUE,
ADDRESS VALUE and `CALL (v.)`; with a string default (`s. = 7`), an array default and a `makeString`
default each position uses the default's string value (`one\ntwo`, `KM`, `3`). Chained defaults
(`b. = a.`) and a `Stem` subclass instance follow. Untrapped, `say v.` is `NSTR` (the stem's `STRING`).
The oracle raises NOSTRING for every plain stem (the recorded oracle defect). One position differs
for every object, not only stems: a queue candidate below.

**R3.** `r3a/a.rex` (12-byte file, OPEN WRITE APPEND): `seek =1`, `=12`, `<1`, `-1` write char and
`=2`/`=3 write line` answer 0 NOTREADY; `=13` and `<0` move; `charout('abc',5)` 3 NOTREADY,
`charout('abcd',12)` 4 NOTREADY, `charout('X',13)` 0 READY, `charout(,3)` 0 NOTREADY; `lineout` with
line 1, 2, 3 and `lineout(,1)` 1 NOTREADY; nothing written by the refusals. Reopen after close
(`both append` on the grown file) takes the new size as extent; a file grown by another stream object
after the open accepts a SEEK past the old extent (`=19` on a file grown 13 to 17 bytes), as "the
file as it was at open" says; `open write` then `open write append` without a close refuses `=1`.
`lines('C')`/`chars` after the refusals count from the read position (`r3c/c.rex`). A SEEK naming no
pointer on BOTH APPEND (`seek('=1')`) is refused whole, read pointer included, which is the stream
model's `wr & within_extent` rule. The stream_model tests pass with the `append-extent` rule.

**R4 as replaced.** The debug EOF programs (`debug_input_eof`, `debug_input_eof_call_on`,
`debug_input_eof_call_on_loop`, `debug_input_stdin_eof_call_on`, `debug_input_eof_iterate`,
`debug_input_eof_leave` with its `.stdin`, `debug_input_eof_iterate_outer`,
`debug_handler_exit_at_iterate`) and `debug_typed_traps_off.rex` with its `.stdin`: o~h equal on all
three descriptors (o~b equal too: `e620097f0` predates round 1's change). Their entries are back in
`corpus/phase-6-1.txt`; `corpus/unfiled.txt` and `input_oracle/ruled` hold only
`stream_append_writes_at_end.rex`. Under the header-pause mutant, `debug_input_eof_call_on_loop`,
`debug_input_eof_iterate`, `debug_input_eof_iterate_outer` and `debug_handler_exit_at_iterate` differ
from the oracle on stdout and stderr, the rest stay equal: the arm is witnessed again.
`debug_typed_traps_off`'s stdout shows each claimed path (`typed ZZUNSET`, `ru 5`, `typed  done`,
`hn STDIN 8`). Round 1's window is gone: `replace_debug_pause` is called only by
`run_debug_fragment` (`run/interpret.rs:398`, `:427`), as at `e620097f0`, `debug_input_line` is a plain
send, and `drop_queued_for_delayed` has no definition or caller (`grep -rn`).

**R5.** `r5/q.rex`: oracle `seek 1 READY`, base and head 93.958 rc 163. **R2.** `r2/h.rex`: `== called`
on the second put and the read at base and head, never on the oracle.

**R6 narrowed.** `r6a/b.rex` (stemarr): head `v.~string` is `NSTR` where oracle and base say `a NSTR`;
every TRACE line equals the oracle's (`>V>   V. => "a NSTR"`). `r6b/s2.rex`: `k.~string` is `KS`,
`t.~string` `an Array`. The TRACE sites render through `traced_string_value_text`;
`string_representation` is reached only from `stem_string_representation` (`grep -rn`).

## Findings

### Important

**N1. A rendering of a variable reference sends nothing: round 1 regressed an oracle-matching output
and left I1's shape open through a reference.** `to_text`'s `Redirect::VarRefValue` arm
(`value.rs:763`) and `text_len_inner`'s (`value.rs:309`) now call `unsent_string_value_text`, where
`e620097f0` called `string_value_text`, which sent `DEFAULTNAME`. Every rendering that reaches a
reference through `to_text` (an Array join's item, `~string` of a reference, a TRACE line naming a
reference) now shows the referenced object's default name. `t87c-rr1-scratch/i1b/g.rex`:

    o = .dn~new; r = >o   with  ::method defaultname; say '  (dn sent)'; return 'DN'
    say 'arr' .array~of(>o)~makestring    oracle, base: (dn sent) / arr DN    head: arr a DN
    say 'str' r~string                    oracle, base: (dn sent) / str DN    head: str a DN
    trace i; z = r                        oracle, base: >V>   R => "DN"       head: >V>   R => "a DN"

The rule gives `DN` too: a reference's string value is its value's (Deviation 72), and with no
`makeString` that is the `STRING` answer, `Object`'s, which is `OBJECTNAME` and so `DEFAULTNAME`.
`rexx-head-246ab583d` prints the same as head, so round 1 introduced it (report concern 5 said no
probe reaches these renderings). With a raising `DEFAULTNAME` (`i1d/k.rex`) the oracle traps
`98.900 boom.` in all three renderings; base and head print `a BAD` and run on, so I1's rule does not
hold for a reference. Fix: render a reference's referenced value by `string_value_text` and
propagate its `Failure` from the renderings that can carry one.

**N2. RAISE renders its RC, DESCRIPTION and ADDITIONAL values whether or not a trace line will print
them, and since round 1 that rendering raises.** `run/condition.rs:1170`, `:1189` and `:1232` call
`traced_string_value_text(value)?` unconditionally; `trace_keyword` gates only the printing. The send
itself predates the rounds (base also sends `DEFAULTNAME` untraced, `i1g/p.rex`: oracle prints nothing,
base and head print `(noisy dn sent)` on stdout), but a raising `DEFAULTNAME` now changes the outcome of
an untraced RAISE. `i1f/n.rex`, each under SIGNAL ON SYNTAX:

    raise syntax 93.900 additional (o)    oracle, base: 98.939 Additional information for SYNTAX errors ...
                                          head: 98.900 boom.
    raise syntax 93.900 description (o)   oracle, base: rc 163, Error 93.900 untrapped
                                          head: trapped 98.900 boom.

Gate the three renderings on `self.trace_mode().results`, as `trace_redirect_target` and
`echo_header_value` do.

### Minor

**M-a. The APPEND extent saturates at 4 GiB, so R3 does not hold on a larger file.** `body.rs`
`append_extent: u32`, `dispatch/stream.rs:834` `u32::try_from(size).unwrap_or(u32::MAX)`.
`t87c-rr1-scratch/r3b/b.rex` on a sparse 5368709120-byte file opened WRITE APPEND, crate only:
`seek =4294967290 write` `0 NOTREADY`, `seek =4294967296 write` `4294967296 READY`, `seek =5000000000`
`5000000000 READY`, both positions within the file as it was opened. The field's doc states the
saturation (round 1 concern 4); the ruling has no such limit. A sentinel (`u32::MAX` meaning "read
the size recorded at open") or a `u64` outside the hot struct would keep the padding.

**M-b. `eval`'s comment says it is the single insertion point of TRACE I's value events
(`eval.rs:101`-`:102`); `eval_in_parse` (`eval.rs:124`) is a second one.** A false sentence; it should
name `eval_in_parse`, the copy a change to `eval`'s depth bookkeeping must also make.

## Concern 5 (code quality only)

`eval_in_parse` repeats `eval`'s prologue and depth bookkeeping and differs only in how the trace hook's
`Result` returns. The round measured the shared `#[inline(always)]` helper (`fix2-exp-g`: parse
+0.1172, pingguard +0.1051) and kept two bodies. Acceptable as a measured trade, but the copies are
unlinked: nothing fails if one changes. With M-b fixed, a `debug_assert` or a test that runs PARSE
patterns and an ordinary expression to the same `MAX_EVAL_DEPTH` refusal would bind them.

## Perf

* `fix-r2/callgrind.txt`: every `.vg` in `cg-9d/t8.7c/fix2-final2` names
  `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7c/rexx-head-8dd9a4ae0` (head) or
  `rexx-base-97eee8a9a` (base), 99 each (`grep -h 'Command:' ... | sort | uniq -c`); every `.rc` is 0.
  Libc subtracted, every program at most +0.0403% (arith); the largest spread is 0.0045% (define, base), under the script's 0.01%, so no SPREAD flag
  (pingguard +0.0009, pingsem +0.0352).
* Re-measured: `bench-programs/callgrind.sh -B 0.5 -r 3 -j 6 -p "arith parse" -o
  /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7c-rr1 base=.../rexx-base-97eee8a9a
  head=.../rexx-head-8dd9a4ae0`, status 0, every `.vg` naming those two binaries:

| program | head d% | head -libc d% | spread% |
|---|---|---|---|
| arith | +0.0382 | +0.0403 | 0.0000 |
| parse | +0.0371 | +0.0396 | 0.0000 |

  Equal to the report's figures to the fourth decimal.
* N2's fix adds a branch on the RAISE path only; N1's on the reference rendering path, which is cold.

## Fixture test under load

`fixtures::a_child_refusing_on_the_inherited_stderr_leaves_the_run_ended_normally` passed every time:
alone (`--exact`) in repeated sequential runs beside two `rexx_exec` lib-test binaries at
`--test-threads 32`, and with the whole `fixtures::` module in parallel, five runs plain and five under
the same lib-test load (load average up to 11.75). Not reproduced. One of the loaded module runs failed
a different test, `fixtures::a_command_a_test_leaves_running_is_killed_with_the_run` ("the command
outlived the run", `suite.rs:2002`): `grandchild_sleeps` scans every process in `/proc` for
`sleep 41.5`, so a grandchild of an earlier run not yet reaped reads as this run's. Harness code, not
this task's.

## Queue candidates (pre-existing at `e620097f0`, outside the rounds)

1. **Stem `[]` does not take its indexes' string values.** collclasses.xml Stem `[]`: "concatenates the
   index string values". `r1c/d.rex`: `a.KM = 'km-tail'; say a.[kk]` with `kk`'s `makeString` `KM`:
   oracle `km-tail`, base and head `A.a KMAKE`; under SIGNAL ON NOSTRING `a.[nn]` (no `makeString`)
   oracle NOSTRING, crate `A.a NSTR`; a reference index (`r1b/c.rex`) the same. A stem index
   follows from it (`a.[k.]`, `a.[v.]` in `r1a`).
2. **LOSTDIGITS' description for an object operand.** `i1c/l.rex`, `numeric digits 5; say 1 + k` with
   `k`'s `makeString` `1234567890` and a `DEFAULTNAME`: oracle `LOSTDIGITS 1234567890`, base `DNK`
   (sent), head `a KN` (round 1 made `lostdigits_raise` and the fuzz raise unsent,
   `run/condition.rs:473`-`:531`). Both crate answers are wrong; the operand's string value is the rule's.
3. **`grandchild_sleeps` is process-wide** (above).

## Cleanup

`/tmp/claude-1000/t87c-rr1-target-base`, `-head`, `-mut` and the archive trees
`/tmp/claude-1000/t87c-rr1-src-base`, `-head`, `-mut` deleted after this review.
