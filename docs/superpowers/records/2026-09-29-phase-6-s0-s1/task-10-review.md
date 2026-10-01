# Task 10 review (92ac5c054..0f5aef01c)

Reviewer: p6-t10-review. Builds: `git archive` of each commit under
`$S/p6-t10-review/{new,old}`, touched, separate target dirs, both logs carry `Compiling rexx-exec`.
The probe harness `$S/p6-t10-review/cmp.sh` runs each probe alone in a fresh directory on the oracle,
on 92ac5c054 and on 0f5aef01c, and compares stdout, stderr (run paths normalised) and rc.
`$S` = `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad`.

**Spec compliance: FAIL on P20** (one Important finding). The brief's steps are otherwise met.
**Task quality: behaviour sound, two Important findings** (a false perf claim, and the perf rise is
attributable) plus Minors.

## Findings

### Important 1: P20, the seam trait

`rust/crates/rexx-exec/src/scheduler.rs:56-83` (the trait) declares all ten methods under a
trait-level `#[expect(dead_code, reason = "the seam methods the single-activity driver does not call")]`.

- Methods with a caller: `spawn` (`ir/drive.rs` `split_continuation`), `park` (the
  `ExecOutcome::Park` arm in `region_ops!`) and `run_until_park` (`drive`'s `Exit::Suspended` arm).
- Methods with no caller: `unpark`, `yield_at_slice`, `exit_for_native`, `exit_for_block`,
  `post_completion`, `request_baton` and `stop_the_world`.
- Among the called three, `SingleActivity::spawn` answers `None` and `SingleActivity::park` is
  empty. P20 also excludes both ("no empty or None-answering methods").
- There are more dead_code allowances: `ExecOutcome::Park`, `ExecOutcome::Split` and
  `ParkReason::Guard` each carry `cfg_attr(not(test), expect(dead_code, ...))` (`scheduler.rs:22-43`).
- The trait docs of the seven uncalled methods describe a baton, slices, native exits and
  completions that do not exist ("releasing the baton", "A callback's thread asks for the baton").
  This is forward-looking prose.

Fix: delete the seven uncalled methods, their docs and the trait-level `expect`. For
`spawn`/`park` and the three cfg_attr allowances, the controller rules under P20.

### Important 2: false claim in the perf doc and the report, "no bench program has a plain DO"

`docs/superpowers/plans/phase-6-perf.md` (the `## Task 10` section) and task-10-report.md say "No
bench program has a plain DO block (`grep -E '(then|else|^) *do *$'` over
`bench-programs/*.rex` finds none), so the moves do not come from running a flattened block."
rexxcps is in the same table, but it lives in `bench-rexxcps/`, which that grep does not search.
`bench-rexxcps/rexxcps.rex:54` is `        if j<5 then do   /* This path taken */`, a plain DO
inside the timed inner loop. The trailing comment also defeats the `do *$` pattern.
`rexxcps.rex:100` (`else do`) is a second one. Probe:
`/bin/grep -a -n -i -E '(^|;|then|else|otherwise)[[:space:]]*do[[:space:]]*($|;)' rust/bench-rexxcps/rexxcps.rex`,
or read line 54.

Fix: delete the sentence (both files), or narrow it to the programs under `bench-programs/`. As
it stands, rexxcps's +0.06% includes running a flattened block.

### Important 3 (perf, attributed, not optimised): the per-clause Ir rise comes from code the benchmarks never run

These are callgrind runs of `nop.rex` and `assign.rex` with `n = 50000` (`$S/p6-t10-review/cg/`):

| build | nop Ir | assign Ir |
|---|---|---|
| 92ac5c054 | 580,259,239 | 1,060,562,844 |
| 0f5aef01c | 595,295,275 | 1,095,599,377 |
| control A: 0f5aef01c with the `Park`/`Split` arms of `region_ops!`'s `Op::Exec` arm replaced by one loud `Ok(_)` arm, **and** the `Park { at: match deliver { Deliver::Wake => ... - 1, _ => ... } }` epilogue reverted to `at: op_after(...)` | 580,259,380 | 1,060,563,298 |
| control B: only the `at: match deliver` epilogue reverted | 580,254,636 | 1,080,563,551 |

- Line-level annotation shows every *identified* source line in `ops_loop_steady` with identical
  counts old vs new. The rise is in callgrind's "unidentified lines" of `drive.rs` (+15.05M on nop,
  +20.05M on assign), i.e. compiler-generated code. No new check executes on these paths. NOP and
  assignment do not go through `Op::Exec`, and `ops_loop_steady`'s frame is 0x688 in both builds.
- The cause is two diff hunks that nop and assign never execute. The `Deliver::Wake` `at` match in
  `region_ops!`'s shared Park epilogue (`ir/drive.rs:1652-1655`, the `Deliver::Wake => op_after(...) - 1` line is 1653) accounts for all of nop's +15M and
  about 43% of assign's +35M. The new `ExecOutcome::Park`/`Split` arms (`ir/drive.rs` in the
  `Op::Exec` arm of `region_ops!`) account for the rest of assign's.
- varlookup was not measured here.

A candidate that stays within the brief is for the Park arm to hand its own `at` (it is
`region_op`'s own position) instead of the shared epilogue matching on `deliver`. It is the perf
agent's call. It is not optimised here.

### Minor 1: `split_continuation`'s doc is false for `Some`

`ir/drive.rs` `split_continuation` doc: "Hands the continuation ... to the activity the scheduler
makes for it." The `Some` arm is a loud `op_not_driven`, so nothing is handed to any activity.
Fix: delete the first sentence and keep "With none made, the continuation stays with this activity"
(or state that `Some` is loud).

### Minor 2: the test-only Split cannot split the instructions that would split

`run.rs` `exec_instruction`'s `#[cfg(test)]` arm answers `Scripted::Split` by running `exec_flow`.
GUARD and REPLY moved out of `exec_flow`, so a scripted Split on a REPLY or GUARD clause reaches
`other => Err(Loud::instruction(other))`. That is REPLY, the one instruction whose continuation
really does go to another activity. The test-only harness cannot express it. There is no
production effect. Fix: route the scripted Split through the same match as the unscripted path,
and map `Done(flow)` to `Split(flow)`.

## Checks that found nothing

### 1. Flat plain DO vs oracle and vs 92ac5c054

There were 45 probes (`$S/p6-t10-review/probes*/`). Every probe is byte-identical between 0f5aef01c
and 92ac5c054 on stdout, stderr and rc. Every probe is identical to the oracle except these four,
each identical on 92ac5c054 and so pre-existing:

- `do_reply`: REPLY inside DO is refused.
- `signal_into_end` and `signal_out_in`: a label inside DO is refused as 47.2 without the traceback.
- `interp_tail`: an error message insert for a LF inside INTERPRET.

The probes covered:

- LEAVE/ITERATE from plain DOs (single and nested) to the enclosing loop, ITERATE with a name, in
  TO, FOREVER, WHILE, UNTIL, OVER, `do n`, LOOP, `while f()` (nested path) and labelled loops.
  LEAVE of a label through a plain DO. Error 28.1/28.2 for LEAVE/ITERATE in a plain DO outside a
  loop.
- SIGNAL out of plain DOs (top level, loop, subroutine), SIGNAL ON SYNTAX/NOVALUE/ERROR and CALL ON
  ERROR/NOTREADY firing inside and at the last clause of blocks, and the handler's SIGL.
- Plain DO as the THEN/ELSE/WHEN/OTHERWISE target, DO inside SELECT inside DO, and one-line
  `do; ...; end` forms.
- EXIT/RETURN inside nested blocks, including in methods and routines. A block as a body's last
  instruction.
- 200-deep and 3000-deep recursion through nested blocks.
- NUMERIC, ADDRESS and DROP/PARSE under `trace i` inside blocks.
- `trace r`/`trace i`/`trace a` over blocks, including indentation after LEAVE, ITERATE, SIGNAL and
  RETURN out of blocks.
- INTERPRET of blocks, including LEAVE out of an interpreted block (28.1 matches).
- `DO LABEL` with LEAVE/ITERATE by name.
- The witness `lang/do_block_calls.rex` (identical, and its stdout matches the report).

### 2. Park/Split

I added 10 scratch-only unit tests to the archived tree (not the repo) that run a program plain and
with scripted outcomes. They compare stdout, stderr, rc and the clause-entry count:

- Park at a DROP in a flat loop body, in `if 1 then do ... end` under `trace r`, in a callee, in a
  SELECT WHEN, in a method, and before a NOVALUE-trapping SAY.
- `trace r` and `trace i` with one and two parks.
- Three consecutive parks.
- Split twice under `trace r`.

All 10 pass (`$S/p6-t10-review/review-tests2.log`). There is no re-open and no re-echo, the parks
counter matches, and the output is unchanged. The scripted arm and the counters are `#[cfg(test)]`
only. REPLY behaviour (`reply2`, bare and valued, trailing code) is identical to the oracle.

### 3. Pinning

At 0f5aef01c, `cargo test -p rexx-exec --features pinning --test concurrency_tests measured::`:
7 pass. `pinned_parks_over_the_derived_list` failed only because the archive has no `ootest/`
checkout. That is environmental.

I re-ran the same test with `if false && ` added to the flatten condition in `run/loops.rs`
(`$S/p6-t10-review/measured-mut.log`). `a_park_inside_a_stackless_entry_is_under_no_pinned_frame`
goes red with `[[NestedLoop]]` for exactly the two new programs. `a_park_under_each_frame_kind_records_it`
passes with the `do label l` NestedLoop probe on head. The measured tests are meaningful.

### 5. refusal-sites.tsv

44 rows changed. In every one, `crates/rexx-exec/src/lib.rs:N` became `N+3` and the rest of the
row is byte-identical. I checked this by substituting N+3 into each old row and comparing. lib.rs's
first hunk inserts three lines at 89, and every changed row cites a line past it.

### 6. Comments

Apart from Minor 1, the P20 trait docs and Important 2, the new comments match the code.
`compile.rs`'s `Simple` END comment, `FlatStart::Block`, the loops.rs fallback comment,
`Deliver::Wake`, `Exit::Suspended`, the `counters.rs` block and the updated test docs were each
checked against the code.
