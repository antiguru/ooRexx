# Spec review, lens `facts`: Phase 6.1

Spec: `docs/superpowers/specs/2026-10-07-phase-6-1-loose-ends.md` (draft, uncommitted). Tree `be19fd06a`.
Build: `git worktree add /tmp/claude-1000/p61/rf/wt be19fd06a`, `CARGO_TARGET_DIR=/tmp/claude-1000/p61/rf/target memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`.
Probes: `/tmp/claude-1000/p61/rf/run.sh N FILE...` (ours once, then the scout-common oracle command N times, each from the
probe's own directory). Probe files kept under `/tmp/claude-1000/p61/rf/probes/` and `/tmp/claude-1000/p61/rf/m11/`;
worktree and target dirs removed.

## Scout claims re-run

| claim | command | result | holds |
|---|---|---|---|
| b13: native row on a wrong-type receiver crashes the oracle | `run.sh 5 probes/b4_subclass_supplier/p.rex probes/b2_run_list_items_on_inst/p.rex` | oracle rc 139 x5 each; ours rc 120 (`a message send to a value that is not a supplier ... (Phase 5)`; `a one-off method whose body ... (Phase 5)`) | yes for these two |
| b13: Array ITEMS on an instance answers a varying number | `run.sh 3 probes/b2_run_array_items_on_inst/p.rex` | oracle rc 0: `140194114805696`, `140085406834624`, `140701682212800` | yes (not a crash) |
| b5 | `run.sh 1` on `b10_setmethod_from_plain_method`, `b9_setmethod_from_directive_method`, `b10_run_constant_method`, `b6_setmethod_primitive_array_subclass` | ours rc 120 `a one-off method whose body this crate does not hold ... (Phase 5)`; oracle `um um`, `5`, `5`, `3` | yes |
| c1 | `run.sh 1` on `c_do_counter_ctrl`, `c_do_with_over_dir`, `c_do_over_stem_three`, `c_do_counter_over` | ours rc 120 `DO is not implemented`; oracle `3 4`, `A 1`, `1 ABC 2`, `2 b` | yes |
| c4 | `run.sh 1 probes/c_condition_d_raise6/p.rex` | ours rc 120; oracle `D=[] NOVALUE` | yes |
| D1 `rnd` | `run.sh 1 probes/rnd/rnd.rex` | ours `876 457 99 70`; oracle `876 457 517 735` | yes |
| D1 `sl1` | `run.sh 1 probes/sl1/sl1.rex` | ours `main endlocal 1`; oracle `0` | yes |
| D1 `sl2` | `run.sh 1 probes/sl2/sl2.rex`, 5 times | 5/5 ours `cont endlocal 0`/`main endlocal 1`; 5/5 oracle the reverse | yes |
| `sl3` oracle abort | `run.sh 5 probes/sl3/sl3.rex` | oracle rc 134 x5; ours rc 0 `main sees routine`, `after internal internal 1` | yes (but see I8) |
| receiver_class stem default (census b row) | `run.sh 1 probes/stemdef/p.rex` (`a. = 'dflt'`, `a.~length`, `a.~upper`) | both `4`, `DFLT` | agrees |
| M11 | crate test on `run_program_on_stack` (40 MiB stack), program `m11/m11.rex`; `cargo test -j 4 -p rexx-exec --lib rf_m11_probe -- --nocapture` | unmutated: `found 1425`, `final caught`, `result slow done after 1`, rc 0, 9.36 s. With `cancel_wait`'s `sleepers.retain` deleted (`scheduler.rs:1047-1049`): `result The NIL object after 0` | scout C holds; the spec misreads it (B1) |
| `Activation` has room for D1's packing | `cargo +nightly rustc -j 4 -p rexx-exec --lib -- -Zprint-type-sizes` | `activation::Activation`: 512 bytes, 37 fields summing to 512, no padding line | no (I1) |
| O2's 213 parser sites | `/bin/grep -rn 'error(\s*[0-9]\+,\s*[0-9]\+)\|ParseError::new(' --include=*.rs crates/rexx-parse/src \| /bin/grep -v tests \| wc -l` from `rust/` | 198 (169 `error(` + 29 `ParseError::new(`); 213 only without `grep -v tests` | no (I5) |
| ooTest never subclasses the b14 classes | `/bin/grep -rliE 'subclass[[:space:]]+\.?(string\|stem\|method\|routine\|message)\b' ootest/`; `/bin/grep -rliE '\.(string\|stem\|method\|routine\|message)~(subclass\|mixinclass)' ootest/` | no file | used in B2 |

## BLOCKER

**B1. Section 4: "M11 is a defect (a trapped 11.1 at a pinned `SysSleep` leaves a stale sleeper; `m~result` then answers
`.nil` early), fixed with the prototype as its witness"; section 7 T8 "M11 fixed".**
Wrong. M11 is a mutant, and the prototype killed it. The shipped `cancel_wait` already withdraws the sleeper
(`scheduler.rs:1047-1049`, `.sleepers.retain(|Reverse((_, _, sleeper))| *sleeper != running)`). Scout C section 4:
"Unmutated: ... `result slow done after 1`"; "M11 (the `sleepers.retain` in `cancel_wait` removed): ... `result The NIL
object after 0`". Re-run above reproduces both. A plan built from the spec would hunt for a defect that does not exist.
Fix: "M11 is killed by scout C's prototype; it lands as a crate test (a 40 MiB `run_program_on_stack` stack finds the
depth in 9.4 s in a debug test build, against scout C's ~25 s at 64 MiB)." T8: "M11's witness landed".

**B2. D4 / O1: b14 rehomed to Phase 9 because Phase 9's row says "the full suite green against existing baselines".**
Not a true reason. Phase 9's exit is the ooTest suite (`2026-07-27-rust-rewrite.md:661`, and the quote omits its
"*excluding* the groups enumerated below"). No ooTest file subclasses String, Stem, Method, Routine or Message (both
greps above return nothing), and scout A's b14 row has "none in C6". So Phase 9 could close green with b14 still
refusing, which is the rot 6.1 exists to remove (roadmap 6.1: "re-homed to a remaining phase with a true reason").
"The blocking refusal already names Phase 9" is no reason either: that label is `Loud::native_method`, which the census
says was relabelled wholesale and "not checked against any plan row" (`loud-census.md`, (a) caveats). Exit criterion 1
("Phase 9 ... with a true reason") then cannot be checked for b14. Fix: either IMPLEMENT b14 with subclass `NEW` in 6.1,
or amend Phase 9's row to name these refusals as its own work, quoting the amended text. Moved to open questions.

## IMPORTANT

**I1. Section 2: "`size_of::<Activation>() == 512` holds (`activation.rs:497`): the clock anchor inline as an `i64` ...,
the two flags as bits in `ActivationFlags`, and the seed, the SETLOCAL list and the condition in one cold box".**
False as written. `-Zprint-type-sizes` shows 512 bytes with no padding: the fields sum to exactly 512, and the trailing
small fields (`condition_syntax` 6, `trace_mode` 9, eight 1-byte fields) fill the last three words. An inline `i64` plus
a box pointer is 16 bytes more, so the assert fails. Scout B said it was unchecked ("Whether 8 inline bytes still fit in
512 must be checked with the assert; I did not build it"). Also: `current_case_text` (D1's list) has no home in this
packing (it is an `Option<Vec<u8>>`, 24 bytes, `activity.rs:158`); and "the condition" is ambiguous, since `Activation`
already has `condition: Option<TrappedCondition>` (64 bytes, `activation.rs:364`), distinct from
`Activity::active_condition` (`activity.rs:120`). Fix: state the packing that fits (for example `cached_clock` plus the
anchor in one 16-byte pair, or `condition` moved into the cold box with the new fields, measured), or raise the assert to
a stated size with D1's per-call cost measured against it. Put `current_case_text` on the SELECT frame or in the box.

**I2. D3: "A native row on a receiver of the wrong type crashes the oracle (SIGSEGV, 5 of 5 runs each, scout A b13).
... Recorded in `corpus/oracle-crashes.txt`."**
True for six borrowed rows and the Supplier subclass, not for the class. Scout A b13: "Array ITEMS and Queue ITEMS
answer a number that varies by run, Table ITEMS `0`, Method SCOPE empty, Class ID 91.999" (results lines 1759, 1761,
1764); re-run: Array ITEMS answers three different addresses at rc 0. Those rows reach the same `receiver_class` guards
once b5 lands (section 3), and an `oracle-crashes.txt` entry would misdescribe them. Scout A also splits b13: the
`Err(kind)` sites from `receiver_kind` and the "this crate did not build" sites are GUARD, not DEVIATION; the spec
drops that split. Fix: D3 says "crashes or answers memory-dependent garbage (scout A b13 lists which)"; record the
crashing rows in `oracle-crashes.txt` and the non-crashing ones as a deviation row; carry scout A's GUARD half of b13.

**I3. Section 5: "D6's pause decision sits behind the existing `$debugging` register on the Flowed arm".**
There is no `$debugging` test on the Flowed arm today. `ir/drive.rs:1905` is the only one (hot exit); the
`RegionEnd::Flowed(flow)` arm (`:1930`) has none. Scout B section 2: "Adding `if $debugging && ...` there is a per-clause
branch on a register-held bool, which the record prices at ~0.5% rexxcps". That alone is the whole +0.5% budget.
Fix: say D6 adds one branch on the Flowed arm, priced at ~0.5% by `oorexx-per-clause-branch-costs`, and either budget
for it or name the design that avoids it (for example deciding the pause at lowering, under the existing hot-exit test).

**I4. Exit criterion 3: "each scout B scope probe agrees with the oracle (5 runs each for the REPLY ones)".**
Not checkable for three of them. `sl3` aborts the oracle (5/5 rc 134, re-run above), so it cannot agree; `cond3` is out
of scope (D9) and already agrees on rc; `reply` prints elapsed times that differ per run on both engines (scout B:
`m after 0.031429..0.031942`), so byte agreement never holds. Fix: list the probes by name, exclude `sl3` and `cond3`,
and state the `reply` comparison (for example `m before 0` exactly and `m after` below the main's elapsed time).

**I5. O2: "D5's message inserts touch 213 parser sites (L, mechanical)".**
198 at `be19fd06a` with scout B's own `grep -v tests` (213 includes test files). And the count is of all raising sites,
while scout B says "Only sites whose message has `&N` inserts need values". The better sizing is already recorded:
`phase-4-exclusions.txt` (the 3a paragraph after `:1912`) "at least 56 `(code, sub)` pairs over at least 121 pairs and
up to ~230 call sites, and the compiler will not find them: a field defaulting to empty leaves every raiser compiling and
silently unsubstituted". Fix: cite that row and its risk; replace 213 with the derived count of `(code, sub)` pairs
whose message has an insert, with the command.

**I6. D5 / O2 against Phase 3's recorded scope.** Roadmap row 3 (`2026-07-27-rust-rewrite.md:654`): parse errors give
"number and sub-number on a plausible line, with message text and substitutions deliberately not reproduced (2026-07-28
scope decision)". D5 reverses that decision without naming it. Also, O2's reason ("a top-level parse error is a class
(c) refusal") is about the rc 120 refusal, which reporting the error without inserts already removes; the inserts are
not the refusal. And c8's existing instrument, `corpus/errors/parse-errors.tsv` driven through `rexxc`
(`phase-4-exclusions.txt`, end of the 3a paragraph), is not mentioned. Fix: D5 names the 2026-07-28 decision and
whether 6.1 reverses it; c8's witness names `parse-errors.tsv`. Moved to open questions.

**I7. Exit criterion 1 rests on "A committed command re-derives the census (scout A's `sites.py` and `classify.py`)".**
`classify.py` keys its per-site classes by `file:line` (`loud-census.md` appendix, e.g.
`'rexx-exec/src/environment.rs:527': 'a|b'`). Scout A section 1: three inserted lines in `run.rs` moved two keys and
produced `1 UNCLASSIFIED` until the keys were hand-edited. Every 6.1 stage moves lines, so the committed command breaks
on every edit. Fix: key the per-site table by constructor plus enclosing function (or a marker comment), and make the
command fail on UNCLASSIFIED.

**I8. Section 2: "`sl3` aborts the oracle ... an `oracle-crashes.txt` entry".**
Already recorded: `corpus/oracle-crashes.txt` entry 10b ("the second `ENDLOCAL` that actually restores", SIGABRT, rc 134,
`free(): invalid pointer`; "Exactly one restore per process survives"). `sl3` is 10b reached through D1's
termination restore. That matters for D1's witnesses: any program with two restores in one process (termination
restores included) aborts the oracle, so every SETLOCAL witness must stay at one restore. Fix: extend entry 10b with the
termination-restore route instead of a new entry, and state the one-restore rule for the witnesses.

**I9. Exit criterion 5 omits rows the scouts attribute to 6.1 items.** Scout A: b2 GUARD
TEST_WHEN_USE_LOCAL_NO_WAIT (derived); b7 TRACE_TraceObject `test_object_and_scope`; c1's Method whole and derived,
MethodArgs TEST_REQUEST_STRING_*, TRACE TEST_TRACE_LABEL_WITH_FORWARD and TRACE_TraceObject
TEST_CALLER_STACK_FRAME_REPLY_START (COUNTER and DO WITH both); b5's Class TEST_CLASS_DEFINE (indirect). Scout B: TIME
TEST_5 and TEST_11 fail in the same rows and are unattributed (`table.txt` lines 4-5). Fix: list them, the unattributed
ones as needing a recorded reason.

**I10. Section 4, `pct:d`: "places d preemption points uniformly over the run's clause steps".** The run's step count is
not known when the run starts. Scout C: "drawn uniformly over clause steps 1..k (k from `k=`, else from the previous
run's step count, which the run report prints)". Without k the policy is not defined, and k is part of the replay key.
Fix: carry `k` into the policy syntax and the replay line.

## MINOR

- M1. Section 3, GUARD b12: "198 probes agree". Scout A: 198 probes, "none refused; 196 agree"; the 2 differing are
  silent (section 4). Fix: "198 probes, none refused".
- M2. D9: "the silent divergences scout A met (its section 4) and scout B's `cond3` traceback, queued as their own items".
  None is queued (`/bin/grep -il 'rexxinfo\|subclass package\|CALL ON ANY\|cond3' .superpowers/sdd/queued/*.md` finds
  nothing; only `2026-10-01-routine-call-context-name` pre-exists). Fix: "to be queued", with a stage that does it.
- M3. Section 2, `random_seed` "seeded from the activity's generator on first use". The oracle seeds at activation
  creation (`RexxActivation.cpp:174`, `:312`), advancing `Activity::randomSeed` per top-level activation
  (`Activity.cpp:469-475`). Not observable, since the activity seed starts from `time()`, `clock()`, pid and thread id
  (`Activity.cpp:442-461`), but say it is a licensed laziness, not the oracle's rule.
- M4. Section 4 Report: "`rexx-run` prints one `rexx-sim:` stderr line, which the harness masks". The harness compares
  raw stderr today (scout C section 3: "`agree` compares masked stdout, raw stderr and status"); the mask is new work,
  and a child `rexx` started by a command writes its own line into the parent's stream. Fix: name both.
- M5. Section 5: "the ping-pong benchmark". `bench-programs/pingpong/` holds three (`pingguard.rex`, `pingmsg.rex`,
  `pingsem.rex`). Name which.
- M6. Section 2 witnesses: "a crate test otherwise (`reply`, `sl2`)" under "where the oracle's output is deterministic".
  `sl2` is deterministic (5/5 both engines); the reason it is a crate test is that it is concurrent. Reword.
- M7. Section 4 refusals: scout C also refuses "waiting for a post with nothing in flight that this thread will deliver
  (a native's own thread calling back)"; the spec's table lists only named semaphores and rxapi.

Checked and correct: P89's lines `ir/drive.rs:2984-2986`; `ActivationFlags` is a `u8` with three bits used
(`activation.rs:495-508`); `scheduler.rs:1990-1999` is the switch-mode arm; `RexxActivation.cpp:3455-3460`,
`:4640-4682`, `:1494-1500`; `isTopLevelCall` and `isInternalLevelCall` partition the same activations
(`ActivationSettings.hpp:67-80`; DEBUGPAUSE becomes INTERPRET, `RexxActivation.cpp:205-210`); `LanguageParser.cpp:866-876`,
`:4129-4132`; `Activity.cpp:3240-3263`; the oracle's 24 ms slice (`ActivityManager.hpp:359`); `phase-4-exclusions.txt:844`
and `:530`; `closed_phases.rs` `CLOSED` and its debt paragraph; `condition_copy` (`condition.rs:512`),
`assign_expr_target` (`run.rs:2510`), `operator_message_receiver` (`eval.rs:1184`), `request_array`
(`dispatch/array.rs:647`); `a_reply_continuation_keeps_the_elapsed_clock_and_the_random_seed`
(`scheduler/tests.rs:1058`); c7's two callers match `Refused::Raised` first (`dispatch/library.rs:471-476`, `:541-550`).

Homes: every queued item named by roadmap row 6.1 has one (`phase-5-refusal-labels` D8/T6, `do-with-over-refusal` c1,
`use-arg-message-term` c2, `elapsed-clock-per-routine-and-reset` and `setlocal-scope` D1, both debug items D6,
`interpret-syntax-traceback-line` D5, `unjudged-scheduler-mutants` section 4, `seeded-random-switch-mode` D7). Every
census (b) constructor and (c) row maps to a scout A group with a disposition, except b13's GUARD half (I2) and b14's
untrue rehome (B2). The `a|b` Phase 5 half is b18; the `c|d` site is c7.

## OPEN QUESTIONS

1. **O1 (b14), answered differently.** The spec recommends rehoming to Phase 9. Phase 9's row cannot hold it (B2).
   Recommend: take subclass `NEW` on String, Stem, Method, Routine, Message into 6.1 with b14 (scout A sizes b14 S once
   `NEW` exists; `NEW`'s own size is unestimated, so ask for it before deciding), and fix VariableReference subclass
   `NEW` (oracle 93.967, scout A section 4) with it. If not, amend Phase 9's row text to name them.
2. **O2 (inserts), answered differently.** It reverses the 2026-07-28 Phase 3 scope decision (I6), and the
   exclusions row warns the compiler cannot find missed sites (I5). Recommend: 6.1 lands the traceback line and
   attribution for all three paths (removing the refusals), and the inserts only if Moritz reverses the Phase 3
   decision explicitly; then with a type-level guard (a raising API that cannot be called without the insert values
   for a message that has inserts) rather than a defaulted field. Exit criterion 4's `i2` probes need inserts, so it
   follows the answer.
3. **b13 non-crashing rows (I2).** Whether a deterministic oracle answer reached by an undefined read (Table ITEMS `0`,
   Class ID 91.999, Method SCOPE empty) is a deviation or something to match. Recommend deviation: the oracle is
   reading a wrong-type object and the value is an accident of layout.
4. **`Activation` size (I1).** Whether 512 is a hard line. Recommend keeping it and moving `condition` into the cold box
   with the new fields, measured on fibcall, fibfunc, dispatch and sendloop, since a CONDITION() handler is the rare path.
