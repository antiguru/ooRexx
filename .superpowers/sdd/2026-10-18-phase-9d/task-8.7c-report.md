# Phase 9d Task 8.7c report: variables and streams residuals

Status: DONE_WITH_CONCERNS. Base `97eee8a9a` (dispatch commit `34a25ee6b` on top). Code `a33114804`,
`e335a9162` and `e620097f0` (the perf round); records, closings and inventory rows in the commit after,
with this report; the workspace result in the commit after that. The closing lines and inventory rows
name `e620097f0`, the last code commit. Records: `docs/superpowers/records/2026-10-18-phase-9d/t8.7c/`
(`three.sh` the runner: `o` oracle, `b` base `rexx-base-97eee8a9a`, `h` `rexx-head-e620097f0`, each
probe directory's outputs under `out/`; `witnesses/out/` the three-way runs of the new corpus programs
from their scratch copies; `suite/`, `checks-*.txt`, `callgrind.txt`).

## What changed, per item

**2026-10-27-varref-operator-traceback-line.md (closed).** Rule: an operator is a message, and a
VariableReference forwards a message it does not know to the referenced value, so the failure is
inside that value's operator method. `is_forwarding_receiver` (`eval.rs`, was `is_stem_receiver`)
answers for a reference whose chain ends at a string or number value or an unset variable, so
`blame_forwarded_operator` adds the `Compiled method "<op>" with scope "String"` line on every
arithmetic, prefix, logical, comparison and loop-header failure. Probes `probes/varref/v1`, `v3`, `v4`
(o~b differ, o~h equal). Witness `corpus/lang/variable_reference_operator_traceback.rex`.

**2026-10-27-stem-array-default-string-value.md (closed).** Rule: rexxref intro.xml:2653-2656, "the
STRING method will return the Stem object's default value's string representation", and TRACE renders
a non-string by its STRING (instrc.xml:3697-3699). `string_value_text` (`value.rs`) renders a stem as
its default's `string_value_text` (`stem_string_value_text`, the chain of defaults and references);
`Object~string` of a Stem subclass instance takes the same. `t.~string`, `u~string` and `trace r; x =
t.` answer `an Array` (`probes/stemarr/a1.rex`). `say t.` still joins the items here: Deviation
candidate 1. Witness `corpus/lang/stem_default_string_rendering.rex`.

**2026-10-22-lineout-four-args-error.md (closed).** Rule: rexxref does not order the two errors; the
oracle's `NativeActivation::processArguments` converts each declared argument before it counts the
rest (`NativeActivation.cpp:682`). `stream_charin`, `stream_charout`, `stream_linein` and
`stream_lineout` are `Arity::Counted` and call `declared_only` after their conversions
(`dispatch/stream.rs`, `dispatch/native.rs`). Probe `probes/lineout4/l2.rex`. Witness
`corpus/lang/stream_method_extra_arguments.rex`.

**2026-10-28-internal-rendering-defaultname.md (closed).** Rule: rexxref fundclasses.xml, Object
`string`/`objectName`/`defaultName`: the string representation is the objectName, which is the
defaultName where none is set. The oracle's `RexxObject::stringValue` sends `OBJECTNAME`
(`ObjectClass.cpp:1157`), whose primitive sends `DEFAULTNAME` (`:1713`). `string_value_text` now sends
`OBJECTNAME` to an instance or class whose `OBJECTNAME` is a method of its own, else `DEFAULTNAME` under
the primitive `OBJECTNAME`'s level where that is one of its own and no name is set
(`sent_string_value`); `default_string_value_text` is the unsent rendering the primitives
`OBJECTNAME`, `DEFAULTNAME` and `STRING` answer with. `Object~string` and `~objectName` push their
levels where the send reaches Rexx code, and `Array~makeString`/`~toString` run as leveled primitives
(`RENDERING_METHODS`, `dispatch.rs`). `string_value_text` reaches the sends through an opaque function
pointer (Perf). Probes `probes/dn/dn4`-`dn7` (dn7 differs in the count of sends
under trace: Concerns 2). Witness `corpus/lang/internal_rendering_default_name.rex`.

**2026-10-28-trace-object-renderings.md (closed).** Rule: TRACE renders a non-string result by its
string representation (instrc.xml:3697-3699), and `VariableReferenceOp::evaluate` traces the
reference's `getName()` (`VariableReferenceOp.cpp:116`). PARSE VALUE's `>K>` renders the value by
`string_value_text` (`parse_template.rs`); `>O> ">"` names the reference's variable (`eval.rs`).
Probes `probes/traceobj/trace-parse-value-a14.rex`, `s25.rex`, `r1.rex`. Witness
`corpus/lang/trace_reference_and_parse_value.rex`.

**2026-10-28-primitives-without-frame.md (closed).** The set, from the oracle: the collection
primitives that compare or hash an argument through `equalValue`, `isEqual` or `hash`
(`ArrayClass.cpp:2107`, `ListContents.cpp:758`/`:793`, `HashContents.hpp:430`/`:437`,
`StemClass.cpp:1020`, `DirectoryClass.cpp:417`), each of which runs under its `InternalActivationFrame`.
A new `NativeBody::Compared` (`dispatch.rs`, rows in `COMPARED_METHODS`, converted by method after the
table is built) runs such a primitive on the ordinary native path, as its own level where an argument
is not of a base class (`may_run_rexx`); `Table~at`/`~[]`, whose row `Directory` shares, level inside
`hash::hash_at` (`[]` is now its own function so the level has its name); the rows already leveled by
Task 8.7b (Array `HASITEM`, the hash `PUT`/`[]=`) are unchanged. Probe `probes/primframe/f4.rex`:
every frame now agrees; the rows that still differ are which messages the crate sends (Queue candidate
1). Witness `corpus/lang/collection_primitive_compare_frames.rex` (the agreeing rows).

**2026-10-29-notready-delay-across-internal-call.md (closed).** Rule: rexxref condtra.xml, Notes: the
state (ON, OFF or DELAY) of each trap is saved on entry to a subroutine, so an internal routine
inherits its caller's DELAY; Deviation 75 ignores a NOTREADY raised while delayed.
`notready_trap_delayed_at` (`run/condition.rs`) counts a condition queued at any activation the
internal-call chain from the trapping one reaches. n4 and n7 run the handler once (`after a, m = 1`)
at head and twice at base (`probes/notready/out/*.b.out`, `*.h.out`; never run on the oracle).
Witness: two new cases in `tests/delayed_trap_cases/crate`.

**2026-10-28-stream-query-residuals.md (not closed: part 3 is Deviation candidate 5).**
* Part 1 (fixed). Rule: a console redirected from a regular file is a persistent stream open for
  reading (rexxref funct.xml, STREAM QUERY SIZE, POSITION, DATETIME, SEEK). `.STDIN` and an opened
  program-made STDIN answer from the file standard input reads (`Interp::input_file`, `input.rs`;
  `standard_input_file`, `seek_standard_input`, `query_size`, `query_time`, `query_position`,
  `dispatch/stream.rs`; `position`'s move is now `seek_open`, shared). Probes
  `probes/stdinq/q2.rex`, `q6.rex` (stdin `in.txt`): o~h equal. `q5.rex` still differs on `.STDOUT`'s
  queries (Queue candidate 3) and on a write-position or zero SEEK, which Deviation 59 refuses. Witness
  `input_oracle::a_file_console_answers_queries_and_seeks` (gated; red at base, `q6.rex.b.out`).
* Part 2 (ruled, unchanged). LINES and CHARS on a stream opened WRITE answer 0 under Deviation 55
  (`probes/lineswo/w1.rex`).
* Part 3: Deviation candidate 5.

**2026-10-28-append-stream-positioned-writes.md (not closed: Deviation candidate 3).**

**2026-10-28-hash-lookup-identity-first.md (not closed: Deviation candidate 2).**

**2026-10-27-stem-default-arithmetic-nostring.md (not closed: Deviation candidate 1).**

**2026-10-29-debug-input-traps-off.md (not closed: Deviation candidate 4).**

**gate:phase-9c-gate.md:139 (closed: probe agrees).** `2026-10-16-stream-command-open-missing-file.md`
was closed by Task 6.4; `probes/scopen/scopen.rex` agrees at base and head (`ERROR:2`, `rc ok`).

**gate:phase-9a-gate.md:485 (closed: probe agrees).** `samples` `TEST_MAKESTRING`, `scclient`
`TEST_04_SCCLIENT.REX` and `sfclient` `TEST_04_SFCLIENT.REX` pass on both sides in `suite/tests.tsv`
(and in `t8.7b/fix-r3/suite/tests.tsv`).

## Deviation candidates (not written)

1. **A stem in a required-string position.** Rule: a string value is `request('STRING')` (rexxref
   provide.xml:758), and a Stem's `request` forwards every class but ARRAY to its default
   (collclasses.xml, Stem `request`), so a stem's string value is its default's, and NOSTRING is
   raised only where that has none. Here `s. = 7; say s. ''` is `7`, and `t.` with an array default
   joins the items. ORACLE: `StemClass` has no `primitiveMakeString`, so `requestString` on any stem
   takes `stringValue` and raises NOSTRING naming the stem (`ObjectClass.cpp:1240`-`:1247`): under
   `SIGNAL ON NOSTRING`, `say s. ''`, `length(s.)`, `'<'||s.` and `'x' = s.` with `s. = 7` all trap
   `NOSTRING |7| Stem`, and `say t.` prints `an Array`; in an arithmetic operand and a comparison's
   right operand with a default that has no string value it forwards and names the default (`1 + v.`
   `|NSTR| NSTR`), where this crate names the stem (`|a NSTR| Stem`) as Deviation 72 does for a
   reference to a stem (`probes/stemnostr/s1.rex`, `s2.rex`). This crate is not uniform either: `say
   v. ''` (the stem as a blank concatenation's left operand) names the default. Candidate: a stem
   names itself in every position (Deviation 72's stem sentence extended to a plain stem, and the
   concatenation receiver made to agree), or names its default in every position.
2. **A hash collection's lookup of the identical index.** Rule: "Index equality is determined by
   using the `==` method of index" (collclasses.xml, Table `at`, `hasIndex`, `put`, `remove` and the
   rest; items likewise), which this crate follows: `t[k] = 2` after `t[k] = 1` sends `k~'=='(k)`.
   ORACLE: `equalValue` compares identity first (`ObjectClass.hpp:369`, from `HashContents.hpp:430`,
   `ListContents.cpp:758`, `ArrayClass.cpp:2107`), so the index's own `==` is never sent for itself
   (`probes/hashid/hash-identity-first-idq.rex`: `== called` twice here, never there; the `other
   object` row agrees). Candidate: compare identity first (`==` treated as reflexive, which also takes
   the send off a lookup's hit), or keep the rule.
3. **Positioned writes on an APPEND stream.** Rule: APPEND "positions the write pointer at the end of
   the stream ... The write pointer cannot be moved anywhere within the extent of the file as it
   existed when the file was opened" (funct.xml:4307-4312), so a write SEEK, a CHAROUT start or a
   LINEOUT line within that extent is refused (0 and NOTREADY, as Deviation 59's invalid SEEK; the
   residual count for CHAROUT). Neither interpreter enforces it: `seek('=1 write char')` answers `1` on
   both. The oracle then keeps the logical position (`pos3 4`) while its writes land at the end;
   this crate reports the end (`pos3 17`); `lineout('L4', 1)` is NOTREADY here under Deviation 55 (an
   APPEND open is write-only) and writes on the oracle (`probes/append/a1.rex`). Candidate: enforce
   the extent rule for SEEK and the CHAROUT start, or license both answers.
4. **A condition raised while debug input is read.** Rule: "While user input is processed during
   interactive tracing, all condition traps are temporarily set OFF" (condtra.xml:489-494, Note 4),
   so every condition the `.DEBUGINPUT` read raises takes its default action. This crate drops one
   only where its trap is delayed (Deviation 75) and delivers the rest: `dp3`'s `h 2 from input`, dp2's
   first read in main (`t8.7b/fix-r3/dp2`, `dp3`). ORACLE: `debugPause` is set only around
   `debugInterpret` (`RexxActivation.cpp:2789`), so the read's conditions reach their traps; not run
   (the delayed shape is `oracle-crashes.txt` entries 64-67's). Candidate: extend Deviation 75's
   sentence to every condition the read raises (the paused activation's debug-pause flag set across
   the read, so `trap_for` answers none but SYNTAX; the debug case's expected block changes to no
   handler run), or keep the delayed-only drop.
5. **A SEEK of an unopened transient stream.** Rule: positioning a transient stream is invalid,
   93.958. ORACLE: `stream_position` tests `transient` before its implicit open sets it
   (`StreamNative.cpp:2694`), so the first SEEK of `.stream~new('STDIN')` with standard input
   `/dev/null` answers `1` and `READY`; after `open('read')` it is 93.958 on both
   (`probes/stdinq/q4.rex`). This crate raises 93.958 either way. Candidate: license the difference.
6. **TRACE and Stem STRING renderings of an object overriding STRING.** Rule: TRACE sends STRING
   (instrc.xml:3697-3699) and a Stem's STRING is its default's string representation (intro.xml:2656),
   so a default or traced value whose class overrides `STRING` renders as that answer. Both
   interpreters render by `stringValue`, the `OBJECTNAME`: `v. = .nstr~new; say v.~string` and `trace
   i; x = v.` show `a NSTR` on both where the rule gives `NSTR` (`probes/stemnostr/s2.rex`,
   `probes/stemarr/b.rex`). Unchanged here. Candidate: follow the rule, or record the
   shared answer.

## New oracle defects (for defects.json; not edited)

1. Any stem in a required-string position raises NOSTRING naming the stem, whatever its default:
   `s. = 7; signal on nostring; say s. ''` traps `NOSTRING` with description `7`
   (`probes/stemnostr/s2.rex`; candidate 1).
2. The first SEEK of an unopened transient stream answers `1 READY` (`probes/stdinq/q4.rex`;
   candidate 5).

## Queue candidates

1. **Which messages a hash collection sends.** `Table~remove`, `Set~remove` and `Set~removeItem` send
   the index's `HASHCODE` twice where the oracle sends it once; `Bag~hasItem` and `Relation~allAt`,
   `~hasItem`, `~items` and `~removeItem` scan with `==` where the oracle hashes the index and finds
   an empty bucket (`probes/primframe/f4.rex`, the rows outside the witness).
2. **TRACE renders an assignment's value once for its `>>>` and `>=>` lines**, where the oracle
   renders each line, so a `DEFAULTNAME` of the object's own runs twice here and three times there
   for `x = o` under `trace i` (`probes/dn/dn7.rex`).
3. **`.STDOUT`'s QUERY SIZE and POSITION when standard output is a regular file** answer the null
   string here and the file's size and position on the oracle (`probes/stdinq/q5.rex`, `out size 34
   1`).

## Witnesses

New `corpus/lang` programs, in `phase-9.txt`, each with its `sourceline_oracle` file generated by the
test's driver from a scratch copy: `variable_reference_operator_traceback`,
`stem_default_string_rendering`, `stream_method_extra_arguments`, `trace_reference_and_parse_value`,
`internal_rendering_default_name`, `collection_primitive_compare_frames`. Each differs from the oracle
at base and agrees at head on stdout, stderr and rc (`witnesses/out/`, run from the scratch copies;
at head also by the corpus gate). I read each oracle stdout for the paths it claims: each failing
operator's `Compiled method` line and the agreeing `say s + 0` and `t = 1` rows; `an Array` for the
stem, the subclass and the stem-default stem and `7`; each 88.907 naming argument 1 or 2 and each
88.922; the `>K>` `an Array` line and each `>O> ">" => "V"`; `DNC`, `DNI`, `DN` and `ON` with their
`OBJECTNAME`/`STRING` frames and the named instance's `named`; each primitive's `METHOD` frame. Also
`input_oracle::a_file_console_answers_queries_and_seeks` and two `delayed_trap_cases` cases (above).

## Perf

`callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 6 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7c/final2
base=.../rexx-base-97eee8a9a head=.../rexx-head-e620097f0`, every program, 3 rounds, exit 0, no OVER or
SPREAD flag; every `.vg` names one of the two binaries (99 each). Libc subtracted, every program is at
most +0.0404% (pingmsg); pingsem +0.0351%, startup +0.0199%, hashnew +0.0198%; the largest falls are
heapshape -0.0436% and dirread -0.0223%. Whole process at most +0.0323% (dirread). The base was built
from a `git archive` of `97eee8a9a` (`rust/`, `interpreter/`, `extensions/`) in its own target dir.

The full run at `a33114804` was stopped after its first round (`callgrind-a33114804-r1.txt`, from its
`.row` files): dirread +4.50%, parse +1.39%, rexxcps +0.33%, fibcall +0.27%, startup +0.24%. The causes,
each found by `cgdiff.py` and per-line callgrind costs on one-round reruns (`cg-9d/t8.7c/try2`-`try20`):
the `Compared` rows dispatched through `run_other` with `is_base_class` on every argument (dirread);
the row conversion's linear search over every native row at build (startup, +140k instructions); a
direct call from `string_value_text` to the sends, which moved register allocation in callers that
render only on a failure (`header_number`, `exec_parse`, `resume_call`; reverting `value.rs` alone
removed it, and calling through `std::hint::black_box` keeps it removed); a third caller of
`hash::store_at`, which stopped it being inlined (dirread); and a changed `rendered_into_parse_buffer`
signature (parse, fibcall).

## Commands

All from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t87c-target`, at
`e620097f0`; outputs in `t8.7c/`.

| command | result |
|---|---|
| `cargo fmt --all --check` | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test method_bodies --test coverage --test signals` (`checks-touched.txt`) | 0; lib 1207 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --test input_oracle --test stream_handle --test stream_model --test standard_stream_reopen --test stdin_contention --test collection_arity --test trace_oracle --test trace_indent --test licensed_divergences --test native_entries --test dispatch_seam` (`checks-streams.txt`) | 0; input_oracle 27 passed, none skipped |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101: the one mismatch the known `rexxutil_file_date_time.rex`. The run at `a33114804` also had `command_redirected_target_waits_elsewhere.rex` (`guard 0 3 waited 0` against the oracle's `waited 1`), the known redirected-target timing flake |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-parse --test sourceline_oracle --test errors --test tiling --test variants` (`checks-parse.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`) | 0, 74 passed |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.7c/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`checks-suite.txt`) | 0; `tests.tsv`, `table.tsv`, `verdicts.tsv`, `oracle-differences.tsv` identical to `t8.7b/fix-r3/suite/` (also at `a33114804`) |
| `memcap 16G cargo test --release -j 4 --no-fail-fast --workspace`, default target dir, `rust/target/release/rexx` built first, at `6d16e0037` (`close-workspace-build.txt`, `close-workspace-tests.txt`) | 101: two failures, both known flakes. `collect_stress`'s `the_l0_subset_passes_again_under_collect_on_every_allocation` on `command_redirected_target_waits_elsewhere.rex` (`guard 0 3 waited 0` plain, `waited 1` under stress), the redirected-target row; `datetime_zone`'s `the_file_date_time_witness_reads_the_zone_the_oracle_does`, the DST flake. `collect_stress` alone at `e620097f0` was 74 of 74 (`checks-collect-stress.txt`) |

## Concerns

1. **A failing `OBJECTNAME` or `DEFAULTNAME` send inside a rendering is dropped**: `string_value_text`
   returns bytes, so a method of the object's own that raises renders the default name here, where
   the oracle raises from the trace line, message or join. No probe reaches it.
2. Queue candidate 2 is visible only through a side-effecting `DEFAULTNAME` under trace.
3. Four items stop at Deviation candidates and stay open; part 1 of the stream-query item is fixed but
   the item stays open for candidate 5.

## Fix round 1

Status: DONE_WITH_CONCERNS. Code `1462fd46f`, `6da312e62`, `246ab583d` (the perf round) and `3c8978185`
(a comment); records, closings and this section in the commit after. Records:
`docs/superpowers/records/2026-10-18-phase-9d/t8.7c/fix-r1/` (`probes/r3.sh` the runner: `o` oracle, `b`
`rexx-head-e620097f0` (this round's base), `h` `rexx-head-246ab583d`; each probe directory's outputs under
`out/`, the oracle and head stderr of the recursion probe `i1/r4.rex` kept as its last three lines).

### Findings

**I1 (fixed).** `string_value_text` answers `Result`: a condition `DEFAULTNAME` or `OBJECTNAME` raises
while an Array join, a trace line or an error insert renders an object is the rendering's, and a
leveled `DEFAULTNAME` adds the primitive `OBJECTNAME`'s `Compiled method` traceback line
(`value.rs`; the callers across `rexx-exec` propagate it, a constructor returning `Failure` returns it,
the arithmetic inserts through `rendered_failure`). The trace emitters (`eval.rs`, `trace.rs`,
`run.rs`, `loops.rs`, `redirect.rs`, `parse_template.rs`, `run/condition.rs`) and the region loop's trace
ops (`ir/drive.rs`, a flag and a held failure: Perf) propagate it. The array join behind `to_text`, a
reference's `to_text` and the native API's renderings cannot raise; they render with nothing sent
(`unsent_string_value_text`), and `Host::string_value_text` holds a raised condition on the native
frame. Probes `probes/i1/e1`, `e1n`, `e3`, `e4`, `e4n`, `e4c`: o~b differ, o~h equal; `r4` rc 245 on
the oracle and head (the depth at which the stack runs out differs), rc 0 at base. Witnesses
`corpus/lang/internal_rendering_raises.rex` (o~b differ, o~h equal, `probes/w1`) and
`tests::a_join_rendering_itself_through_defaultname_runs_out_of_stack`.

**I2 (fixed).** `notready_trap_delayed_at` stops the internal-call chain at a frame whose `NOTREADY` (or
`ANY`) trap is not the one its caller holds (a new `CALL ON` or `SIGNAL ON` makes a new label,
`same_notready_trap`). `probes/nr/n1.rex` (crate only): head prints `hn2 NOTREADY no_such_file_zz2.txt`
first, as the base before Task 8.7c did; `b` does not. Witness: the `delayed_trap_cases/crate` case "an
internal routine's own CALL ON NOTREADY ...", which re-arms the caller's own label and so tests the trap's
identity rather than its name.

**M1 (fixed).** The answer of a sent `DEFAULTNAME` or `OBJECTNAME` renders as the primitive `OBJECTNAME`
renders it (`default_string_value_text`): `an Array` in a trace line and an error insert
(`probes/i1/e6.rex`; the witness's `answered` case). `e6` still differs from the oracle in its first two
lines: the oracle's join writes raw bytes (the reviewer's oracle defect) and its `.on~new~string` answers
the array an `OBJECTNAME` returned, as `e620097f0` did.

**M2 (fixed).** `leveled_store_at` calls `store_at` directly, with no comment.

**M3 (fixed).** `unopened_standard` and `standard_input_file` each carry their own doc block.

### Rulings

**R1 (implemented).** Rule: a stem's string value is its default's `request('STRING')` (rexxref
provide.xml, "Required String Values"; collclasses.xml, Stem `request`); where that answers none,
NOSTRING names the stem, with the stem's `STRING` answer (R6) as its description (condtra.xml, NOSTRING).
The concatenation receiver (`v. ''`, `v.''`, `v. || 'x'`) now joins the stem's string value
(`concat_reference`, `eval.rs`), and the NOSTRING description of a stem is its `STRING`
(`dispatch/reqstr.rs`). Oracle: NOSTRING for every stem in every required-string position (`|7| Stem`),
and in an arithmetic operand and a comparison's right operand the default named (`|NSTR| NSTR`)
(`probes/r1/s2.rex`, `c1.rex`, `s1.rex`; `a1.rex` `say t.` joins here, the oracle prints `an Array`).
Witness `tests::a_stems_string_value_is_its_defaults_and_nostring_names_the_stem` (`s2`'s shapes; the
`v.` rows red at `e620097f0`, which named the default for the concatenation receiver).

**R2 (pinned).** Rule: "Index equality is determined by using the `==` method of index" (collclasses.xml,
Table `at` and `put`). Oracle: identity first, the index's own `==` never sent
(`t8.7c/probes/hashid/hash-identity-first-idq.rex`). Witness
`tests::a_hash_lookup_sends_the_identical_indexs_equality` (green at `e620097f0` too: a pin).

**R3 (implemented).** Rule: on an APPEND stream the write pointer cannot move into the file as it was opened
(funct.xml:4307-4312): a write SEEK there answers 0 and is NOTREADY (as Deviation 59's invalid SEEK), a
CHAROUT start answers its residual count and a LINEOUT line 1, both NOTREADY, writing and moving nothing.
The extent is the file's size at an APPEND open (`StreamState::append_extent`, a `u32` saturating at
`u32::MAX`: Perf). Oracle: `seek =1 write char` answers 1 and the write position moves there while the
writes land at the end, and `lineout(f,'L4',1)` writes (`probes/ap/a1.rex`: `seek 0`, `w3 1`, `w4 1` here).
The stream model (`corpus/model/stream.rex`, `within_extent`) follows; the named case `append-extent` is
the witness, and `append-writes-at-end` and `append-record` now position past the extent.
`stream_model`'s `oracle_rule` judges the oracle's departures under a new rule `append-extent`
(Deviation 0 there, for the controller to number); `departures-oracle.tsv` refreshed: the scenarios
whose first departure is now an extent refusal changed label.

**R4 (implemented).** Rule: "While user input is processed during interactive tracing, all condition traps
are temporarily set OFF" (condtra.xml:489-494), and "a syntax error during interactive tracing does not
cause the exit from the program but is trapped specially" (the same note), so the paused activation's
traps but SYNTAX's are OFF across the `.DEBUGINPUT` read as while the typed line runs
(`debug_input_line`, `run/interpret.rs`; `drop_queued_for_delayed` removed). Oracle: `debugPause` is set
only around `debugInterpret`, so the read's conditions reach their traps, the end-of-input NOTREADY
included. Witnesses: the pinned debug case of `delayed_trap_cases/crate` (no handler runs now) and a new
case "a SYNTAX condition the debug pause's input read raises is still trapped". Consequence: eight
`corpus/lang` programs that matched the oracle depart (below).

**R5 (pinned).** Rule: positioning a transient stream is 93.958 (iostrms.xml:194-197). Oracle: the first
SEEK of an unopened `.stream~new('STDIN')` with standard input `/dev/null` answers `1 READY`
(`t8.7c/probes/stdinq/q4.rex`). Witness
`input_oracle::the_first_seek_of_an_unopened_transient_stream_is_refused` (a pin).

**R6 (implemented).** Rule: "Results other than strings appear in the string representation obtained by
sending them a STRING message" (instrc.xml:3697-3699), and a Stem's `STRING` is its default's string
representation (intro.xml:2653-2656): `string_representation` (`value.rs`) sends `STRING` to an object
whose `STRING` is not `Object`'s, and renders a stem as its chain end's; every TRACE value line and
`Stem~string` use it. Oracle: both render by the object's name (`v.~string` and `>V> V.` `a NSTR`,
`>V> Q` `a SN`; `probes/r1/b.rex`, `probes/r6/dn7.rex`). Witness
`tests::a_traced_value_and_a_stems_string_send_string_to_an_override` (red at `e620097f0`).
Consequence: ooTest `base/keyword/TRACE` `TEST_TRACE_FORWARD` (below).

### What else changed

* **Nine corpus programs leave the differential.** R4: `debug_input_eof.rex`,
  `debug_input_eof_call_on.rex`, `debug_input_eof_call_on_loop.rex`, `debug_input_stdin_eof_call_on.rex`,
  `debug_input_eof_iterate.rex`, `debug_input_eof_leave.rex`, `debug_input_eof_iterate_outer.rex`,
  `debug_handler_exit_at_iterate.rex`, each witnessing the end-of-input NOTREADY of a debug read reaching
  its trap. R3: `stream_append_writes_at_end.rex`. Each is in `corpus/unfiled.txt` with its ruling, out
  of `phase-6-1.txt` and `phase-9.txt` (their comments corrected), and this crate's answers are pinned
  by `input_oracle::the_unfiled_ruled_programs_answer_as_this_crate_rules` (`tests/input_oracle/ruled`,
  each run from a copy with its `.stdin` sidecar); `corpus.rs`'s sidecar check accepts a program that file
  runs.
* `ir/drive.rs`'s header-pause delivery (`Op::Clause`'s `Do`/`Loop` arm after a pause) lost its only
  witnesses with those programs: disabling it left the lib tests, the corpus gate and `input_oracle`
  green. Its comment no longer says the pause's read queues the condition.

### Perf

`fix-r1/callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 6 -o .../cg-9d/t8.7c/fix1-final2
base=.../rexx-base-97eee8a9a head=.../rexx-head-246ab583d`, every program, 3 rounds; every `.vg` names
one of the two binaries (99 each). Flags: SPREAD on pingguard and pingsem (0.015-0.017%), reported; the
run's exit status was not recorded, and a SPREAD flag makes the script exit 1 (its usage text). Libc
subtracted against base, and this round's own delta (minus `e620097f0`'s, `t8.7c/callgrind.txt`):

| program | head -libc d% | own |
|---|---|---|
| pingguard | +0.0705 | **+0.0695** |
| parse | +0.0525 | **+0.0518** |
| pingsem | +0.0351 | +0.0000 |
| arith | +0.0316 | +0.0315 |
| startup | +0.0202 | +0.0003 |
| decloop | +0.0171 | +0.0166 |
| dispatchclass | -0.1304 | -0.1304 |
| fibcall | -0.2295 | -0.2296 |

The rest lie between -0.2296 (fibcall) and +0.0315 (arith) own, in `fix-r1/callgrind-own.tsv` (the two
tables' `-libc d%` columns joined by program); whole process at most +0.0617% (pingguard).
**Two programs are over the +0.05% budget, parse by 0.0018 and pingguard by 0.0195.** The causes, each found
by `cgdiff.py` against one-round reruns (`fix-r1/callgrind-variants/`):
* The region loop's trace ops answering `Result`: `nop` +6.5%, `assign` +4.3% at `1462fd46f`
  (`callgrind-1462fd46f.txt`); all four ops infallible restores them (`fix1-exp-a`), and each alone moves
  `nop` between -0.01% and +2.1% (`fix1-exp-c`). They now answer a flag and hold the failure on the
  interpreter (`fix1-exp-e`).
* An `i64` in `OpenFile`: `intcall` +5.8% whole process, all in libc's allocator (one malloc per pass off
  the fast path); without it +0.04% (`fix1-exp-f`). The extent is a `u32` in `StreamState`'s padding
  (`fix1-exp-g`).
* `result_text`/`intermediate_text` through `then(..).transpose()`: `compound` +0.17%, `alloc4c` +0.10%;
  an early `return Ok(None)` restores them (`fix1-exp-h`).
* The held failure on `Activity`: `pingguard` +0.30%, `pingsem` +0.26% (`callgrind-6da312e62.txt`); on
  `Interp`, `fix1-exp-i`.
* `nomethod` and the arithmetic inserts inline: `dirread` +0.05%; cold and out of line, -0.03%
  (`fix1-exp-i`).
* What is left is `eval`'s trace hook: parse's `exec_parse` and `eval_node`, pingguard's `exec_guard`.
  Other shapes of the hook were measured (`fix1-exp-j`, `-k`, `-l`, `-mop`, `-t`) against the committed one
  (`fix1-exp-i`, parse +0.0525 and pingguard +0.0703 against base): a cold `traced_value` tail gives
  pingguard +0.0018 and parse +0.3633; `eval` not inlined gives parse +0.7906 and pingguard +0.9734. None
  was within budget on both.
* `value.rs`'s `black_box` stays load-bearing: without it `decloop` +0.1502 against +0.0171 and `fibcall`
  -0.0625 against -0.2296 (`fix1-ctl-nobb`); its comment states that.

### Closings

`**CLOSED 2026-10-29 by Phase 9d Task 8.7c fix round 1 (6da312e62) ...**` heads
`2026-10-27-stem-default-arithmetic-nostring.md` (R1), `2026-10-28-hash-lookup-identity-first.md` (under
the R2 Deviation), `2026-10-28-append-stream-positioned-writes.md` (R3), `2026-10-29-debug-input-traps-off.md`
(R4) and `2026-10-28-stream-query-residuals.md` (part 1 by `e620097f0`, part 2 Deviation 55, part 3 R5);
their inventory rows read `closed:6da312e62` in columns 4 and 6. R6 has no queued file.

### Commands

All from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t87c-fix1-target` but the
workspace run, at `3c8978185`; outputs in `fix-r1/`.

| command | result |
|---|---|
| `cargo fmt --all --check` (`checks-fmt.txt`) | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec -p rexx-core --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-core -p rexx-exec --lib` and `--test` owners, closed_phases, phase_ten_reasons, representation_refusals, refusal_sites, refusal_dispositions, method_bodies, coverage, signals, input_oracle, stream_handle, stream_model, standard_stream_reopen, stdin_contention, collection_arity, trace_oracle, trace_indent, licensed_divergences, native_entries, dispatch_seam (`checks-touched.txt`) | 0; lib 1211 passed. The same command at `6da312e62`, run beside a callgrind run, failed `sync::tests::a_timed_wait_ends_close_to_its_timeout` (2.8 ms late) and `scheduler::tests::native::quick_native_calls_at_the_smallest_bound_run_alike` (a bound exceeded), both timing |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101: 1 of 1224 disagree, the known `rexxutil_file_date_time.rex` |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-parse --test sourceline_oracle --test errors --test tiling --test variants` (`checks-parse.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`, `-r2.txt`) | 101 then 0: the first run failed `command_redirected_target_waits_elsewhere.rex` (`guard 0 3 waited 0` plain, `waited 1` under stress), the known redirected-target flake; the rerun 74 of 74 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../fix-r1/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`checks-suite.txt`) | 101 (expected while the phase is open); against `t8.7c/suite/` one row changed: `base/keyword/TRACE` `TEST_TRACE_FORWARD` `passed passed` to `passed failed` (R6, Concerns 1) |
| `memcap 16G cargo build --release -j 4 --bin rexx` then `memcap 16G cargo test --release -j 4 --no-fail-fast --workspace`, default target dir (`close-workspace-build.txt`, `close-workspace-tests.txt`, `close-workspace-status.txt`) | build 0; tests 101: one failure, `datetime_zone`'s `the_file_date_time_witness_reads_the_zone_the_oracle_does`, the known DST flake |

### Concerns

1. **R6 turns ooTest `base/keyword/TRACE` `TEST_TRACE_FORWARD` red.** ooTest's `TestCase` defines
   `::method string` (`ootest/framework/OOREXXUNIT.CLS:1664`), so `>V>   SELF` and `>K>   "TO"` show its
   answer where the test's resource and the oracle show `a TRACE.TESTGROUP`. Licence it, narrow R6 to the
   Stem half, or revert: the controller's call (sent 2026-10-29).
2. **R4 and R3 take nine oracle-matching corpus programs off the differential** (What else changed). Their
   crate answers are pinned; `ir/drive.rs`'s header-pause delivery is now unwitnessed.
3. **parse (+0.0518) and pingguard (+0.0695) are over the +0.05% own budget**, from `eval`'s trace hook
   (Perf).
4. `append_extent` saturates at `u32::MAX`: on a file larger than that opened APPEND, a write position past
   `u32::MAX` within the extent is accepted.
5. The renderings that cannot raise (`to_text` of an array or a reference, the native API's condition
   reports) send nothing; at `e620097f0` they sent a `DEFAULTNAME` of the object's own and dropped its
   failure. No probe here reaches them through a raising `DEFAULTNAME`.
6. `native_found` (`dispatch/library.rs`) still drops a failing `OBJECTNAME` send and renders the default
   name, citing message substitution's catch; `probes/i1/e4n.rex` shows that catch does not hold on the
   oracle for an unhandled condition (it reports the 98.900). It now renders without sending a second time.

### Commits

`1462fd46f` (findings, rulings, witnesses), `6da312e62` (trace ops, trace text gates, the extent in
`StreamState`), `246ab583d` (the held failure on `Interp`, cold error renders), `3c8978185` (the
`black_box` comment); records and this section in the commit after.

## Fix round 2

Status: DONE_WITH_CONCERNS. Code `9af69058d` (R4 as replaced, R6 narrowed) and `8dd9a4ae0` (perf); records,
the closing and this section in the commit after. Records: `docs/superpowers/records/2026-10-18-phase-9d/t8.7c/fix-r2/`
(`probes/r3.sh` the runner: `o` oracle, `b` `rexx-head-246ab583d` (round 1's head), `h` this round's head;
outputs under each probe directory's `out/`). This section supersedes round 1's R4 and R6 entries.

### What changed

* **R4 as replaced.** `debug_input_line` (`run/interpret.rs`) is an ordinary send of `LINEIN` again: round 1's
  traps-OFF window around the read is gone, and Task 8.7b round 3's `drop_queued_for_delayed` stays gone. The
  eight debug EOF programs are back in `corpus/phase-6-1.txt` where they were and out of `corpus/unfiled.txt`;
  `input_oracle/ruled` holds the R3 program alone, and `corpus.rs`'s sidecar check is as before round 1.
  `ir/drive.rs`'s header-pause delivery comment is restored.
* **R6 narrowed.** Every TRACE site round 1 moved to `string_representation` renders by `string_value_text`
  again, through a cold wrapper `traced_string_value_text` (Perf); the ADDITIONAL `>A>` pair renders as before
  round 1. `string_representation` remains for `stem_string_representation` alone (`Stem~string`, a stem's
  NOSTRING description).
* **Perf.** `eval`'s trace hook answers a flag and holds its condition (`trace_intermediate_held`,
  `take_held_trace_failure`); PARSE's source and patterns evaluate through `eval_in_parse`, which takes the
  hook's `Result` in place.

### Rulings

**R1 (unchanged).** Rule: a stem's string value is its default's `request('STRING')` (provide.xml, Required
String Values; collclasses.xml, Stem `request`); where that answers none, NOSTRING names the stem. Oracle:
NOSTRING `|7| Stem` in every required-string position, the default named in an arithmetic operand
(`t8.7c/fix-r1/probes/r1/s2.rex`). Witness `tests::a_stems_string_value_is_its_defaults_and_nostring_names_the_stem`.

**R2 (pinned, unchanged).** Rule: "Index equality is determined by using the `==` method of index"
(collclasses.xml, Table `at`/`put`). Oracle: identity first, `==` never sent
(`t8.7c/probes/hashid/hash-identity-first-idq.rex`). Witness `tests::a_hash_lookup_sends_the_identical_indexs_equality`.

**R3 (unchanged).** Rule: on an APPEND stream a write SEEK, a CHAROUT start or a LINEOUT line within the file's
extent at open is refused, 0 or the residual count and NOTREADY (funct.xml:4307-4312). Oracle: moves the write
position there (`t8.7c/fix-r1/probes/ap/a1.rex`). Witnesses: stream model case `append-extent`, and
`input_oracle::the_unfiled_ruled_programs_answer_as_this_crate_rules` over `stream_append_writes_at_end.rex`.

**R4 as replaced ("Traps off only while the line runs").** Rule: every condition trap is OFF while the line typed
at a pause runs (condtra.xml:489-494, Note 4; xintdeb.xml, "as though DO; line; END;"), so a condition the line
raises takes its default action and SYNTAX reports and prompts again; the read of `.DEBUGINPUT` is an ordinary
stream read, Deviation 75's delayed rule included. Oracle: the same (`RexxActivation::debugInterpret`,
`RexxActivation.cpp:2789`, sets `debugPause` around the line only; `trap` answers false for any other condition
while it is set). Witnesses:
* the eight debug EOF programs (`debug_input_eof.rex`, `_call_on`, `_call_on_loop`, `debug_input_stdin_eof_call_on.rex`,
  `debug_input_eof_iterate.rex`, `_leave`, `_iterate_outer`, `debug_handler_exit_at_iterate.rex`): o~h equal on
  stdout, stderr and rc, o~b differ (`probes/eof/`). The header-pause delivery arm has its witness back: with its
  guard made false (mutation build), `debug_input_eof_call_on_loop`, `debug_input_eof_iterate`,
  `debug_input_eof_iterate_outer` and `debug_handler_exit_at_iterate` differ from the oracle on stdout and stderr.
* new `corpus/lang/debug_typed_traps_off.rex` (with `.stdin`): a NOVALUE under SIGNAL ON NOVALUE, a USER raised
  into the line by a routine's `RAISE ... RETURN` under CALL ON USER and a NOTREADY from a LINEIN in the line
  under SIGNAL ON NOTREADY take their default actions, and the pause's read at the end of standard input
  reaches SIGNAL ON NOTREADY. o~h equal, o~b differ at that last read (`probes/off/all.rex`); with
  `ignored_in_debug_pause` answering false (mutation build) its stdout differs. NOTREADY under CALL ON in a
  typed line was already `debug_typed_condition_ignored.rex`.
* the pinned `delayed_trap_cases/crate` case "a condition the debug pause's input read raises is an ordinary
  read's": each of the six reads that raise USER runs the handler once, those read inside the handler held
  until its RETURN (`after 6`); bounded by its source returning `''` after six reads. The SYNTAX case is
  unchanged.
* `2026-10-29-debug-input-traps-off.md` closed under this ruling.
Not covered by this ruling and pre-existing: a typed `raise user ping` without RETURN, and a typed `exit 3`, end
the program on the oracle and are ignored here (`probes/off/us.rex`, `rr.rex`, `ex.rex`, `ex2.rex`, `ex3.rex`;
the RAISE half is the deferred `2026-10-10-debug-typed-raise-untrapped.md`).

**R5 (pinned, unchanged).** Rule: positioning a transient stream is 93.958 (iostrms.xml:194-197). Oracle: the
first SEEK of an unopened `.stream~new('STDIN')` answers `1 READY` (`t8.7c/probes/stdinq/q4.rex`). Witness
`input_oracle::the_first_seek_of_an_unopened_transient_stream_is_refused`.

**R6 narrowed ("Narrow to the Stem half").** Rule: a Stem's `STRING` is its default's string representation
(intro.xml:2653-2656), sent `STRING` when the default's class overrides it; a TRACE value line renders by the
object's name (`stringValue`), Moritz's licensed departure from instrc.xml:3697-3699. Oracle: `v.~string`
answers the default's name (`a NSTR`); the TRACE lines name the object (`>V>   V. => "a NSTR"`,
`probes/r6/t1.rex`). Witness `tests::a_stems_string_sends_string_to_its_default_and_a_trace_line_names_it`
(stdout red at `e620097f0`, stderr red at `246ab583d`). ooTest `base/keyword/TRACE` `TEST_TRACE_FORWARD`
passes again.

### Witness for the perf shape

`corpus/lang/trace_rendering_raises_in_parse_and_guard.rex`: a DEFAULTNAME raising 98.900 while TRACE I renders
a PARSE pattern, a PARSE VALUE source and a GUARD WHEN operand; o, b and h equal (`probes/hook/w.rex`). Making
`eval`'s hook drop its held condition reddens the GUARD case, and making `eval_in_parse` drop its `Result`
reddens the pattern case (mutation builds, `probes/hook/out/w.rex.m1.*`, `m2.*`).

### Perf

`fix-r2/callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 6 -o .../cg-9d/t8.7c/fix2-final2
base=.../rexx-base-97eee8a9a head=.../rexx-head-8dd9a4ae0`, every program, 3 rounds; every `.vg` names one of the
two binaries (99 each); no SPREAD, OVER or stdout flag in the log (the run's exit status was not recorded). Libc
subtracted against base, every program at most +0.05%:

| program | head -libc d% |
|---|---|
| arith | +0.0403 |
| parse | +0.0396 |
| pingsem | +0.0352 |
| startup | +0.0201 |
| hashnew | +0.0198 |
| decloop | +0.0171 |
| dispatchclass | -0.1304 |
| fibfunc | -0.1064 |

The rest lie between -0.0881 (heapshape) and +0.0107 (sayloop). Shapes measured on the way, one round each
(`fix-r2/callgrind-variants/`, `try.sh` the driver):
* at `9af69058d` (`callgrind-9af69058d.txt`): rexxcps +0.1334, pingguard +0.0703, compound +0.0556, from the
  TRACE sites calling `string_value_text`, which is not cold; the cold wrapper restores round 1's figures
  (`fix2-exp-a`: parse +0.0525, pingguard +0.0737).
* `eval` answering its trace hook early, as a tail call, a flag, or a cold flag (`-b`, `-c`, `-e`): pingguard
  -0.05 to +0.001, parse +0.3633 every time (`exec_parse` +26 Ir per pass, register allocation).
* the flag in `eval` and the `Result` in PARSE's evaluations (`-f`, `-h`): every measured program within budget.
  Sharing their prologue through an `#[inline(always)]` helper (`-g`) gives parse +0.1172 and pingguard +0.1051,
  so the two bodies stay separate.

### Commands

All from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t87c-fix2-target` but the
workspace run, at `8dd9a4ae0`; outputs in `fix-r2/`.

| command | result |
|---|---|
| `cargo fmt --all --check` (`checks-fmt.txt`) | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec -p rexx-core --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-core -p rexx-exec --lib` and the `--test` binaries of round 1's touched list (`checks-touched.txt`) | 0; 1553 passed, lib 1211 |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101: 1233 of 1234 match, the known `rexxutil_file_date_time.rex` |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-parse --test sourceline_oracle --test errors --test tiling --test variants` (`checks-parse.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`) | 0, 74 of 74 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../fix-r2/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`checks-suite.txt`) | 0; `tests.tsv`, `table.tsv`, `verdicts.tsv` and `oracle-differences.tsv` byte-identical to `t8.7c/suite/` (`cmp`). The run at `9af69058d` differed only in SysFileTree `TEST_TREE_DIRECTORY`'s oracle column, the known flake |
| `memcap 16G cargo build --release -j 4 --bin rexx` then `memcap 16G cargo test --release -j 4 --no-fail-fast --workspace`, default target dir (`close-workspace-build.txt`, `close-workspace-tests.txt`, `close-workspace-status.txt`) | build 0; tests 101, 3536 passed, 2 failed: `datetime_zone`'s `the_file_date_time_witness_reads_the_zone_the_oracle_does` (the known DST flake) and `suite`'s `fixtures::a_child_refusing_on_the_inherited_stderr_leaves_the_run_ended_normally` (`passed` `{}` against `{"TEST_A"}`, 10.15 s); that test alone passes 3 of 3 in 0.22 s (`close-workspace-rerun-fixture.txt`) |

### Concerns

1. Deviation 75's sentence "A condition the debug pause's own input read raises while its trap is delayed is
   dropped" is false at `9af69058d`; not edited here (the controller writes Deviations).
2. A typed EXIT at a debug pause is ignored here and ends the program on the oracle (`probes/off/ex.rex`,
   `ex2.rex`, `ex3.rex`; `run_debug_fragment` drops the fragment's `Flow`). No queued item names EXIT; queue
   candidate, beside the deferred typed-RAISE and typed-SIGNAL items.
3. `fixtures::a_child_refusing_on_the_inherited_stderr_leaves_the_run_ended_normally` failed once under the
   workspace run's load and is not on the known-flake list.
4. The workspace run was started through `nohup sh -c`, against the no-wrapper rule.
5. The perf shape rests on two copies of `eval`'s body, measured rather than understood (register allocation
   in `exec_parse`).

### Commits

`9af69058d` (R4 as replaced, R6 narrowed, witnesses), `8dd9a4ae0` (the cold TRACE rendering wrapper, `eval`'s
flag hook, `eval_in_parse`, its witness); records and this section in the commit after.
